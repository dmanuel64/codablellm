use std::{
    collections::HashSet,
    default,
    path::{Path, PathBuf},
};

use crate::types::{decompiler::Decompiler, errors::PipelineError};

#[derive(Debug)]
pub enum Stage {
    Pulling,
    SetupContainer,
    ExtractSourceCode,
    BuildCode,
    DecompileBinaries,
    MapCode,
    CreateDataset,
    Complete,
    Error(String),
}

impl Stage {
    pub fn transition(self, event: Event) -> Result<Self, PipelineError> {
        match (self, event) {
            (Self::Pulling, Event::PulledRepo) => Ok(Self::SetupContainer),
            (Self::SetupContainer, Event::ContainerSetup) => Ok(Self::ExtractSourceCode),
            (Self::ExtractSourceCode, Event::SourceCodeExtracted) => Ok(Self::BuildCode),
            (Self::BuildCode, Event::CodeBuilt) => Ok(Self::DecompileBinaries),
            (Self::DecompileBinaries, Event::BinariesDecompiled) => Ok(Self::MapCode),
            (Self::MapCode, Event::CodeMapped) => Ok(Self::CreateDataset),
            (Self::CreateDataset, Event::DatasetCreated) => Ok(Self::Complete),
            (stage, event) => Err(PipelineError::InvalidTransition { stage, event }),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    PulledRepo,
    ContainerSetup,
    SourceCodeExtracted,
    CodeBuilt,
    BinariesDecompiled,
    CodeMapped,
    DatasetCreated,
    CriticalFailure(String),
}

#[derive(Debug, Default)]
pub enum Mode<'a> {
    #[default]
    SourceOnly,
    SourceAndBinary {
        build_scripts: &'a [&'a Path],
        strip: bool,
        decompilers: HashSet<Decompiler>,
    },
}

#[derive(Debug, Default)]
pub struct Options<'a> {
    pub repo_url: String,
    pub mode: Mode<'a>,
}
