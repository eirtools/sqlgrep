use crate::cell_to_string::sqlite_cell_to_string;
use crate::error::Level;
use crate::{Pattern, SQLError};

use sqlx::{Column, Executor, Pool, Row, Sqlite};

pub async fn sqlite_check_rows(
    db: &Pool<Sqlite>,
    query_id: &str,
    select_query: &str,
    pattern: &Pattern,
) {
    use futures::TryStreamExt;
    use std::sync::atomic::AtomicU64;
    use std::sync::atomic::Ordering;

    log::debug!("{query_id}: {select_query}");

    let mut rows = db.fetch(select_query);

    let row_counter: AtomicU64 = AtomicU64::new(0);
    loop {
        let row_idx = row_counter.load(Ordering::SeqCst);

        let row = match rows.try_next().await {
            Ok(None) => break,
            Ok(Some(row)) => row,
            Err(error) => {
                SQLError::SqlX((format!("{query_id}::{row_idx}"), error)).report(Level::Warn);
                continue;
            }
        };

        sqlite_process_row(row_idx, &row, query_id, pattern);
        row_counter.fetch_add(1, Ordering::SeqCst);
    }
}

fn sqlite_process_row(
    row_idx: u64,
    row: &sqlx::sqlite::SqliteRow,
    query_id: &str,
    pattern: &Pattern,
) {
    use sqlx::TypeInfo;
    let columns = row.columns();
    for column in columns {
        let index = column.ordinal();
        let column_name = column.name().to_owned();
        let column_type = column.type_info().name();
        let row_id = format!("{query_id}::{row_idx}::{column_name}");

        let value_ref = match row.try_get_raw(index) {
            Ok(value_ref) => value_ref,
            Err(error) => {
                SQLError::SqlX((row_id, error)).report(Level::Warn);
                continue;
            }
        };

        let value_str = match sqlite_cell_to_string(value_ref) {
            Ok(Some(value_str)) => value_str,
            Ok(None) => continue,
            Err(error) => {
                let error_context = format!("{row_id} cell type {column_type}");
                SQLError::ConvertCell((error_context, error)).report(Level::Warn);
                continue;
            }
        };

        if pattern.is_match(&value_str) {
            println!("{row_id} => {value_str}");
        }
    }
}
