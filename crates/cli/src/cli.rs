use clap::{Parser, Subcommand};
use clap_verbosity_flag::WarnLevel;
use std::{
    io::{self, IsTerminal},
    path::PathBuf,
};

pub(crate) fn is_interactive() -> bool {
    io::stdout().is_terminal()
}

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    long_version = "lots of stuff",
    about = "foo",
    long_about = "foobar"
)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub(crate) command: Option<Commands>,
    #[arg(required = !is_interactive())]
    pub(crate) repo: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) verbosity: clap_verbosity_flag::Verbosity<WarnLevel>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Completion,
}
