use crate::cell_to_string::sqlite_cell_to_string;
use crate::{Pattern, SQLError};
use log::Level;
use sqlx::{Column, Executor, Pool, Row, Sqlite};

pub async fn sqlite_check_rows(
    db: &Pool<Sqlite>,
    query_id: &str,
    select_query: &str,
    pattern: &Pattern,
) {
    use futures::TryStreamExt;
    use std::sync::atomic::AtomicI64;
    use std::sync::atomic::Ordering;

    log::debug!("{query_id}: {select_query}");

    let mut rows = db.fetch(select_query);

    let row_idx: AtomicI64 = AtomicI64::new(-1);
    loop {
        row_idx.fetch_add(1, Ordering::SeqCst);
        let idx = row_idx.load(Ordering::SeqCst);

        let row = match rows.try_next().await {
            Ok(None) => break,
            Ok(Some(row)) => row,
            Err(err) => {
                log::warn!(
                    "Error while reading row {idx} while executing query: {}",
                    err
                );
                continue;
            }
        };

        sqlite_process_row(idx, row, query_id, pattern);
    }
}

fn sqlite_process_row(
    row_idx: i64,
    row: sqlx::sqlite::SqliteRow,
    query_id: &str,
    pattern: &Pattern,
) {
    use sqlx::TypeInfo;
    let columns = row.columns();
    for column in columns {
        let index = column.ordinal();
        let column_name = column.name().to_owned();
        let column_type = column.type_info().name();

        let error_context =
            format!("Reading row {row_idx} from table {query_id} column {column_name} of type {column_type}");

        let value_ref = match row.try_get_raw(index) {
            Ok(value_ref) => value_ref,
            Err(error) => {
                SQLError::SqlX((error_context, error)).report(Level::Warn);
                continue;
            }
        };

        let value_str = match sqlite_cell_to_string(value_ref) {
            Ok(Some(value_str)) => value_str,
            Ok(None) => continue,
            Err(error) => {
                SQLError::ConvertCell((error_context, error)).report(Level::Warn);
                continue;
            }
        };

        if pattern.is_match(&value_str) {
            println!("{query_id}::{row_idx}::{column_name} => {value_str:?}");
        }
    }
}
