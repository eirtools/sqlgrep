# SQL Grep

## Overview

Grep SQL like other files on your filesystem and find where specific pattern occurs.

It's easy to find a situation when we know that there's some specific data inside a database, and it's not known where to search it.

In this scenario `sqlgrep` tool is very handy as it's easy to find which cells are interconnected and how.

This could be useful for development or reverse engineering.

This tool reads all cells, converts data to a string, then trying to match a pattern.

## Input

SQL Grep is a command line tool, and most of the data takes from command line arguments.

There's some notable information:

* Pattern is powered by [regex crate](https://lib.rs/crates/regex) by default. Also there's option to change matching to fixed string.
* Table names passed are properly escaped.
* User can pass multiple SQL queries with command line arguments. Every SQL query may contain multiple queries and only `SELECT` queries are currently supported. Every SQL query is validated and reformatted as needed. Values can be passed multiple times as follow:
    * RAW SQL from argument value.
    * From STDIN, by passing `-`.
    * From file by preceding filename with `@` (similar to CURL).
* Queries to select all data from tables (e.g. `SELECT * from table`) and user defined queries combined before execution. If this list contains 2 the same queries, both of them will be executed one after another.
* For SQLite databases, it can be passed by filename and/or URL with `sqlite://` scheme.
* SQLite databases are opened with `readonly` and `immutable` options turned on. Please, have a look on SQLite3 [official documentation](https://sqlite.org/c3ref/open.html) for more information for details.

## Output

Data is read from each SQL query, supported columns are converted to string which is matched to the pattern provided.

If string value matches the pattern, then output in format below is generated.

Output has following format:

`<Table or Query>::<Row index>::<Column>::<Value>`.

`Table or Query` is `Table <table name>` or `Query <query index>` for a user to identify the source of information.

`<Row index>` is plain index starting with `0` of given select.

**NOTE:** `Row Index` has nothing in common with `rowid`, which is not queried. It's just plain index. In some cases, it may be the same as `rowid - 1`.

`<Column>` column name which has match.

`<Value>` is string value after conversion.

**NOTE:** Even `Value` is an UTF-8 string, it isn't sanitized, so be warned what data you checking. PR is welcome. AFAIK, most of tools which do pattern search doesn't do any sanitization for output.

## Contributing

PR are are always welcome
