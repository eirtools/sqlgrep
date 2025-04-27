use log::Level;
use sqlparser::parser::ParserError;

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
                };

                65
            }
            SQLError::ParseError(error) => {
                log::log!(level, "Unable to parse SQL: {error}");

                66
            }
            SQLError::SqlX((context, error)) => {
                let context = if context.is_empty() {
                    "".to_owned()
                } else {
                    format!(" ({context})")
                };

                log::log!(level, "Query execution error{context}: {error}");

                74
            }
            SQLError::ConvertCell((context, error)) => {
                let context = if context.is_empty() {
                    "".to_owned()
                } else {
                    format!(" ({context})")
                };

                log::log!(level, "Query execution error{context}: {error}");

                73
            }
        }
    }
}
