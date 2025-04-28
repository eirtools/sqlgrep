mod args;
mod cell_to_string;
mod error;
mod matching;
mod pattern;
mod query;
mod select;

use std::fs::OpenOptions;
use std::io::stdin;

use error::Level;
use error::SQLError;
use matching::sqlite_check_rows;
use pattern::{Pattern, PatternKind};
use query::{prepare_queries, SelectVariant};
use sqlparser::dialect::SQLiteDialect;

use sqlx::{sqlite::SqliteConnectOptions, Executor as _, Pool, Row as _, Sqlite, SqlitePool};

#[tokio::main()]
async fn main() {
    let args = args::parse_args();
    // Set default log level to 2.
    let quiet_level: i16 = 2 + i16::from(args.verbose) - i16::from(args.quiet);

    stderrlog::new()
        .module(module_path!())
        .quiet(quiet_level < 0)
        .verbosity(quiet_level.unsigned_abs() as usize)
        .timestamp(stderrlog::Timestamp::Off)
        .color(stderrlog::ColorChoice::Auto)
        .init()
        .unwrap();

    let pattern = match Pattern::new(args.pattern.as_str(), &PatternKind::Regex) {
        Ok(pattern) => pattern,
        Err(err) => std::process::exit(err.report(Level::Error)),
    };

    let queries = match read_queries(args.query) {
        Ok(queries) => queries,
        Err(err) => std::process::exit(err.report(Level::Error)),
    };

    match process_sqlite_database(
        args.database_uri,
        pattern,
        args.table,
        queries,
        args.ignore_non_readonly,
    )
    .await
    {
        Ok(()) => {}
        Err(err) => std::process::exit(err.report(Level::Error)),
    }
}

async fn process_sqlite_database(
    database_uri: String,
    pattern: Pattern,
    tables: Vec<String>,
    queries: Vec<String>,
    ignore_non_read: bool,
) -> Result<(), SQLError> {
    let dialect = SQLiteDialect {};

    let options: SqliteConnectOptions = database_uri
        .parse::<SqliteConnectOptions>()
        .map(|options| options.read_only(true).immutable(true))
        .map_err(|error| SQLError::SqlX(("Database URI".into(), error)))?;

    let db = SqlitePool::connect_with(options)
        .await
        .map_err(|error| SQLError::SqlX(("Database connection".into(), error)))?;

    let select_variant = prepare_queries(
        tables.into_iter(),
        queries.into_iter(),
        &dialect,
        ignore_non_read,
    )?;

    let queries = match select_variant {
        SelectVariant::Queries(queries) => queries,
        SelectVariant::WholeDB => {
            let tables = sqlite_select_tables(&db).await?;
            let select_variant = prepare_queries(
                tables.into_iter(),
                vec![].into_iter(),
                &dialect,
                ignore_non_read,
            )?;
            match select_variant {
                SelectVariant::WholeDB => vec![],
                SelectVariant::Queries(queries) => queries,
            }
        }
    };

    for (query_id, query) in queries {
        sqlite_check_rows(&db, query_id.as_str(), query.as_str(), &pattern).await;
    }

    Ok(())
}

async fn sqlite_select_tables(db: &Pool<Sqlite>) -> Result<Vec<String>, SQLError> {
    let select_query = "SELECT name FROM sqlite_schema WHERE type = 'table'";

    log::debug!("Execute query: {select_query}");

    let result = db
        .fetch_all(select_query)
        .await
        .map_err(|err| SQLError::SqlX(("fetch tables".into(), err)))?;

    Ok(result
        .into_iter()
        .filter_map(|row| match row.try_get::<String, &str>("name") {
            Ok(value) => Some(value),
            Err(err) => {
                SQLError::SqlX(("fetch tables".into(), err)).report(Level::Warn);
                None
            }
        })
        .collect())
}

fn read_queries(queries: Vec<String>) -> Result<Vec<String>, SQLError> {
    let mut acc = vec![];

    queries.into_iter().try_fold((), |(), query| {
        if query.is_empty() {
            return Ok(());
        }

        if query == "-" {
            return read_query(&mut stdin(), "<stdin>").map(|query| {
                acc.push(query);
            });
        }

        match query.strip_prefix('@') {
            None => {
                acc.push(query);
                Ok(())
            }
            Some(filename) => read_from_file(filename).map(|query| {
                acc.push(query);
            }),
        }
    })?;

    Ok(acc)
}

#[inline]
fn read_from_file(filename: &str) -> Result<String, SQLError> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(filename)
        .expect("Unable to open");

    read_query(&mut file, filename)
}

#[inline]
fn read_query<File>(file: &mut File, filename: &str) -> Result<String, SQLError>
where
    File: std::io::Read,
{
    let mut query = String::new();

    file.read_to_string(&mut query)
        .map_err(|error| SQLError::Io((format!("read {filename}"), error)))
        .map(|_| query)
}
