mod args;
mod cell_to_string;
mod select;

use sqlparser::dialect::SQLiteDialect;

use sqlx::sqlite::SqliteConnectOptions;
use sqlx::Error;
use sqlx::{Column, Executor, Pool, Row, Sqlite, SqlitePool};

#[tokio::main()]
async fn main() {
    let args = args::parse_args();
    // Set default log level to 2.
    let quiet_level: i16 = 2 + args.verbose as i16 - args.quiet as i16;

    stderrlog::new()
        .module(module_path!())
        .quiet(quiet_level < 0)
        .verbosity(quiet_level.unsigned_abs() as usize)
        .timestamp(stderrlog::Timestamp::Off)
        .color(stderrlog::ColorChoice::Auto)
        .init()
        .unwrap();

    let pattern = match regex::Regex::new(&args.pattern) {
        Ok(pattern) => pattern,
        Err(err) => {
            log::error!("Unable to compile pattern: {}", err);
            std::process::exit(64);
        }
    };

    let select_variant = match args.table {
        None => SelectVariant::WholeDB,
        Some(table_names) => SelectVariant::SpecificTables(table_names),
    };

    match process_sqlite_database(args.database_uri, &pattern, select_variant).await {
        Ok(_) => {}
        Err(err) => {
            log::error!("Unable read tables from database: {}", err);
            std::process::exit(74);
        }
    }
}

async fn process_sqlite_database(
    database_uri: String,
    pattern: &regex::Regex,
    select_variant: SelectVariant,
) -> Result<(), Error> {
    let dialect = SQLiteDialect {};

    let options: SqliteConnectOptions = match database_uri.parse::<SqliteConnectOptions>() {
        Ok(options) => options.read_only(true).immutable(true),
        Err(err) => {
            log::error!("Database URI error: {}", err);
            std::process::exit(64);
        }
    };

    let db = match SqlitePool::connect_with(options).await {
        Ok(db) => db,
        Err(err) => {
            log::error!("Database connection error: {}", err);
            std::process::exit(74);
        }
    };

    match select_variant {
        SelectVariant::WholeDB => match sqlite_select_tables(&db).await {
            Ok(table_names) => {
                let mut table_names = table_names;
                sqlite_select_from_tables(&db, &mut table_names, pattern, &dialect).await
            }
            Err(err) => Err(err),
        },
        SelectVariant::SpecificTables(table_names) => {
            let mut table_names = table_names.into_iter();
            sqlite_select_from_tables(&db, &mut table_names, pattern, &dialect).await
        }
    }
}

async fn sqlite_select_from_tables<Iter>(
    db: &Pool<Sqlite>,
    table_names: &mut Iter,
    pattern: &regex::Regex,
    dialect: &SQLiteDialect,
) -> Result<(), Error>
where
    Iter: Iterator<Item = String>,
{
    for table_name in table_names {
        let select_query = select::generate_select(table_name.as_str(), dialect);
        sqlite_check_rows(&table_name, db, select_query.as_str(), pattern).await;
    }

    Ok(())
}

async fn sqlite_select_tables(db: &Pool<Sqlite>) -> Result<impl Iterator<Item = String>, Error> {
    let select_query = "SELECT name
        FROM sqlite_schema
        WHERE type ='table'";

    log::debug!("Execute query: {select_query}");
    let result = db.fetch_all(select_query).await?;

    Ok(result
        .into_iter()
        .filter_map(|row| match row.try_get::<String, &str>("name") {
            Ok(value) => Some(value),
            Err(err) => {
                log::warn!("Error while reading from table `sqlite_schema`: {}", err);
                None
            }
        }))
}

async fn sqlite_check_rows(
    table_name: &String,
    db: &Pool<Sqlite>,
    select_query: &str,
    pattern: &regex::Regex,
) {
    use futures::TryStreamExt;
    use std::sync::atomic::AtomicI64;
    use std::sync::atomic::Ordering;

    log::debug!("Execute query: {select_query}");
    let mut rows = db.fetch(select_query);

    log::debug!("==> {table_name}");
    let row_idx: AtomicI64 = AtomicI64::new(-1);
    loop {
        row_idx.fetch_add(1, Ordering::SeqCst);
        let idx = row_idx.load(Ordering::SeqCst);

        let row = match rows.try_next().await {
            Ok(None) => break,
            Ok(Some(row)) => row,
            Err(err) => {
                log::warn!(
                    "Error while reading row {idx} from table `{table_name}`: {}",
                    err
                );
                continue;
            }
        };

        sqlite_process_row(idx, row, table_name, pattern);
    }
}

fn sqlite_process_row(
    row_idx: i64,
    row: sqlx::sqlite::SqliteRow,
    table_name: &String,
    pattern: &regex::Regex,
) {
    use sqlx::TypeInfo;
    let columns = row.columns();
    for column in columns {
        let index = column.ordinal();
        let column_name = column.name().to_owned();

        let value_ref = match row.try_get_raw(index) {
            Ok(value_ref) => value_ref,
            Err(err) => {
                log::warn!("Error while reading row {row_idx} from table {table_name} column {column_name}: {}", err);
                continue;
            }
        };

        let value_str = match cell_to_string::sqlite_cell_to_string(value_ref) {
            Ok(Some(value_str)) => value_str,
            Ok(None) => continue,
            Err(err) => {
                let column_type = column.type_info().name();
                log::warn!("Error while converting data from row {row_idx} from table {table_name} column {column_name} of type {column_type}: {}", err);
                continue;
            }
        };

        if pattern.is_match(&value_str) {
            println!("{table_name}::{row_idx}::{column_name} => {value_str:?}");
        }
    }
}

#[non_exhaustive]
enum SelectVariant {
    WholeDB,
    SpecificTables(Vec<String>),
}
