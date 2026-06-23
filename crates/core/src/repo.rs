use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use glob::glob;
use strum::IntoEnumIterator;
use thiserror::Error;
use url::Url;

use crate::{FileSource, language::Language, storage};

static REPOS_ROOT: LazyLock<PathBuf> =
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

pub enum Repository {
    #[cfg(any(
        feature = "github",
        feature = "gitlab",
        feature = "forgejo",
        feature = "custom-forge"
    ))]
    Remote(RemoteRepository),
    Local(PathBuf),
}

#[cfg(any(
    feature = "github",
    feature = "gitlab",
    feature = "forgejo",
    feature = "custom-forge"
))]
pub struct RemoteRepository {
    forge: Empty,
    owner: String,
    name: String,
    git_ref: GitRef,
    pub languages: Vec<Language>,
}

#[cfg(any(
    feature = "github",
    feature = "gitlab",
    feature = "forgejo",
    feature = "custom-forge"
))]
pub enum Forge {
    #[cfg(feature = "github")]
    GitHub,
    #[cfg(feature = "gitlab")]
    GitLab,
    #[cfg(feature = "forgejo")]
    Gitea { base_url: Url },
    #[cfg(feature = "forgejo")]
    Forgejo { base_url: Url },
    #[cfg(feature = "custom-forge")]
    Custom(Url),
}

pub enum Empty {}

pub enum GitRef {
    Branch(String),
    Tag(String),
    Commit(String), // worth including if you ever want pinned deps
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

pub fn fetch(source: FileSource) -> Result<Repository, Error> {
    fetch_with_options(source, Options::default())
}

pub fn fetch_with_options(source: FileSource, options: Options) -> Result<Repository, Error> {
    todo!();
    // TODO: check if zipfile or tarfile
    // let archive = tempfile().map_err(|e| storage::Error::Streaming(e))?;
    // let local_repo_dir = REPOS_ROOT.join(url_to_path(&source));
    // storage::download_file(&RemoteFile::new(source), &archive, options.display_progress)?;
    // storage::decompress_archive(&archive, &local_repo_dir, options.display_progress)?;
    // Ok(Repository::new(local_repo_dir))
}

fn url_to_path(url: &Url) -> String {
    let root = url.host_str().unwrap_or("unknown");
    todo!()
}
