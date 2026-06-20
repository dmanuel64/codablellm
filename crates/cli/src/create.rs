use std::path::PathBuf;

use clap::Args;
use color_eyre::eyre::Result;

#[derive(Debug, Args)]
pub struct Command {
    /// Name of the dataset being created
    dataset: String,
    /// The path to the local repository
    #[arg(value_name = "REPO", required_unless_present = "url")]
    repository_path: Option<PathBuf>,
    /// URL to download the remote repository from
    #[arg(long, conflicts_with = "repository_path")]
    url: Option<String>,
}

pub fn run(command: &Command) -> Result<()> {
    println!("{:#?}", command);
    Ok(())
}
