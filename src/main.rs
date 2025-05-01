mod args;
mod cell_to_string;
mod error;
mod matching;
mod pattern;
mod query;
mod select;

use std::io::stdin;
use std::io::Read;

use error::Level;
use error::SQLError;
use matching::sqlite_check_rows;
use pattern::Pattern;
use query::{prepare_queries, SelectVariant};
use sqlparser::dialect::SQLiteDialect;

use sqlx::{sqlite::SqliteConnectOptions, Executor as _, Pool, Row as _, Sqlite, SqlitePool};

#[tokio::main()]
async fn main() {
    let args = args::parse_args();

    setup_logging(args.verbose.level());

    let pattern = create_pattern(&args.pattern)
        .unwrap_or_else(|error| std::process::exit(error.report(Level::Error)));

    let queries = match read_queries(args.query.query, stdin) {
        Ok(queries) => queries,
        Err(error) => std::process::exit(error.report(Level::Error)),
    };

    match process_sqlite_database(
        args.database_uri,
        pattern,
        args.query.table,
        queries,
        args.query.ignore_non_readonly,
    )
    .await
    {
        Ok(()) => {}
        Err(error) => std::process::exit(error.report(Level::Error)),
    }
}

fn setup_logging(verbosity_level: i16) {
    stderrlog::new()
        .module(module_path!())
        .quiet(verbosity_level < 0)
        .verbosity(verbosity_level.unsigned_abs() as usize)
        .timestamp(stderrlog::Timestamp::Off)
        .color(stderrlog::ColorChoice::Auto)
        .init()
        .unwrap();
}

fn create_pattern(options: &args::PatternArgs) -> Result<Pattern, SQLError> {
    let kind = if options.fixed {
        pattern::PatternKind::Fixed
    } else {
        pattern::PatternKind::Regex
    };

    Pattern::new(
        options.pattern.as_str(),
        &kind,
        pattern::PatternOptions {
            case_insensitive: options.case_insensitive,
            whole_string: options.whole_string,
        },
    )
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
        .map_err(|error| SQLError::SqlX(("fetch tables".into(), error)))?;

    Ok(result
        .into_iter()
        .filter_map(|row| match row.try_get::<String, &str>("name") {
            Ok(value) => Some(value),
            Err(error) => {
                SQLError::SqlX(("fetch tables".into(), error)).report(Level::Warn);
                None
            }
        })
        .collect())
}

fn read_queries<R: Read>(
    queries: Vec<String>,
    stdin_func: fn() -> R,
) -> Result<Vec<String>, SQLError> {
    let mut acc = vec![];

    queries
        .into_iter()
        .try_fold((), |(), query| {
            let query = query.trim();
            let mut chars = query.chars();
            let query = match chars.next() {
                Some('-') => {
                    // This comparison is correct since we compare with an ASCII character
                    // Alternatively, matches!(chars.next(), None)
                    if query.len() == 1 {
                        let mut query = String::new();

                        let _ = stdin_func()
                            .read_to_string(&mut query)
                            .map_err(|error| SQLError::Io(("read <stdin>".to_string(), error)))?;

                        query
                    } else {
                        query.to_owned()
                    }
                }
                Some('@') => {
                    let filename: &str = chars.as_str(); // first char is already removed

                    std::fs::read_to_string(filename)
                        .map_err(|error| SQLError::Io((format!("read \"{filename}\""), error)))?
                }
                None | Some(_) => query.to_owned(),
            };

            if !query.is_empty() {
                acc.push(query);
            }

            Ok::<(), SQLError>(())
        })
        .map(|()| acc)
}
