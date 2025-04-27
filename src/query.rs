use sqlparser::dialect::Dialect;

use crate::error::SQLError;
use crate::select::{escape_table_name, generate_select, read_verify_query};

#[non_exhaustive]
pub(crate) enum SelectVariant {
    WholeDB,
    Queries(Vec<(String, String)>),
}

pub(crate) fn prepare_queries<T>(
    table: T,
    queries: T,
    dialect: &impl Dialect,
    ignore_non_read: bool,
) -> Result<SelectVariant, SQLError>
where
    T: Iterator<Item = String>,
{
    let mut queries_result: Vec<(String, String)> = table
        .map(|table_name| {
            (
                format!("Table {}", escape_table_name(table_name.as_str(), dialect)),
                generate_select(&table_name, dialect),
            )
        })
        .collect();

    let mut idx = 0usize;
    queries.into_iter().try_fold((), |_, sql| {
        read_verify_query(&sql, dialect, ignore_non_read, &mut idx)?
            .into_iter()
            .for_each(|query| queries_result.push((format!("Query #{idx}"), query)));
        Ok(())
    })?;

    Ok(if queries_result.is_empty() {
        SelectVariant::WholeDB
    } else {
        SelectVariant::Queries(queries_result)
    })
}
