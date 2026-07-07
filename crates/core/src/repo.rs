use glob::glob;
use std::{path::PathBuf, sync::LazyLock};
use thiserror::Error;
use url::Url;

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
            paths.extend(glob(&pattern).expect("glob pattern to be valid").flatten());
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
            self.metadata
                .git_ref
                .as_ref()
                .map(String::as_str)
                .unwrap_or("unknown")
        )
        .replace(".", "-");
        REPOS_ROOT.join(slug_dirname).join(repo_dirname)
    }
}

fn strip_git_suffix(s: &str) -> &str {
    s.strip_suffix(".git").unwrap_or(s)
}

fn strip_archive_ext(s: &str) -> &str {
    [".tar.gz", ".tgz", ".tar.bz2", ".tar", ".zip"]
        .iter()
        .find_map(|ext| s.strip_suffix(ext))
        .unwrap_or(s)
}

/// Last segment is the repo; everything before it is the owner path
/// (handles GitLab groups/subgroups). Requires at least owner + repo.
fn split_owner_repo(path: &[&str]) -> Option<(String, String)> {
    let (repo, owner) = path.split_last()?;
    if owner.is_empty() {
        return None;
    }
    Some((owner.join("/"), strip_git_suffix(repo).to_string()))
}

pub struct Metadata {
    pub owner: String,
    pub name: String,
    pub git_ref: Option<String>,
}

impl Metadata {
    pub fn from_github_url(url: &Url) -> Option<Self> {
        let segs: Vec<&str> = url.path_segments()?.filter(|s| !s.is_empty()).collect();
        let owner = segs.first()?.to_string();
        let name = strip_git_suffix(segs.get(1)?).to_string();

        let git_ref = match &segs[2..] {
            [] => None,
            ["archive", "refs", "heads", tail @ ..] | ["archive", "refs", "tags", tail @ ..]
                if !tail.is_empty() =>
            {
                Some(strip_archive_ext(&tail.join("/")).to_string())
            }
            ["archive", tail @ ..] if !tail.is_empty() => {
                Some(strip_archive_ext(&tail.join("/")).to_string())
            }
            ["tree", tail @ ..] | ["blob", tail @ ..] if !tail.is_empty() => Some(tail.join("/")),
            _ => None,
        };

        Some(Metadata {
            owner,
            name,
            git_ref,
        })
    }

    pub fn from_gitlab(url: &Url) -> Option<Self> {
        let segs: Vec<&str> = url.path_segments()?.filter(|s| !s.is_empty()).collect();

        match segs.iter().position(|&s| s == "-") {
            Some(dash) => {
                let (owner, name) = split_owner_repo(&segs[..dash])?;
                let git_ref = match &segs[dash + 1..] {
                    // /-/archive/<ref...>/<filename>
                    ["archive", middle @ .., _file] if !middle.is_empty() => Some(middle.join("/")),
                    ["tree", tail @ ..] if !tail.is_empty() => Some(tail.join("/")),
                    _ => None,
                };
                Some(Metadata {
                    owner,
                    name,
                    git_ref,
                })
            }
            // plain clone / .git URL: whole path is the project, no ref
            None => {
                let (owner, name) = split_owner_repo(&segs)?;
                Some(Metadata {
                    owner,
                    name,
                    git_ref: None,
                })
            }
        }
    }

    pub fn from_gitea_url(url: &Url) -> Option<Self> {
        let segs: Vec<&str> = url.path_segments()?.filter(|s| !s.is_empty()).collect();
        let owner = segs.first()?.to_string();
        let name = strip_git_suffix(segs.get(1)?).to_string();

        let git_ref = match &segs[2..] {
            ["archive", tail @ ..] if !tail.is_empty() => {
                Some(strip_archive_ext(&tail.join("/")).to_string())
            }
            ["src", "branch", tail @ ..]
            | ["src", "tag", tail @ ..]
            | ["src", "commit", tail @ ..]
                if !tail.is_empty() =>
            {
                Some(tail.join("/"))
            }
            _ => None,
        };

        Some(Metadata {
            owner,
            name,
            git_ref,
        })
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
