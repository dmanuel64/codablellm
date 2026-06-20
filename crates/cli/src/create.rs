use std::path::PathBuf;

use strum::IntoEnumIterator;
mod binary;

use clap::{Args, Subcommand};
use codablellm::{Language, RepoSource};
use color_eyre::eyre::{Report, Result};

#[derive(Debug, Subcommand)]
enum Commands {
    /// Only extract source
    Source,
    /// Extract source and binary
    Binary,
}

#[derive(Debug, Args)]
pub struct Command {
    /// Name of the dataset being created
    dataset: String,
    /// The path or url to the repository
    repo: RepoSource,
    /// Languages to only extract
    ///
    /// By default, codablellm will extract all possible languages.
    #[arg(long = "langs", alias = "lang", value_enum, value_delimiter = ',', default_values_t = Language::iter(), hide_default_value = true)]
    languages: Vec<Language>,
    #[clap(subcommand)]
    command: Commands,
}

pub fn run(command: Command) -> Result<()> {
    let dataset_path = match command.command {
        Commands::Source => create_source_dataset(command),
        Commands::Binary => todo!(),
    }?;
    Ok(())
}

fn create_source_dataset(command: Command) -> Result<PathBuf> {
    codablellm::run(command.repo, codablellm::Mode::SourceOnly).map_err(Report::from)
}
