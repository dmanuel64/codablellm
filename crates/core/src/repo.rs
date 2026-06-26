use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use gitlab::api::Query;
use glob::glob;
use strum::IntoEnumIterator;
use thiserror::Error;
use url::Url;

use crate::{FileSource, Forge::GitLab, language::Language, storage};

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
    #[cfg(any(
        feature = "github",
        feature = "gitlab",
        feature = "forgejo",
        feature = "custom-forge"
    ))]
    Remote {
        metadata: Metadata,
        forge: Forge,
    },
    Local {
        metadata: Metadata,
        path: PathBuf,
    },
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

pub fn fetch_with_options(source: Source, options: Options) -> Result<Repository, Error> {
    match source {
        Source::Local { metadata, path } => Ok(Source::Local(path)),
        #[cfg(any(
            feature = "github",
            feature = "gitlab",
            feature = "forgejo",
            feature = "custom-forge"
        ))]
        Source::Remote { metadata, forge } => {
            let local_path = fetch_remote(&metadata, &forge)?;
            Ok(Source::Local(local_path))
        }
    }
}

#[cfg(any(
    feature = "github",
    feature = "gitlab",
    feature = "forgejo",
    feature = "custom-forge"
))]
fn fetch_remote(metadata: &Metadata, forge: &Forge) -> Result<Repository, Error> {
    match forge {
        #[cfg(feature = "github")]
        Forge::GitHub => fetch_from_github(metadata),
        #[cfg(feature = "gitlab")]
        Forge::GitLab => fetch_from_gitlab(metadata),
        #[cfg(feature = "forgejo")]
        Forge::Forgejo { base_url } => todo!(),
        #[cfg(feature = "forgejo")]
        Forge::Gitea { base_url } => todo!(),
        #[cfg(feature = "custom-forge")]
        Forge::Custom { base_url } => todo!(),
    }
}

#[cfg(feature = "github")]
fn fetch_from_github(metadata: &Metadata) -> Result<Repository, Error> {
    let octocrab = octocrab::instance();
    let repo = octocrab.repos(metadata.owner, metadata.name);
    repo.download_tarball(metadata.git_ref.to_string());
}

#[cfg(feature = "gitlab")]
fn fetch_from_gitlab(metadata: &Metadata) -> Result<Repository, Error> {
    use gitlab::{
        Gitlab,
        api::projects::repository::{Archive, ArchiveFormat},
    };

    // TODO: inject token properly rather than hardcoding
    let client = Gitlab::new("gitlab.com", "private-token").map_err(Error::GitLab)?;
    let endpoint = Archive::builder()
        .project(format!("{}/{}", metadata.owner, metadata.name))
        .sha(metadata.git_ref.to_string())
        .format(ArchiveFormat::TarGz)
        .build()
        .map_err(Error::GitLabBuilder)?;

    // `raw` returns the bytes directly instead of deserializing JSON
    let bytes: Vec<u8> = gitlab::api::raw(endpoint)
        .query(&client)
        .map_err(Error::GitLabQuery)?;

    extract_tarball_from_bytes(&bytes, dest)
}

#[cfg(feature = "forgejo")]
fn fetch_from_forgejo(metadata: &Metadata) -> Result<Repository, Error> {
    use forgejo_api::{Auth, Forgejo};

    // Use Auth::Basic or Auth::Token if the repo is private
    let api = Forgejo::new(Auth::None, base_url.clone()).map_err(Error::Forgejo)?;

    // get_archive is a blocking call returning bytes
    let bytes = api
        .repo_get_archive(
            &metadata.owner,
            &metadata.name,
            &metadata.git_ref.to_string(),
        )
        .map_err(Error::Forgejo)?;

    extract_tarball_from_bytes(&bytes, dest)
}
