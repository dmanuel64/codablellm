use glob::glob;
use std::{path::PathBuf, sync::LazyLock};
use thiserror::Error;

use crate::{FileSource, language::Language, storage};

pub static ARCHIVES_ROOT: LazyLock<PathBuf> = LazyLock::new(|| storage::CACHE_DIR.join("repos"));
pub static REPOS_ROOT: LazyLock<PathBuf> = LazyLock::new(|| storage::DATA_DIR.join("repos"));

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to decompress repository")]
    Storage(#[from] storage::Error),
}

pub struct Repository {
    pub path: PathBuf,
    pub source: Source,
    pub languages: Vec<Language>,
}

impl Repository {
    pub fn source_files(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for ext in self.languages.iter().flat_map(Language::file_extensions) {
            let pattern = format!("{}/**/*.{}", self.path.display(), ext);
            paths.extend(glob(&pattern).unwrap().flatten());
        }
        paths
    }
}

pub struct Source {
    pub metadata: Metadata,
    pub path: FileSource,
}

impl Source {
    pub fn dest_path(&self) -> PathBuf {
        let slug_dirname = match &self.path {
            FileSource::Local(_) => "local",
            FileSource::Remote(remote) => {
                &remote.url.host_str().unwrap_or("unknown").replace(".", "-")
            }
        };
        let repo_dirname = format!(
            "{}-{}-{}",
            self.metadata.owner,
            self.metadata.name,
            self.metadata.git_ref.to_string()
        )
        .replace(".", "-");
        REPOS_ROOT.join(slug_dirname).join(repo_dirname)
    }
}

pub struct Metadata {
    pub owner: String,
    pub name: String,
    pub git_ref: GitRef,
}

// TODO: evaluate whether this should stay as an enum or new-type struct
pub enum GitRef {
    Branch(String),
    Tag(String),
    Commit(String),
}

impl ToString for GitRef {
    fn to_string(&self) -> String {
        match self {
            GitRef::Branch(c) | GitRef::Commit(c) | GitRef::Tag(c) => c.clone(),
        }
    }
}

impl GitRef {
    pub fn main_branch() -> Self {
        Self::Branch("main".to_string())
    }

    pub fn master_branch() -> Self {
        Self::Branch("master".to_string())
    }
}

impl Default for GitRef {
    fn default() -> Self {
        Self::main_branch()
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

pub fn fetch(source: Source) -> Result<Repository, Error> {
    fetch_with_options(source, Options::default())
}

pub fn fetch_with_options(
    source: Source,
    Options {
        display_progress,
        request_builder,
    }: Options,
) -> Result<Repository, Error> {
    todo!()
}
