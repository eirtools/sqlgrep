use clap::{ArgAction, Parser};
use indoc::indoc;

pub(crate) fn parse_args() -> Args {
    Args::parse()
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[command(flatten)]
    pub(crate) verbose: Verbose,

    #[command(flatten)]
    pub(crate) pattern: PatternArgs,

    #[command(flatten)]
    pub(crate) query: QueryArgs,

    #[arg(help = indoc!("
    Database URI to connect to.

    General URL format is <database>://username:password@hostname?<options>.

    For SQLite username, password are never, hostname is just a file path.
    If none of options provided, uri provided support direct filename.

    Currently supported databases: SQLite 3.x with prefix sqlite
    "
    ))]
    pub(crate) database_uri: String,
}

#[derive(Parser, Debug)]
pub struct Verbose {
    #[arg(help = "Decrease verbosity")]
    #[arg(short = 'q', long = "quiet")]
    #[arg(action=ArgAction::Count)]
    pub(crate) quiet: u8,

    #[arg(help = "Increase verbosity")]
    #[arg(short = 'v', long = "verbose")]
    #[arg(action=ArgAction::Count)]
    pub(crate) verbose: u8,
}

#[derive(Parser, Debug)]
pub struct PatternArgs {
    #[arg(short = 'F', long = "pattern-fixed")]
    #[arg(help = "Pattern is a fixed string")]
    #[arg(action=ArgAction::SetTrue)]
    pub(crate) fixed: bool,

    #[arg(short = 'W', long = "pattern-whole")]
    #[arg(help = "Pattern matches whole string")]
    #[arg(action=ArgAction::SetTrue)]
    pub(crate) whole_string: bool,

    #[arg(short = 'i', long = "pattern-case-insensitive")]
    #[arg(help = "Pattern is case insensitive")]
    #[arg(action=ArgAction::SetTrue)]
    pub(crate) case_insensitive: bool,

    #[arg(help = "Pattern to match every cell with")]
    pub(crate) pattern: String,
}

#[derive(Parser, Debug)]
pub struct QueryArgs {
    #[arg(short = 't', long = "table")]
    #[arg(help = "Table or view to query. Can be used multiple times")]
    #[arg(action=ArgAction::Append)]
    pub(crate) table: Vec<String>,

    #[arg(short = 's', long = "sql")]
    #[arg(help = "SQL query to run. Can be used multiple times")]
    #[arg(action=ArgAction::Append)]
    pub(crate) query: Vec<String>,

    #[arg(short = 'I', long = "ignore")]
    #[arg(help = "Ignore non-readonly queries")]
    #[arg(action=ArgAction::SetTrue)]
    pub(crate) ignore_non_readonly: bool,
}

impl Verbose {
    /// Verbosity level.
    ///
    /// Default log level is Info (2)
    pub fn level(&self) -> i16 {
        2 + i16::from(self.verbose) - i16::from(self.quiet)
    }
}
