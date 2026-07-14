use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Commands {
    #[command(visible_alias = "ls")]
    List,
    Show {
        name: String,
    },
    #[command(visible_alias = "refetch")]
    Update {
        name: String,
    },
    #[command(visible_alias = "rm")]
    Remove {
        name: String,
        #[arg(long, exclusive = true)]
        all: bool,
    },
    Prune,
    Path {
        name: String,
    },
}
