use clap::{Parser, Subcommand, ValueEnum};
use clap_verbosity_flag::WarnLevel;
use std::{
    io::{self, IsTerminal},
    num::NonZero,
    path::PathBuf,
    sync::LazyLock,
};
use strum::{Display, EnumIter};

pub(crate) static INTERACTIVE: LazyLock<bool> = LazyLock::new(|| io::stdout().is_terminal());

#[derive(Debug, Parser)]
#[command(
    author,
    version,
    long_version = "lots of stuff",
    about = "foo",
    long_about = "foobar"
)]
#[command(args_conflicts_with_subcommands = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub(crate) command: Option<Commands>,
    #[arg(required = !*INTERACTIVE)]
    pub(crate) repo: Option<PathBuf>,

    /// Extract by filtered source code languages
    ///
    /// Specify "-l"/"--languages" to automatically figure out the source code languages. In a non-interactive
    /// environment, this will default to "--languages=auto".
    #[arg(
        long = "languages",
        short = 'l',
        aliases = ["language", "lang"],
        visible_alias = "langs",
        ignore_case = true,
        require_equals = true,
        value_delimiter = ',',
        num_args = 0..=1,
        default_missing_value = "auto",
        default_value = if *INTERACTIVE { None } else { Some("auto") },
    )]
    pub(crate) source_languages: Option<Vec<SourceLanguage>>,

    /// Restrict source code extraction to the specified subdirectories.
    ///
    /// Can either be an absolute or relative path. Conflicts with --exclude.
    #[arg(
        long,
        conflicts_with = "exclude",
        require_equals = true,
        value_delimiter = ','
    )]
    pub(crate) restrict: Option<Vec<PathBuf>>,

    /// Exclude subdirectories from the source code extraction.
    ///
    /// Can either be an absolute or relative path. Conflicts with --restrict.
    #[arg(
        long,
        conflicts_with = "restrict",
        require_equals = true,
        value_delimiter = ','
    )]
    pub(crate) exclude: Option<Vec<PathBuf>>,

    #[arg(long)]
    pub(crate) max_extractor_jobs: Option<NonZero<usize>>,

    // TODO: need a sentinel value to represent when the user passes -o without a value (to ignore interactive prompting)
    #[arg(long, short)]
    pub(crate) out: Option<PathBuf>,

    #[command(flatten)]
    pub(crate) verbosity: clap_verbosity_flag::Verbosity<WarnLevel>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Output shell completion code for the specified shell (zsh, powershell, bash, elvish, or fish)
    Completion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, ValueEnum, Display, EnumIter)]
pub(crate) enum SourceLanguage {
    #[default]
    Auto,
    C,
}
