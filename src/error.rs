use log::Level as LogLevel;
use sqlparser::parser::ParserError;

pub(crate) type Level = LogLevel;

pub(crate) enum SQLError {
    Regex(regex::Error),
    QueryError(QueryError),
    ParseError(ParserError),
    SqlX((String, sqlx::Error)),
    ConvertCell((String, String)),
}

pub(crate) enum QueryError {
    ReadOnlyQueryAllowed,
}

impl SQLError {
    /// Report and return error code if needed
    pub fn report(&self, level: Level) -> i32 {
        match self {
            SQLError::Regex(error) => {
                log::log!(level, "Regex error: {error}");

                64
            }
            SQLError::QueryError(query_error) => {
                match query_error {
                    QueryError::ReadOnlyQueryAllowed => {
                        log::log!(level, "Only readonly query is allowed");
                    }
                }

                65
            }
            SQLError::ParseError(error) => {
                log::log!(level, "Unable to parse SQL: {error}");

                66
            }
            SQLError::SqlX((context, error)) => {
                let context = format_context(context);
                log::log!(level, "SQL error{context}: {error}");

                74
            }
            SQLError::ConvertCell((context, error)) => {
                let context = format_context(context);

                log::log!(level, "Cell conversion error{context}: {error}");

                73
            }
        }
    }
}

#[inline]
fn format_context(context: &String) -> String {
    if context.is_empty() {
        String::new()
    } else {
        format!(" ({context})")
    }
}
