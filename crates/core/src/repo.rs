use glob::glob;
use serde::{Deserialize, Serialize};
use std::{io, path::PathBuf};
use strum::IntoEnumIterator;
use thiserror::Error;

use crate::language::SourceLanguage;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repository {
    pub path: PathBuf,
    pub languages: Vec<SourceLanguage>,
}

impl Repository {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        Self::new_with_languages(path, SourceLanguage::iter().collect())
    }

    pub fn new_with_languages(
        path: PathBuf,
        languages: Vec<SourceLanguage>,
    ) -> Result<Self, Error> {
        if !path.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("{} is not a directory", path.display()),
            )
            .into());
        }
        Ok(Self { path, languages })
    }

    pub fn source_files(&self) -> impl Iterator<Item = PathBuf> {
        self.languages
            .iter()
            .flat_map(SourceLanguage::file_extensions)
            .map(|ext| format!("{}/**/*.{}", self.path.display(), ext))
            .flat_map(|pattern| glob(&pattern).expect("glob pattern to be valid").flatten())
    }
}
