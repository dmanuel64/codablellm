use std::{collections::HashSet, mem, path::PathBuf};

use indicatif::ProgressBar;
use strum::{Display, EnumCount};
use thiserror::Error;

use crate::{builder, extractor, repo};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Repository(#[from] repo::Error),
    #[error(transparent)]
    Extractor(#[from] extractor::Error),
    #[error(transparent)]
    Builder(#[from] builder::Error),
    #[error("cannot transition from stage \"{stage}\" via event \"{event}\"")]
    InvalidTransition { stage: Stage, event: Event },
}

struct Manager {
    source: repo::Source,
    mode: Mode,
    stage: Stage,
    progress: ProgressBar,
}

impl Manager {
    pub fn new(source: repo::Source, mode: Mode, display_progress: bool) -> Self {
        let progress = if display_progress {
            ProgressBar::new(if let Mode::SourceOnly = mode {
                3
            } else {
                Stage::COUNT as u64 - 1
            })
        } else {
            ProgressBar::hidden()
        };

        Self {
            source,
            mode,
            stage: Stage::default(),
            progress,
        }
    }

    fn step(&self) -> Result<Event, Error> {
        match self.stage {
            Stage::Pulling => {
                self.progress.set_message("Pulling repo...");
                // repo::pull(&self.repo_url)?;
                Ok(Event::RepoPulled)
            }
            Stage::ExtractSourceCode => {
                self.progress.set_message("Extracting source code...");
                Ok(Event::SourceCodeExtracted)
            }
            Stage::SetupBuilder => {
                self.progress.set_message("Setting up builder...");
                Ok(Event::BuilderSetup)
            }
            Stage::BuildCode => {
                self.progress.set_message("Building repository...");
                Ok(Event::CodeBuilt)
            }
            Stage::DecompileBinaries => {
                self.progress.set_message("Decompiling binaries...");
                Ok(Event::BinariesDecompiled)
            }
            Stage::MapCode => {
                self.progress
                    .set_message("Mapping decompiled/source code...");
                Ok(Event::CodeMapped)
            }
            Stage::CreateDataset => {
                self.progress.set_message("Creating dataset...");
                Ok(Event::DatasetCreated)
            }
            Stage::Complete => todo!(),
        }
    }

    /// Some docstring
    pub fn run(&mut self) -> Result<PathBuf, Error> {
        // A comment
        loop {
            if let Stage::Complete = self.stage {
                break;
            }
            let event = self.step()?;
            if let Event::SourceCodeExtracted = event
                && let Mode::SourceOnly = self.mode
            {
                // Skip to create dataset stage
                self.stage = Stage::CreateDataset
            } else {
                self.stage.transition(event)?;
            }
            self.progress.inc(1);
        }
        todo!("path to dataset")
    }
}

#[derive(Debug, Copy, Clone, Display, EnumCount)]
pub enum Stage {
    Pulling,
    ExtractSourceCode,
    SetupBuilder,
    BuildCode,
    DecompileBinaries,
    MapCode,
    CreateDataset,
    Complete,
}

impl Stage {
    pub fn start() -> Stage {
        Self::Pulling
    }

    pub fn transition(&mut self, event: Event) -> Result<(), Error> {
        let next = match (&*self, event) {
            (Self::Pulling, Event::RepoPulled) => Self::SetupBuilder,
            (Self::SetupBuilder, Event::BuilderSetup) => Self::ExtractSourceCode,
            (Self::ExtractSourceCode, Event::SourceCodeExtracted) => Self::BuildCode,
            (Self::BuildCode, Event::CodeBuilt) => Self::DecompileBinaries,
            (Self::DecompileBinaries, Event::BinariesDecompiled) => Self::MapCode,
            (Self::MapCode, Event::CodeMapped) => Self::CreateDataset,
            (Self::CreateDataset, Event::DatasetCreated) => Self::Complete,
            (stage, event) => {
                return Err(Error::InvalidTransition {
                    stage: *stage,
                    event,
                });
            }
        };
        *self = next;
        Ok(())
    }
}

impl Default for Stage {
    fn default() -> Self {
        Self::start()
    }
}

#[derive(Debug, Display)]
pub enum Event {
    RepoPulled,
    BuilderSetup,
    SourceCodeExtracted,
    CodeBuilt,
    BinariesDecompiled,
    CodeMapped,
    DatasetCreated,
}

#[derive(Debug)]
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
    pub repo_options: repo::Options,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            display_progress: Default::default(),
            repo_options: repo::Options::default(),
        }
    }
}

pub fn run(source: repo::Source, mode: Mode) -> Result<PathBuf, Error> {
    run_with_options(source, mode, &Options::default())
}

pub fn run_with_options(
    source: repo::Source,
    mode: Mode,
    options: &Options,
) -> Result<PathBuf, Error> {
    Manager::new(source, mode, options.display_progress).run()
}
