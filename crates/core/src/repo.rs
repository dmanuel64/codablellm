use glob::glob;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::{fs, task};
use url::Url;

use crate::{language::Language, storage, utils::ProgressDisplay};

pub static REPOS_ROOT: LazyLock<PathBuf> = LazyLock::new(|| storage::CACHE_DIR.join("repos"));

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to decompress repository")]
    Storage(#[from] storage::Error),
    #[error("unsupported URL scheme")]
    UnsupportedScheme,
    #[cfg(feature = "git")]
    #[error("failed to prepare git clone")]
    ClonePrepare(#[from] gix::clone::Error),
    #[cfg(feature = "git")]
    #[error("failed to fetch git repository")]
    CloneFetch(#[from] gix::clone::fetch::Error),
    #[cfg(feature = "git")]
    #[error("failed to check out git repository")]
    CloneCheckout(#[from] gix::clone::checkout::main_worktree::Error),
    #[cfg(feature = "git")]
    #[error("invalid git ref")]
    InvalidRef(#[source] Box<dyn std::error::Error + Send + Sync>),
}

pub struct Repository {
    pub path: PathBuf,
    pub source: Location,
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

fn dest_path(
    forge: Option<&str>,
    kind: &Kind,
    Metadata {
        owner,
        name,
        git_ref,
    }: &Metadata,
) -> PathBuf {
    let forge = forge
        .map(|f| f.replace(".", "-"))
        .unwrap_or_else(|| "local".to_string());
    let kind_dirname = match kind {
        Kind::Direct => "direct",
        #[cfg(feature = "git")]
        Kind::Git => "git",
    };
    let path = REPOS_ROOT
        .join(forge)
        .join(kind_dirname)
        .join(owner)
        .join(name);
    match kind {
        Kind::Direct => match git_ref {
            Some(tag) => path.join(tag),
            None => path.join("unknown"),
        },
        #[cfg(feature = "git")]
        Kind::Git => path,
    }
}

#[derive(Debug, Clone)]
pub enum Location {
    Path(PathBuf),
    Url(Url),
}

impl FromStr for Location {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Url::parse(s) {
            Ok(url) if matches!(url.scheme(), "http" | "https") => Ok(Location::Url(url)),
            // single letter scheme = Windows drive path (C:\...)
            Ok(url) if url.scheme().len() == 1 => Ok(Location::Path(PathBuf::from(s))),
            // file:// -> honor it as local
            Ok(url) if url.scheme() == "file" => url
                .to_file_path()
                .map(Location::Path)
                .map_err(|_| Error::UnsupportedScheme),
            // some other scheme we don't handle (ssh://, git://...) → explicit error
            Ok(_) => Err(Error::UnsupportedScheme),
            Err(_) => Ok(Location::Path(PathBuf::from(s))),
        }
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Location::Path(path_buf) => path_buf.display().to_string(),
                Location::Url(remote_file) => remote_file.to_string(),
            }
        )
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

#[derive(Debug, Clone)]
pub struct Metadata {
    pub owner: String,
    pub name: String,
    pub git_ref: Option<String>,
}

impl Metadata {
    pub fn from_github(url: &Url) -> Option<Self> {
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

    pub fn from_gitea(url: &Url) -> Option<Self> {
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

#[derive(Debug, Clone)]
pub enum Kind {
    Direct,
    #[cfg(feature = "git")]
    Git,
}

impl From<&Location> for Kind {
    fn from(value: &Location) -> Self {
        match value {
            Location::Url(url) => {
                #[cfg(feature = "git")]
                if url.scheme() == "git" || url.path().ends_with(".git") {
                    return Kind::Git;
                }
                Kind::Direct
            }
            Location::Path(path) => {
                #[cfg(feature = "git")]
                if path.join(".git").is_dir() {
                    return Kind::Git;
                }
                Kind::Direct
            }
        }
    }
}

/// How to refresh a repository that may already be cached at its
/// destination path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshMode {
    /// Re-fetch/re-clone from scratch, overwriting whatever is cached.
    ForceDownload,
    /// If the cached copy is a git clone, fetch and fast-forward it in
    /// place. A no-op (not an error) if it's already up to date, or if
    /// the cached copy isn't a git clone at all.
    #[cfg(feature = "git")]
    Pull,
}

#[derive(Debug, Clone)]
pub struct Options {
    pub kind: Option<Kind>,
    pub progress_display: ProgressDisplay,
    pub force: Option<RefreshMode>,
    // pub request_builder: Option<reqwest::ClientBuilder>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            kind: None,
            progress_display: ProgressDisplay::default(),
            force: None,
            // request_builder: None,
        }
    }
}

#[cfg(feature = "git")]
async fn clone(
    url: &Url,
    git_ref: Option<&str>,
    dest: PathBuf,
    progress_display: ProgressDisplay,
) -> Result<(), Error> {
    if let Some(parent) = dest.parent() {
        storage::ensure_dir_exists(parent).await?;
    }

    let progress = progress_display.new_spinner();
    progress.set_message("Cloning repository...");

    // TODO: gix's fetch/checkout progress callbacks are given `Discard`
    // above, so this spinner only ever shows a start/finish message, no
    // incremental ticks the way the archive-download path gets via
    // `progress.wrap_write`. Bridging gix's `Progress`/`NestedProgress`
    // trait into an indicatif bar can go in later.
    let url = url.as_str().to_owned();
    let git_ref = git_ref.map(str::to_owned);
    task::spawn_blocking(move || -> Result<(), Error> {
        let mut prepare = gix::prepare_clone(url.as_str(), &dest)?;
        if let Some(r) = git_ref.as_deref() {
            prepare = prepare
                .with_ref_name(Some(r))
                .map_err(|e| Error::InvalidRef(Box::new(e)))?;
        }
        let (mut checkout, _) =
            prepare.fetch_then_checkout(gix::progress::Discard, &Default::default())?;
        checkout.main_worktree(gix::progress::Discard, &Default::default())?;
        Ok(())
    })
    .await
    .map_err(storage::Error::from)??;

    progress.finish_with_message("Clone complete!");
    Ok(())
}

#[cfg(feature = "git")]
async fn pull(_dest: &Path, _progress_display: &ProgressDisplay) -> Result<(), Error> {
    // TODO: gix doesn't have a one-shot "pull" helper the way it has
    // `prepare_clone` for cloning - updating an already-checked-out repo
    // means opening it, finding its remote, fetching, and fast-forwarding
    // the worktree by hand. Needs its own verification pass against the
    // gix API (the same way `clone()` got) rather than guessing at it here.
    todo!("pull latest changes for the existing git clone at {}", _dest.display())
}

pub async fn fetch(location: Location, metadata: Metadata) -> Result<Repository, Error> {
    fetch_with_options(location, metadata, Options::default()).await
}

pub async fn fetch_with_options(
    location: Location,
    metadata: Metadata,
    Options {
        kind,
        progress_display,
        force,
        // request_builder,
    }: Options,
) -> Result<Repository, Error> {
    let kind = kind.unwrap_or_else(|| Kind::from(&location));
    let forge = match &location {
        Location::Path(_) => None,
        Location::Url(url) => Some(url.host_str().unwrap_or("unknown")),
    };
    let dest = dest_path(forge, &kind, &metadata);
    let exists = fs::try_exists(&dest).await.unwrap_or(false);

    // Whether we need to fetch at all, and (for the Path/local case only)
    // whether an existing destination may be overwritten.
    #[cfg_attr(not(feature = "git"), allow(unused_mut))]
    let mut should_fetch = !exists;
    match force {
        Some(RefreshMode::ForceDownload) => should_fetch = true,
        #[cfg(feature = "git")]
        Some(RefreshMode::Pull) if exists => {
            // Only meaningful for git clones; silently do nothing for an
            // up-to-date repo or a non-git cached copy.
            if matches!(kind, Kind::Git) {
                pull(&dest, &progress_display).await?;
            }
        }
        _ => {}
    }
    let overwrite = force == Some(RefreshMode::ForceDownload);

    if should_fetch {
        match &location {
            Location::Path(path) => {
                storage::copy_data("repository", path, &dest, overwrite).await?
            }
            Location::Url(url) => match kind {
                Kind::Direct => {
                    storage::ensure_dir_exists(&dest).await?;
                    let archive = NamedTempFile::new().map_err(storage::Error::Io)?;
                    storage::download_file(url, archive.as_file(), progress_display.clone())
                        .await?;
                    storage::decompress_archive(
                        archive.path().to_path_buf(),
                        dest.clone(),
                        progress_display.clone(),
                    )
                    .await?;
                }
                #[cfg(feature = "git")]
                Kind::Git => {
                    clone(
                        url,
                        metadata.git_ref.as_deref(),
                        dest.clone(),
                        progress_display.clone(),
                    )
                    .await?
                }
            },
        }
    }

    Ok(Repository {
        path: dest,
        source: location,
        languages: Vec::new(), // TODO: detect languages present in the repo
    })
}
