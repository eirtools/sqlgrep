use sqlparser::ast::helpers::attached_token::AttachedToken;
use sqlparser::ast::{
    GroupByExpr, Ident, Select, SelectFlavor, SelectItem, SetExpr, Statement, TableFactor,
    TableWithJoins, WildcardAdditionalOptions,
};
use sqlparser::dialect::Dialect;
use sqlparser::parser::Parser;
use sqlparser::tokenizer::Span;

use crate::error::{QueryError, SQLError};

///
///  Generates wildcard select for given dialect:
///
/// ```rust
/// // connect to SQLite
/// use sqlparser::dialect::SQLiteDialect;
/// let driver = SQLiteDialect{};
/// let query = generate_select("table", driver);
/// assert_eq!("SELECT * FROM `table`", query.as_str());
/// ```
///
pub(crate) fn generate_select(table_name: &str, dialect: &impl Dialect) -> String {
    let ast = SetExpr::Select(Box::new(Select {
        flavor: SelectFlavor::Standard,
        distinct: None,
        top: None,
        projection: [SelectItem::Wildcard(WildcardAdditionalOptions {
            wildcard_token: AttachedToken::empty(),
            opt_ilike: None,
            opt_exclude: None,
            opt_except: None,
            opt_rename: None,
            opt_replace: None,
        })]
        .to_vec(),
        into: None,
        from: [TableWithJoins {
            relation: TableFactor::Table {
                name: [Ident {
                    value: table_name.to_owned(),
                    quote_style: dialect.identifier_quote_style(table_name),
                    span: Span::empty(),
                }]
                .to_vec()
                .into(),
                alias: None,
                args: None,
                with_hints: vec![],
                version: None,
                partitions: vec![],
                with_ordinality: false,
                json_path: None,
                sample: None,
                index_hints: vec![],
            },
            joins: vec![],
        }]
        .to_vec(),
        lateral_views: [].to_vec(),
        selection: None,
        group_by: GroupByExpr::Expressions(vec![], vec![]),
        cluster_by: [].to_vec(),
        distribute_by: [].to_vec(),
        sort_by: [].to_vec(),
        having: None,
        named_window: [].to_vec(),
        qualify: None,
        value_table_mode: None,
        select_token: AttachedToken::empty(),
        top_before_distinct: false,
        prewhere: None,
        window_before_qualify: false,
        connect_by: None,
    }));

    ast.to_string()
}

pub(crate) fn escape_table_name(table_name: &str, dialect: &impl Dialect) -> String {
    Ident {
        value: table_name.to_owned(),
        quote_style: dialect.identifier_quote_style(table_name),
        span: Span::empty(),
    }
    .to_string()
}

/// Checks and reformat select
pub(crate) fn read_verify_query(
    sql: &str,
    dialect: &impl Dialect,
    ignore_non_read: bool,
    idx: &mut usize,
) -> Result<Vec<String>, SQLError> {
    let ast = Parser::parse_sql(dialect, sql).map_err(SQLError::ParseError)?;
    let mut acc: Vec<String> = vec![];
    ast.iter().try_fold((), |(), statement| {
        if matches!(statement, Statement::Query(_)) {
            acc.push(statement.to_string());
            *idx += 1;
            Ok(())
        } else if ignore_non_read {
            Ok(())
        } else {
            Err(SQLError::QueryError(QueryError::ReadOnlyQueryAllowed))
        }
    })?;

    Ok(acc)
}
