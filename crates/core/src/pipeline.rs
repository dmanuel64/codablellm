use std::{collections::HashSet, path::PathBuf};

use indicatif::ProgressBar;
use strum::{Display, EnumCount};

use crate::repo;

struct Manager {
    repo_url: String,
    mode: Mode,
    stage: Stage,
    progress: ProgressBar,
}

impl Manager {
    pub fn new(repo_url: String, mode: Mode, display_progress: bool) -> Self {
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
            repo_url,
            mode,
            stage: Stage::default(),
            progress,
        }
    }

    fn step(&self) -> Result<Event, crate::Error> {
        match self.stage {
            Stage::Pulling => {
                self.progress.set_message("Pulling repo...");
                repo::pull(&self.repo_url)?;
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

    pub fn run(&mut self) -> Result<PathBuf, crate::Error> {
        loop {
            if let Stage::Complete = self.stage {
                break;
            }
            let event = self.step()?;
            self.stage = if let Event::SourceCodeExtracted = event
                && let Mode::SourceOnly = self.mode
            {
                // Skip to create dataset stage
                Stage::CreateDataset
            } else {
                self.stage.transition(&event)?
            };
            self.progress.inc(1);
        }
        todo!("path to dataset")
    }
}

#[derive(Debug, Display, EnumCount)]
enum Stage {
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

    pub fn transition(&self, event: &Event) -> Result<Self, crate::Error> {
        match (self, event) {
            (Self::Pulling, Event::RepoPulled) => Ok(Self::SetupBuilder),
            (Self::SetupBuilder, Event::BuilderSetup) => Ok(Self::ExtractSourceCode),
            (Self::ExtractSourceCode, Event::SourceCodeExtracted) => Ok(Self::BuildCode),
            (Self::BuildCode, Event::CodeBuilt) => Ok(Self::DecompileBinaries),
            (Self::DecompileBinaries, Event::BinariesDecompiled) => Ok(Self::MapCode),
            (Self::MapCode, Event::CodeMapped) => Ok(Self::CreateDataset),
            (Self::CreateDataset, Event::DatasetCreated) => Ok(Self::Complete),
            (stage, event) => Err(crate::Error::InvalidTransition {
                stage: stage.to_string(),
                event: event.to_string(),
            }),
        }
    }
}

impl Default for Stage {
    fn default() -> Self {
        Self::start()
    }
}

#[derive(Debug, Display)]
enum Event {
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

pub fn run(repo_url: String, mode: Mode) -> Result<PathBuf, crate::Error> {
    run_with_options(repo_url, mode, &Options::default())
}

pub fn run_with_options(
    repo_url: String,
    mode: Mode,
    options: &Options,
) -> Result<PathBuf, crate::Error> {
    Manager::new(repo_url, mode, options.display_progress).run()
}
