use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use glob::glob;
use strum::IntoEnumIterator;
use tempfile::tempfile;
use thiserror::Error;
use url::Url;

use crate::{
    FileSource,
    language::Language,
    storage::{self, RemoteFile},
};

static LOCAL_REPO_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| storage::APP_DIRS.cache_dir().join("repos"));

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to decompress repository")]
    Storage(#[from] storage::Error),
}

pub enum Format {
    Archive(storage::ArchiveFormat),
    #[cfg(feature = "git")]
    Git,
}

pub struct Repository {
    path: PathBuf,
    origin: Option<Url>,
    pub languages: Vec<Language>,
}

impl Repository {
    pub fn new(path: PathBuf) -> Self {
        let languages = Language::iter().collect();
        Self {
            path,
            origin: None,
            languages,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn source_files(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for ext in self.languages.iter().flat_map(|l| l.file_extensions()) {
            let pattern = format!("{}/**/*.{}", self.path.display(), ext);
            paths.extend(glob(&pattern).unwrap().flatten());
        }
        paths
    }
}

#[derive(Debug)]
pub struct Options {
    pub display_progress: bool,
    pub request_builder: Option<reqwest::blocking::ClientBuilder>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            display_progress: true,
            request_builder: None,
        }
    }
}

pub fn pull(url: FileSource) -> Result<Repository, Error> {
    pull_with_options(url, Options::default())
}

pub fn pull_with_options(url: Url, options: Options) -> Result<Repository, Error> {
    // TODO: check if zipfile or tarfile
    let archive = tempfile().map_err(|e| storage::Error::Streaming(e))?;
    storage::download_file(&RemoteFile::new(url), &archive, options.display_progress)?;
    let local_repo_dir = LOCAL_REPO_ROOT.join(url_to_path(&url));
    storage::decompress_archive(&archive, &local_repo_dir, options.display_progress)?;
    Ok(Repository::new(local_repo_dir))
}

fn url_to_path(url: &Url) -> String {
    let root = url.host_str().unwrap_or("unknown");
}
