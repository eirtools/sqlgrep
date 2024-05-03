use sqlparser::ast::*;
use sqlparser::dialect::Dialect;

/// Generates wildcard select for given dialect
///
/// ```rust
/// // connect to SQLite
/// use sqlparser::dialect::SQLiteDialect;
/// let driver = SQLiteDialect{};
/// let query = generate_select("table", driver);
/// assert_eq!("SELECT * FROM `table`", query.as_str());
/// ```
pub(crate) fn generate_select(table_name: &str, dialect: &impl Dialect) -> String {
    let ast = SetExpr::Select(Box::new(Select {
        distinct: None,
        top: None,
        projection: [SelectItem::Wildcard(WildcardAdditionalOptions {
            opt_exclude: None,
            opt_except: None,
            opt_rename: None,
            opt_replace: None,
        })]
        .to_vec(),
        into: None,
        from: [TableWithJoins {
            relation: TableFactor::Table {
                name: ObjectName(
                    [Ident {
                        value: table_name.to_owned(),
                        quote_style: dialect.identifier_quote_style(table_name),
                    }]
                    .to_vec(),
                ),
                alias: None,
                args: None,
                with_hints: [].to_vec(),
                version: None,
                partitions: [].to_vec(),
            },
            joins: [].to_vec(),
        }]
        .to_vec(),
        lateral_views: [].to_vec(),
        selection: None,
        group_by: GroupByExpr::Expressions([].to_vec()),
        cluster_by: [].to_vec(),
        distribute_by: [].to_vec(),
        sort_by: [].to_vec(),
        having: None,
        named_window: [].to_vec(),
        qualify: None,
        value_table_mode: None,
    }));

    ast.to_string()
}
