use std::{collections::HashSet, mem, path::PathBuf};

use indicatif::ProgressBar;
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use thiserror::Error;
use tracing::instrument;

use crate::{
    FileSource, builder,
    dataset::{self, Dataset},
    decompiler, extractor, language, mapper, repo, storage,
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
}

impl Manager {
    pub fn new(source: repo::Source, mode: Mode, display_progress: bool) -> Self {
        let progress = if display_progress {
            ProgressBar::no_length()
        } else {
            ProgressBar::hidden()
        };

        Self {
            source,
            mode,
            progress,
        }
    }

    /// Some docstring
    pub fn run(&self) -> Result<dataset::Kind, Error> {
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
            self.process(&stage)?;
        }
        todo!("path to dataset")
    }

    fn process(&self, stage: &Stage) -> Result<(), Error> {
        match stage {
            Stage::Pulling => todo!(),
            Stage::ExtractSourceCode => todo!(),
            Stage::SetupBuilder => todo!(),
            Stage::BuildCode => todo!(),
            Stage::DecompileBinaries => todo!(),
            Stage::MapCode => todo!(),
            Stage::CreateDataset => todo!(),
        }
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
pub enum Mode {
    SourceOnly,
    SourceAndBinary {
        build_commands: Vec<String>,
        strip: bool,
        decompilers: HashSet<String>,
    },
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

pub fn run(source: repo::Source, mode: Mode) -> Result<dataset::Kind, Error> {
    run_with_options(source, mode, &Options::default())
}

#[instrument(name = "pipeline", skip(options))]
pub fn run_with_options(
    source: repo::Source,
    mode: Mode,
    options: &Options,
) -> Result<dataset::Kind, Error> {
    tracing::trace!(?options);
    Manager::new(source, mode, options.display_progress).run()
}
