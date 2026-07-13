use std::{
    num::NonZeroUsize,
    path::PathBuf,
    str::FromStr,
    sync::{LazyLock, OnceLock},
};

use clap::{Args, ValueEnum};
use codablellm::{DatasetKind, FileSource, Language, dataset::Script, repo};
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

fn inferred_forge(val: &str) -> &'static Result<Option<ForgeChoice>> {
    static FORGE_CHOICE: OnceLock<Result<Option<ForgeChoice>>> = OnceLock::new();

    let choice = FORGE_CHOICE.get_or_init(|| {
        if let FileSource::Remote(f) = FileSource::from_str(val)? {
            let url = f.url;
            let choice = if let Some(_) = repo::Metadata::from_gitlab(&url) {
                ForgeChoice::GitHub
            } else if let Some(_) = repo::Metadata::from_gitlab(&url) {
                ForgeChoice::GitLab
            } else if let Some(_) = repo::Metadata::from_gitea(&url) {
                ForgeChoice::Gitea
            } else {
                ForgeChoice::Other
            };
            Ok(Some(choice))
        } else {
            Ok(None)
        }
    });
    choice
}
#[derive(Debug, Args)]
pub struct CreateDatasetArgs {
    /// Name of the dataset being created
    name: Option<String>,

    /// The path or url to the repository
    repo: Option<FileSource>,

    /// The type of the code forge if REPO is a URL
    ///
    /// If this option is not specified, it will be automatically determined from the URL
    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long)]
    forge: Option<ForgeChoice>,

    /// An authorization token to access the remote forge
    #[arg(help_heading = NETWORK_OPTS_HEADING, long)]
    token: Option<String>,

    /// Skip TLS authorization for fetching the repository from the remote forge
    #[arg(help_heading = NETWORK_OPTS_HEADING, long)]
    insecure: bool,

    /// Local path to the certificate authority for the remote forge
    #[arg(help_heading = NETWORK_OPTS_HEADING, long, conflicts_with = "insecure")]
    ca_cert: Option<PathBuf>,

    /// Path(s) to the built binaries of the repository
    ///
    /// Specifying this option will create a compiled dataset mapping source code to compiled code.
    /// The paths can be either absolute or relative. --build-commands must be used with this option.
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

    /// Commands used to build the repository
    #[arg(
        help_heading = BUILDER_OPTS_HEADING,
        short = 'c',
        long = "build-command",
        value_name = "COMMAND",
        requires = "binaries"
    )]
    build_commands: Vec<String>,

    /// Decompilers to use
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

    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long = "ref", visible_aliases = ["rev", "branch", "tag"], required_if_eq("forge", "other"))]
    git_ref: Option<String>,

    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long = "owner", required_if_eq("forge", "other"))]
    repo_owner: Option<String>,

    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long = "name", required_if_eq("forge", "other"))]
    repo_name: Option<String>,

    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long = "remove", visible_alias = "rm")]
    remove: bool,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, value_delimiter = ',')]
    include: Vec<PathBuf>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, value_delimiter = ',')]
    exclude: Vec<PathBuf>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, aliases = ["language", "lang"], visible_alias = "langs", default_values_t = config::get().languages.include)]
    languages: Vec<Language>,

    #[arg(short, long)]
    jobs: Option<NonZeroUsize>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, requires = "binaries")]
    extractor_jobs: Option<NonZeroUsize>,

    #[arg(help_heading = DECOMPILER_OPTS_HEADING, long, requires = "binaries")]
    decompiler_jobs: Option<NonZeroUsize>,

    #[arg(help_heading = DATASET_OPTS_HEADING, long)]
    scripts: Vec<PathBuf>,

    #[arg(help_heading = DATASET_OPTS_HEADING, long)]
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
        let mut prompted = false;
        let name = require_or_prompt(name, "NAME", interactive, || {
            prompted = true;
            Ok(
                inquire::Text::new("Enter the name of the dataset to create")
                    .with_validator(required!())
                    .prompt()?,
            )
        })?;
        let repo = require_or_prompt(repo, "REPO", interactive, || {
            prompted = true;
            Ok(
                inquire::CustomType::new("Enter the path or URL to the repository")
                    .with_help_message("URLs should be to a repository's git, tarball, or zipfile")
                    .with_error_message("Please enter a valid path or URL")
                    .prompt()?,
            )
        })?;
        let is_compiled_dataset = !binaries.is_empty()
            || !build_commands.is_empty()
            || prompted
                .then(|| {
                    inquire::Confirm::new("Is this a compiled code dataset?")
                        .with_default(false)
                        .prompt()
                })
                .unwrap_or(Ok(false))?;
        if is_compiled_dataset {
            // let binaries = require_or_prompt(binaries, "BINARIES", interactive, prompt)
        }
        Ok(ResolvedCreateDatasetArgs { name, repo })
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Display)]
#[strum(serialize_all = "lowercase")]
#[clap(rename_all = "lowercase")]
pub enum ForgeChoice {
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
