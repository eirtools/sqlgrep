use std::fmt::Display;

use log::warn;
use sqlx::sqlite::SqliteValueRef;
use sqlx::Decode;
use sqlx::Sqlite;
use sqlx::Type;
use sqlx::TypeInfo;
use sqlx::ValueRef;

// REVIEW: better way convert an error to a string?
fn errr_format(value: impl Display) -> String {
    format!("{value}")
}

pub(crate) fn sqlite_cell_to_string(value_ref: SqliteValueRef) -> Result<Option<String>, String> {
    // TODO: add an option to override types in some extent

    if value_ref.is_null() {
        return Ok(None);
    }

    let type_info = value_ref.type_info().into_owned();

    // TEXT
    if <String as Type<Sqlite>>::compatible(&type_info) {
        let value = <String as Decode<Sqlite>>::decode(value_ref).map_err(errr_format)?;
        return Ok(Some(value));
    }

    // // INTEGER, INT4
    if <i64 as Type<Sqlite>>::compatible(&type_info) {
        let value = <i64 as Decode<Sqlite>>::decode(value_ref).map_err(errr_format)?;
        return Ok(Some(format!("{value}")));
    }
    // REAL
    if <f64 as Type<Sqlite>>::compatible(&type_info) {
        let value = <f64 as Decode<Sqlite>>::decode(value_ref).map_err(errr_format)?;
        return Ok(Some(format!("{value}")));
    }
    // BOOL?
    if <bool as Type<Sqlite>>::compatible(&type_info) {
        let value = <bool as Decode<Sqlite>>::decode(value_ref).map_err(errr_format)?;
        return Ok(Some(format!("{value}")));
    }
    // DateTime
    if <chrono::DateTime<chrono::Local> as Type<Sqlite>>::compatible(&type_info) {
        let value = <chrono::DateTime<chrono::Local> as Decode<Sqlite>>::decode(value_ref)
            .map_err(errr_format)?;
        return Ok(Some(value.to_rfc3339()));
    }
    // Date
    if <chrono::NaiveDate as Type<Sqlite>>::compatible(&type_info) {
        let value =
            <chrono::NaiveDate as Decode<Sqlite>>::decode(value_ref).map_err(errr_format)?;
        return Ok(Some(value.format("%Y-%m-%d").to_string()));
    }
    // Time
    if <chrono::NaiveTime as Type<Sqlite>>::compatible(&type_info) {
        let value =
            <chrono::NaiveTime as Decode<Sqlite>>::decode(value_ref).map_err(errr_format)?;
        return Ok(Some(value.format("%H:%M:%S").to_string()));
    }

    // BLOB
    if <Vec<u8> as Type<Sqlite>>::compatible(&type_info) {
        // ignore this type for now.
        // let _  = <Vec<u8> as Decode<Sqlite>>::decode(value_ref).map_err(_format)?;
        // TODO: add option to try decode as zlib string
        // TODO: add option to try decode as UTF-8 bytes
        // TODO: add option to try decode as UTF-16 bytes?
        // TODO: add option to try decode as UUID
        return Ok(None);
    }

    warn!("Unknown cell type: {}", type_info.name());
    Ok(None)
}
