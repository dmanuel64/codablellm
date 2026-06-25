use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use glob::glob;
use strum::IntoEnumIterator;
use thiserror::Error;
use url::Url;

use crate::{FileSource, language::Language, storage};

static REPOS_ROOT: LazyLock<PathBuf> = LazyLock::new(|| storage::CACHE_DIR.join("repos"));

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to decompress repository")]
    Storage(#[from] storage::Error),
}

pub struct Repository {
    pub path: PathBuf,
    pub source: Source,
}

pub enum Source {
    Remote { metadata: Metadata, forge: Forge },
    Local { metadata: Metadata, path: PathBuf },
}

impl Source {
    pub fn dest_path(&self) -> PathBuf {
        match self {
            Source::Remote { metadata, forge } => {
                storage::DATA_DIR.join(forge.slug()).join(format!(
                    "{}-{}-{}",
                    metadata.owner,
                    metadata.name,
                    metadata.git_ref.as_str()
                ))
            }
            Source::Local { metadata, path } => storage::DATA_DIR.join("local").join(format!(
                "{}-{}-{}",
                metadata.owner,
                metadata.name,
                metadata.git_ref.as_str()
            )),
        }
    }
}

pub struct Metadata {
    pub owner: String,
    pub name: String,
    pub git_ref: GitRef,
}

pub enum Forge {
    #[cfg(feature = "github")]
    GitHub,
    #[cfg(feature = "gitlab")]
    GitLab,
    #[cfg(feature = "forgejo")]
    Forgejo { base_url: Url },
    #[cfg(feature = "forgejo")]
    Gitea { base_url: Url },
    #[cfg(feature = "custom-forge")]
    Custom { base_url: Url },
}

impl Forge {
    pub fn slug(&self) -> &str {
        match self {
            Forge::GitHub => "github.com",
            Forge::GitLab => "gitlab.com",
            Forge::Forgejo { base_url } | Forge::Gitea { base_url } => {
                // TODO: should return error
                base_url.host_str().unwrap_or("unknown")
            }
            // TODO: should return error
            Forge::Custom { base_url } => base_url.host_str().unwrap_or("custom"),
        }
    }
}

// TODO: evaluate whether this should stay as an enum or new-type struct
pub enum GitRef {
    Branch(String),
    Tag(String),
    Commit(String),
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

pub fn fetch_with_options(source: Source, options: Options) -> Result<Repository, Error> {
    match repository {
        Repository::Local(path) => Ok(Repository::Local(path)),
        #[cfg(any(
            feature = "github",
            feature = "gitlab",
            feature = "forgejo",
            feature = "custom-forge"
        ))]
        Repository::Remote(remote) => {
            let local_path = fetch_remote(&remote, &options)?;
            Ok(Repository::Local(local_path))
        }
    }
}
