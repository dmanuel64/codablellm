use clap::Parser;
use clap_verbosity_flag::WarnLevel;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    // Make sure this is public
    pub repo: Option<PathBuf>,
    #[command(flatten)]
    pub verbosity: clap_verbosity_flag::Verbosity<WarnLevel>,
}
