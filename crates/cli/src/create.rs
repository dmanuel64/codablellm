use std::{num::NonZeroUsize, path::PathBuf, sync::OnceLock};

use clap::{Args, ValueEnum};
use codablellm_core::{
    BinaryMode, Dataset, Metadata, Mode, Options, Transform, dataset, decompiler, extractor, mapper,
};
use color_eyre::eyre::Result;
use inquire::required;
use strum::Display;

use crate::{
    config,
    errors::user_error,
    resolver::{IntoResolved, require_or_prompt},
    storage,
};

const EXTRACTOR_OPTS_HEADING: &str = "Extractor Options";
const BUILDER_OPTS_HEADING: &str = "Builder Options";
const DECOMPILER_OPTS_HEADING: &str = "Decompiler Options";
const DATASET_OPTS_HEADING: &str = "Dataset Options";

#[derive(Debug, Args)]
pub struct CreateDatasetArgs {
    /// Name of the dataset being created
    name: Option<String>,

    /// The path to the repository
    repo: Option<PathBuf>,

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
        aliases = ["binary", "bin"],
        visible_alias = "bins",
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

    #[arg(long)]
    discard_state: bool,

    #[arg(short, long, visible_alias = "overwrite")]
    force: bool,

    #[arg(short, long, visible_alias = "out")]
    output: Option<PathBuf>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, value_delimiter = ',')]
    include: Vec<PathBuf>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, value_delimiter = ',')]
    exclude: Vec<PathBuf>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, aliases = ["language", "lang"], visible_alias = "langs", default_values_t = config::get().languages.include)]
    languages: Vec<Metadata>,

    #[arg(short, long)]
    jobs: Option<NonZeroUsize>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, requires = "binaries")]
    extractor_jobs: Option<NonZeroUsize>,

    #[arg(help_heading = DECOMPILER_OPTS_HEADING, long, requires = "binaries")]
    decompiler_jobs: Option<NonZeroUsize>,

    #[arg(short, help_heading = EXTRACTOR_OPTS_HEADING, long, aliases = ["transform"])]
    transforms: Vec<PathBuf>,

    #[arg(help_heading = EXTRACTOR_OPTS_HEADING, long, requires = "transforms")]
    in_place: bool,

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
            representations,
            force,
            in_place,
            transforms,
            output,
            include,
            exclude,
            languages,
            jobs,
            extractor_jobs,
            decompiler_jobs,
            scripts,
            paired,
            dry_run,
            format,
            discard_state,
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
            Ok(inquire::Text::new("Enter the path to the repository")
                .with_validator(required!())
                .prompt()?
                .into())
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
            decompilers,
            strip,
            representations,
            force,
            output,
            include,
            exclude,
            languages,
            jobs,
            extractor_jobs,
            decompiler_jobs,
            scripts,
            paired,
            format,
            discard_state,
        })
    }
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
    repo: PathBuf,
    binaries: Vec<PathBuf>,
    build_commands: Vec<String>,
    decompilers: Vec<String>,
    strip: bool,
    representations: Vec<RepresentationChoice>,
    force: bool,
    output: Option<PathBuf>,
    include: Vec<PathBuf>,
    exclude: Vec<PathBuf>,
    languages: Vec<Metadata>,
    jobs: Option<NonZeroUsize>,
    extractor_jobs: Option<NonZeroUsize>,
    decompiler_jobs: Option<NonZeroUsize>,
    scripts: Vec<PathBuf>,
    paired: bool,
    format: FormatChoice,
    dry_run: bool,
    discard_state: bool,
}

pub async fn create_dataset(
    ResolvedCreateDatasetArgs {
        binaries,
        build_commands,
        name,
        repo,
        dry_run,
        decompilers,
        strip,
        representations,
        force,
        output,
        include,
        exclude,
        languages,
        jobs,
        extractor_jobs,
        decompiler_jobs,
        scripts,
        paired,
        format,
        discard_state,
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
    if let Some(output) = &output {
        if !force && output.exists() {
            return Err(user_error(format!(
                "{} already exists; pass --force/--overwrite to replace it",
                output.display()
            ))
            .into());
        }
    }
    let dataset = codablellm_core::run_with_options(
        repo,
        mode,
        &Options {
            display_progress,
            dry_run,
            cache_dir: Some(&storage::CACHE_DIR),
            state_dir: Some(&storage::STATE_DIR),
            ..Default::default()
        },
    )
    .await?;
    Ok(dataset)
}
