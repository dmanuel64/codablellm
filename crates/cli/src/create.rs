use clap::{Args, Subcommand};
use codablellm::RepoSource;
use color_eyre::eyre::{Report, Result};
use std::path::PathBuf;

#[derive(Debug, Args)]
struct CreateDatasetArgs {
    /// The path or url to the repository
    repo: RepoSource,
}

#[derive(Debug, Args)]
struct CreateDatasetOptArgs {
    /// Name of the dataset being created
    name: Option<String>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Only extract source
    SourceDataset {
        #[clap(flatten)]
        create_dataset_args: CreateDatasetArgs,
        #[clap(flatten)]
        create_dataset_opt_args: CreateDatasetOptArgs,
    },
    /// Extract source and binary
    BinaryDataset {
        #[clap(flatten)]
        create_dataset_args: CreateDatasetArgs,
        #[clap(flatten)]
        create_dataset_opt_args: CreateDatasetOptArgs,
    },
    Script {
        name: String,
    },
}

#[derive(Debug, Args)]
pub struct Command {
    #[clap(subcommand)]
    command: Commands,
}

pub fn run(command: Command) -> Result<()> {
    let dataset_path = match command.command {
        Commands::SourceDataset {
            create_dataset_args: CreateDatasetArgs { repo },
            create_dataset_opt_args: CreateDatasetOptArgs { name },
        } => create_source_dataset(repo, name),
        Commands::BinaryDataset {
            create_dataset_args: CreateDatasetArgs { repo },
            create_dataset_opt_args: CreateDatasetOptArgs { name },
        } => create_binary_dataset(repo, name),
        Commands::Script { name } => todo!(),
    }?;
    Ok(())
}

fn create_source_dataset(repo: RepoSource, name: Option<String>) -> Result<PathBuf> {
    codablellm::run(repo, codablellm::Mode::SourceOnly).map_err(Report::from)
}

fn create_binary_dataset(repo: RepoSource, name: Option<String>) -> Result<PathBuf> {
    codablellm::run(repo, codablellm::Mode::SourceOnly).map_err(Report::from)
}
