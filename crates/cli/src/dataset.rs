use std::{num::NonZeroUsize, path::PathBuf, str::FromStr};

use clap::{Args, ValueEnum};
use codablellm::{DatasetKind, FileSource, Language, dataset::Script};
use color_eyre::eyre::Result;
use inquire::{required, validator::Validation};
use strum::Display;

use crate::{
    config,
    resolver::{IntoResolved, require_or_prompt},
};

const NETWORK_OPTS_HEADING: &str = "Network Options";
const REPOSITORY_OPTS_HEADING: &str = "Repository Options";
const EXTRACTOR_OPTS_HEADING: &str = "Extractor Options";
const BUILDER_OPTS_HEADING: &str = "Builder Options";
const DECOMPILER_OPTS_HEADING: &str = "Decompiler Options";
const DATASET_OPTS_HEADING: &str = "Dataset Options";

#[derive(Debug, Args)]
pub struct CreateDatasetArgs {
    /// Name of the dataset being created
    name: Option<String>,

    /// The path or url to the repository
    repo: Option<FileSource>,

    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long, default_value_t = ForgeChoice::Auto)]
    forge: ForgeChoice,

    #[arg(help_heading = NETWORK_OPTS_HEADING, long)]
    token: Option<String>,

    #[arg(help_heading = NETWORK_OPTS_HEADING, long)]
    insecure: bool,

    #[arg(help_heading = NETWORK_OPTS_HEADING, long, conflicts_with = "insecure")]
    ca_cert: Option<PathBuf>,

    #[arg(
        help_heading = BUILDER_OPTS_HEADING,
        short,
        long,
        num_args = 0..,
        value_delimiter = ',',
        alias = "binary",
        requires = "build_commands",
    )]
    binaries: Vec<PathBuf>,

    /// Commands used to build the repository (repeatable)
    #[arg(
        help_heading = BUILDER_OPTS_HEADING,
        short = 'c',
        long = "build-command",
        value_name = "COMMAND",
        requires = "binaries"
    )]
    build_commands: Vec<String>,

    /// Decompilers to use (repeatable)
    #[arg(
        help_heading = DECOMPILER_OPTS_HEADING,
        long,
        value_delimiter = ',',
        alias = "decompiler",
        requires = "binaries",
    )]
    decompilers: Vec<String>,

    /// Strip symbols from the binaries
    #[arg(help_heading = BUILDER_OPTS_HEADING, long, requires = "binaries")]
    strip: bool,

    /// Strip symbols from the binaries
    #[arg(
        help_heading = DATASET_OPTS_HEADING,
        long,
        value_delimiter = ',',
        visible_alias = "reprs",
        aliases = ["representation", "repr"],
        default_values_t = vec![RepresentationChoice::Source, RepresentationChoice::Decompiled, RepresentationChoice::Assembly],
        requires = "binaries"
    )]
    representations: Vec<RepresentationChoice>,

    #[arg(short, long, visible_alias = "overwrite")]
    force: bool,

    #[arg(short, long, visible_alias = "out")]
    output: Option<PathBuf>,

    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long = "ref", visible_aliases = ["rev", "branch", "tag"])]
    git_ref: Option<String>,

    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long = "owner")]
    repo_owner: Option<String>,

    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long = "name")]
    repo_name: Option<String>,

    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long = "rm")]
    remove: bool,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, value_delimiter = ',')]
    include: Vec<PathBuf>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, value_delimiter = ',')]
    exclude: Vec<PathBuf>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, aliases = ["language", "lang"], visible_alias = "langs", default_values_t = config::get().languages.include)]
    languages: Vec<Language>,

    #[arg(short, long)]
    jobs: Option<NonZeroUsize>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long)]
    extractor_jobs: Option<NonZeroUsize>,

    #[arg(help_heading = DECOMPILER_OPTS_HEADING, long)]
    decompiler_jobs: Option<NonZeroUsize>,

    #[arg(help_heading = DATASET_OPTS_HEADING, long)]
    scripts: Vec<PathBuf>,

    #[arg(help_heading = DATASET_OPTS_HEADING, long, default_value_t = true)]
    paired: bool,
}

impl IntoResolved for CreateDatasetArgs {
    type Resolved = ResolvedCreateDatasetArgs;

    fn into_resolved(self, interactive: bool) -> Result<Self::Resolved> {
        let Self {
            name,
            repo,
            binaries,
            build_commands,
            decompilers,
            strip,
            forge,
            token,
            insecure,
            ca_cert,
            representations,
            force,
            output,
            git_ref,
            repo_owner,
            repo_name,
            include,
            exclude,
            languages,
            jobs,
            extractor_jobs,
            decompiler_jobs,
            scripts,
            paired,
            remove,
        } = self;
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

#[derive(Debug, Clone, Copy, ValueEnum, Display)]
#[strum(serialize_all = "lowercase")]
#[clap(rename_all = "lowercase")]
pub enum ForgeChoice {
    Auto,
    GitHub,
    GitLab,
    Gitea,
    Other,
}

#[derive(Debug, Clone, Copy, ValueEnum, Display)]
#[strum(serialize_all = "lowercase")]
#[clap(rename_all = "lowercase")]
pub enum RepresentationChoice {
    Source,
    Decompiled,
    Assembly,
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
