use std::{num::NonZeroUsize, path::PathBuf, str::FromStr, sync::OnceLock};

use clap::{Args, ValueEnum};
use codablellm_core::{
    BinaryMode, Dataset, Language, Location, Mode, Options, RepoMetadata, RepoSource, dataset,
    decompiler, extractor, mapper, repo,
};
use color_eyre::eyre::Result;
use inquire::required;
use strum::Display;

use crate::{
    config,
    errors::user_error,
    resolver::{IntoResolved, require_or_prompt},
};

const NETWORK_OPTS_HEADING: &str = "Network Options";
const REPOSITORY_OPTS_HEADING: &str = "Repository Options";
const EXTRACTOR_OPTS_HEADING: &str = "Extractor Options";
const BUILDER_OPTS_HEADING: &str = "Builder Options";
const DECOMPILER_OPTS_HEADING: &str = "Decompiler Options";
const DATASET_OPTS_HEADING: &str = "Dataset Options";

fn infer_metadata(val: &str) -> Result<Option<RepoMetadata>> {
    if let Location::Url(url) = Location::from_str(val)? {
        let metadata = if let Some(m) = RepoMetadata::from_gitlab(&url) {
            Some(m)
        } else if let Some(m) = RepoMetadata::from_gitlab(&url) {
            Some(m)
        } else if let Some(m) = RepoMetadata::from_gitea(&url) {
            Some(m)
        } else {
            None
        };
        Ok(metadata)
    } else {
        Ok(None)
    }
}

#[derive(Debug, Args)]
pub struct CreateDatasetArgs {
    /// Name of the dataset being created
    name: Option<String>,

    /// The path or url to the repository
    repo: Option<Location>,

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

    #[arg(help_heading = REPOSITORY_OPTS_HEADING, long = "ref", visible_aliases = ["rev", "branch", "tag"])]
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

    #[arg(help_heading = DATASET_OPTS_HEADING, long, default_value_t = FormatChoice::Csv)]
    format: FormatChoice,

    #[arg(long, hide = true)]
    dry_run: bool,
}

impl IntoResolved for CreateDatasetArgs {
    type Resolved = ResolvedCreateDatasetArgs;

    fn into_resolved(self, interactive: bool) -> Result<Self::Resolved> {
        let Self {
            name,
            repo,
            mut binaries,
            mut build_commands,
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
            dry_run,
            remove,
            format,
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
            || (prompted
                && inquire::Confirm::new("Is this a compiled code dataset?")
                    .with_default(false)
                    .prompt()?);
        if is_compiled_dataset {
            binaries = require_or_prompt(
                if !binaries.is_empty() {
                    Some(binaries.clone())
                } else {
                    None
                },
                "--binaries",
                interactive,
                || {
                    let mut bins = Vec::new();
                    loop {
                        let path = PathBuf::from(inquire::Text::new("Enter the path of where the compiled binary will be")
                        .with_validator(required!())
                        .with_help_message("Paths may be absolute or relative to the working directory where the build command is executed")
                        .prompt()?);
                        bins.push(path);
                        let more_bins =
                            inquire::Confirm::new("Do you have more paths to enter?").prompt()?;
                        if !more_bins {
                            break;
                        }
                    }
                    Ok(bins)
                },
            )?;
            build_commands = require_or_prompt(
                if !build_commands.is_empty() {
                    Some(build_commands.clone())
                } else {
                    None
                },
                "--build-commands",
                interactive,
                || {
                    let mut cmds = Vec::new();
                    loop {
                        let cmd = inquire::Text::new("Enter a build command")
                            .with_validator(required!())
                            .with_help_message(
                                "Build commands will be executed in the order they are entered",
                            )
                            .prompt()?;
                        cmds.push(cmd);
                        let more_cmds = inquire::Confirm::new(
                            "Do you have additional build commands to enter?",
                        )
                        .prompt()?;
                        if !more_cmds {
                            break;
                        }
                    }
                    Ok(cmds)
                },
            )?;
        }
        Ok(ResolvedCreateDatasetArgs {
            name,
            repo,
            binaries,
            build_commands,
            dry_run,
            forge,
            token,
            insecure,
            ca_cert,
            decompilers,
            strip,
            representations,
            force,
            output,
            git_ref,
            repo_owner,
            repo_name,
            remove,
            include,
            exclude,
            languages,
            jobs,
            extractor_jobs,
            decompiler_jobs,
            scripts,
            paired,
            format,
        })
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Display)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[clap(rename_all = "lowercase")]
pub enum ForgeChoice {
    GitHub,
    GitLab,
    Gitea,
    Other,
}

#[derive(Debug, Clone, Copy, ValueEnum, Display)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[clap(rename_all = "lowercase")]
pub enum FormatChoice {
    Csv,
    Parquet,
    Jsonl,
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
    repo: Location,
    forge: Option<ForgeChoice>,
    token: Option<String>,
    insecure: bool,
    ca_cert: Option<PathBuf>,
    binaries: Vec<PathBuf>,
    build_commands: Vec<String>,
    decompilers: Vec<String>,
    strip: bool,
    representations: Vec<RepresentationChoice>,
    force: bool,
    output: Option<PathBuf>,
    git_ref: Option<String>,
    repo_owner: Option<String>,
    repo_name: Option<String>,
    remove: bool,
    include: Vec<PathBuf>,
    exclude: Vec<PathBuf>,
    languages: Vec<Language>,
    jobs: Option<NonZeroUsize>,
    extractor_jobs: Option<NonZeroUsize>,
    decompiler_jobs: Option<NonZeroUsize>,
    scripts: Vec<PathBuf>,
    paired: bool,
    format: FormatChoice,
    dry_run: bool,
}

pub async fn create_dataset(
    ResolvedCreateDatasetArgs {
        binaries,
        build_commands,
        name,
        repo,
        dry_run,
        forge,
        token,
        insecure,
        ca_cert,
        decompilers,
        strip,
        representations,
        force,
        output,
        git_ref,
        repo_owner,
        repo_name,
        remove,
        include,
        exclude,
        languages,
        jobs,
        extractor_jobs,
        decompiler_jobs,
        scripts,
        paired,
        format,
    }: ResolvedCreateDatasetArgs,
) -> Result<Dataset> {
    let cfg = config::get();
    let display_progress = cfg.display.progress;
    let mode = if binaries.is_empty() {
        Mode::SourceOnly
    } else {
        Mode::SourceAndBinary(BinaryMode {
            binaries: binaries,
            build_commands,
            strip: false,
            decompilers: vec!["Ghidra"].iter().map(ToString::to_string).collect(),
        })
    };
    let metadata = if let (Some(owner), Some(name), git_ref) = (repo_owner, repo_name, git_ref) {
        RepoMetadata {
            owner,
            name,
            git_ref,
        }
    } else if let Some(m) = infer_metadata(&repo.to_string())? {
        m
    } else {
        return Err(user_error("--owner <REPO_OWNER> and --name <REPO_NAME> are required when using local repositories or non-standard forges").into());
    };
    let source = RepoSource {
        metadata,
        location: repo,
        kind: todo!(),
    };

    let dataset = codablellm_core::run_with_options(
        source,
        mode,
        Options {
            display_progress,
            dry_run,
            repo_options: repo::Options {
                display_progress,
                ..Default::default()
            },
            extractor_options: extractor::Options { display_progress },
            decompiler_options: decompiler::Options { display_progress },
            mapper_options: mapper::Options { display_progress },
            dataset_options: dataset::Options { display_progress },
        },
    )
    .await?;
    Ok(dataset)
}
