use indoc::indoc;

#[derive(clap::Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    #[arg(help="Increase verbosity")]
    #[arg(short='q', long="quiet")]
    #[arg(action=clap::ArgAction::Count)]
    pub(crate) quiet: u8,

    #[arg(help="Increase verbosity")]
    #[arg(short='v', long="verbose")]
    #[arg(action=clap::ArgAction::Count)]
    pub(crate) verbose: u8,

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
