use glob::glob;
use std::{io, path::PathBuf};
use thiserror::Error;

use crate::language::Language;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] io::Error),
}

pub struct Repository {
    pub path: PathBuf,
    pub languages: Vec<Language>,
}

impl Repository {
    pub fn source_files(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for ext in self.languages.iter().flat_map(Language::file_extensions) {
            let pattern = format!("{}/**/*.{}", self.path.display(), ext);
            paths.extend(glob(&pattern).expect("glob pattern to be valid").flatten());
        }
        paths
    }
}

pub fn load(path: PathBuf) -> Result<Repository, Error> {
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{} is not a directory", path.display()),
        )
        .into());
    }

    Ok(Repository {
        path,
        languages: Vec::new(), // TODO: detect languages present in the repo
    })
}
