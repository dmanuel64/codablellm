use std::{collections::HashSet, mem, path::PathBuf};

use indicatif::ProgressBar;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use thiserror::Error;
use tracing::instrument;

use crate::{
    FileSource, builder,
    dataset::{self, Dataset},
    decompiler, extractor,
    function::Function,
    language, mapper,
    repo::{self, Repository},
    storage,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Repository(#[from] repo::Error),
    #[error(transparent)]
    Extractor(#[from] extractor::Error),
    #[error(transparent)]
    Builder(#[from] builder::Error),
    #[error(transparent)]
    Storage(#[from] storage::Error),
}

struct Manager {
    source: repo::Source,
    mode: Mode,
    progress: ProgressBar,
    options: Options,
    repo: Option<Repository>,
    extracted_functions: Vec<Function>,
}

impl Manager {
    pub fn new(source: repo::Source, mode: Mode, options: Options) -> Self {
        let progress = if options.display_progress {
            ProgressBar::no_length()
        } else {
            ProgressBar::hidden()
        };

        Self {
            source,
            mode,
            progress,
            options,
            repo: None,
            extracted_functions: Vec::new(),
        }
    }

    /// Some docstring
    pub async fn run(&mut self) -> Result<dataset::Kind, Error> {
        let stages = if matches!(self.mode, Mode::SourceOnly) {
            tracing::info!("Starting pipeline for source code dataset");
            Box::new(Stage::iter_source_stages()) as Box<dyn Iterator<Item = Stage>>
        } else {
            tracing::info!("Starting pipeline for compiled dataset");
            Box::new(Stage::iter())
        };
        for stage in self.progress.wrap_iter(stages) {
            tracing::info!(
                %stage,
                "Executing pipeline stage"
            );
            self.process(&stage).await?;
        }
        todo!("path to dataset")
    }

    async fn process(&mut self, stage: &Stage) -> Result<(), Error> {
        match stage {
            Stage::Pulling => {
                self.repo = Some(repo::fetch_with_options(
                    self.source.clone(),
                    &self.options.repo_options,
                )?);
            }
            Stage::ExtractSourceCode => {
                self.extracted_functions = extractor::extract_with_options(
                    self.repo.as_ref().expect("repo to have been pulled"),
                    &self.options.extractor_options,
                )?;
            }
            Stage::SetupBuilder => {
                let bin_mode = self.mode.as_binary_mode();
                let artifacts: Vec<_> = bin_mode.binaries.iter().map(PathBuf::as_path).collect();
                let commands: Vec<_> = bin_mode.build_commands.iter().map(String::as_str).collect();
                builder::build(
                    &self.repo.as_ref().expect("repo to have been pulled"),
                    &commands,
                    &artifacts,
                )
                .await?;
            }
            Stage::BuildCode => todo!(),
            Stage::DecompileBinaries => todo!(),
            Stage::MapCode => todo!(),
            Stage::CreateDataset => todo!(),
        };
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Display, Default, EnumCount, EnumIter)]
pub enum Stage {
    #[default]
    Pulling,
    ExtractSourceCode,
    SetupBuilder,
    BuildCode,
    DecompileBinaries,
    MapCode,
    CreateDataset,
}

impl Stage {
    pub fn is_binary_stage(&self) -> bool {
        matches!(
            self,
            Self::SetupBuilder | Self::BuildCode | Self::DecompileBinaries | Self::MapCode
        )
    }

    pub fn iter_source_stages() -> impl Iterator<Item = Stage> {
        Self::iter().filter(|s| !s.is_binary_stage())
    }
}

#[derive(Debug, Clone)]
pub struct BinaryMode {
    pub build_commands: Vec<String>,
    pub binaries: Vec<PathBuf>,
    pub strip: bool,
    pub decompilers: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum Mode {
    SourceOnly,
    SourceAndBinary(BinaryMode),
}

impl Mode {
    pub fn as_binary_mode(&self) -> BinaryMode {
        let Mode::SourceAndBinary(mode) = self else {
            unreachable!("expected SourceAndBinary mode during a binary stage")
        };
        mode.clone()
    }
}

#[derive(Debug)]
pub struct Options {
    pub display_progress: bool,
    pub dry_run: bool,
    pub repo_options: repo::Options,
    pub extractor_options: extractor::Options,
    pub decompiler_options: decompiler::Options,
    pub mapper_options: mapper::Options,
    pub dataset_options: dataset::Options,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            display_progress: true,
            dry_run: false,
            repo_options: repo::Options::default(),
            extractor_options: extractor::Options::default(),
            decompiler_options: decompiler::Options::default(),
            mapper_options: mapper::Options::default(),
            dataset_options: dataset::Options::default(),
        }
    }
}

pub async fn run(source: repo::Source, mode: Mode) -> Result<dataset::Kind, Error> {
    run_with_options(source, mode, Options::default()).await
}

#[instrument(name = "pipeline", skip(options))]
pub async fn run_with_options(
    source: repo::Source,
    mode: Mode,
    options: Options,
) -> Result<dataset::Kind, Error> {
    tracing::trace!(?options);
    Manager::new(source, mode, options).run().await
}
