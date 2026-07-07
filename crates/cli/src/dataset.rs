use std::str::FromStr;

use clap::Args;
use codablellm::{DatasetKind, FileSource};
use color_eyre::eyre::Result;
use inquire::{required, validator::Validation};

use crate::resolver::{IntoResolved, require_or_prompt};

#[derive(Debug, Args)]
pub struct CreateDatasetArgs {
    /// Name of the dataset being created
    name: Option<String>,
    /// The path or url to the repository
    repo: Option<FileSource>,
}

impl IntoResolved for CreateDatasetArgs {
    type Resolved = ResolvedCreateDatasetArgs;

    fn into_resolved(self, interactive: bool) -> Result<Self::Resolved> {
        let Self { name, repo } = self;
        let name = require_or_prompt(name, "NAME", interactive, || {
            Ok(
                inquire::Text::new("Enter the name of the dataset to create")
                    .with_validator(required!())
                    .prompt()?,
            )
        })?;
        let repo = require_or_prompt(repo, "REPO", interactive, || {
            Ok(
                inquire::CustomType::new("Enter the path or URL to the repository")
                    .with_help_message("URLs should be to a repository's git, tarball, or zipfile")
                    .with_error_message("Please enter a valid path or URL")
                    .prompt()?,
            )
        })?;
        Ok(ResolvedCreateDatasetArgs { name, repo })
    }
}

#[derive(Debug)]
pub struct ResolvedCreateDatasetArgs {
    name: String,
    repo: FileSource,
}

pub fn create_source_dataset(
    ResolvedCreateDatasetArgs { name, repo }: ResolvedCreateDatasetArgs,
) -> Result<DatasetKind> {
    todo!()
}
