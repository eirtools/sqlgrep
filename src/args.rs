use clap::{ArgAction, Parser};
use indoc::indoc;

pub(crate) fn parse_args() -> Args {
    Args::parse()
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(help = "Decrease verbosity")]
    #[arg(short = 'q', long = "quiet")]
    #[arg(action=ArgAction::Count)]
    pub(crate) quiet: u8,

    #[arg(help = "Increase verbosity")]
    #[arg(short = 'v', long = "verbose")]
    #[arg(action=ArgAction::Count)]
    pub(crate) verbose: u8,

    #[arg(short = 't', long = "table")]
    #[arg(help = "Table or views to query. Can be used multiple times.")]
    #[arg(action=ArgAction::Set)]
    pub(crate) table: Option<Vec<String>>,

    #[arg(help = "Pattern to match every cell with")]
    pub(crate) pattern: String,

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
