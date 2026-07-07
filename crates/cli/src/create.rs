use clap::{Args, Subcommand};
use codablellm::FileSource;
use color_eyre::eyre::{Report, Result};
use std::path::PathBuf;

#[derive(Debug, Args)]
struct CreateDatasetArgs {
    /// The path or url to the repository
    repo: FileSource,
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

fn create_source_dataset(repo: FileSource, name: Option<String>) -> Result<PathBuf> {
    let dataset = codablellm::run(repo, codablellm::Mode::SourceOnly)?;
    todo!()
}

fn create_binary_dataset(repo: FileSource, name: Option<String>) -> Result<PathBuf> {
    let dataset = codablellm::run(repo, codablellm::Mode::SourceOnly)?;
    todo!()
}
