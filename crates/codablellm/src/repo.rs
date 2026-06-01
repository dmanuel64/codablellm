use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use flate2::read::GzDecoder;
use glob::glob;
use indicatif::ProgressBar;
use strum::IntoEnumIterator;
use tar::Archive;
use tempfile::tempfile;

use crate::{config, languages::Language};

static LOCAL_REPO_ROOT: LazyLock<PathBuf> =
    LazyLock::new(|| config::APP_DIRS.cache_dir().join("repos"));

pub struct Repo {
    path: PathBuf,
    pub target_languages: Vec<Language>,
}

impl Repo {
    pub fn new(path: PathBuf) -> Self {
        let target_languages = Language::iter().collect();
        Self {
            path,
            target_languages,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn source_files(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for ext in self
            .target_languages
            .iter()
            .flat_map(|l| l.file_extensions())
        {
            let pattern = format!("{}/**/*.{}", self.path.display(), ext);
            paths.extend(glob(&pattern).unwrap().flatten());
        }
        paths
    }
}

pub struct Options {
    pub display_progress: bool,
    pub builder: Option<reqwest::blocking::ClientBuilder>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            display_progress: true,
            builder: None,
        }
    }
}

pub fn pull(repo_url: &str) -> Result<Repo, crate::Error> {
    pull_with_options(repo_url, Options::default())
}

pub fn pull_with_options(repo_url: &str, options: Options) -> Result<Repo, crate::Error> {
    // TODO: check if zipfile or tarfile
    let local_repo_archive = tempfile()?;
    fetch(
        repo_url,
        &local_repo_archive,
        options.builder,
        options.display_progress,
    )?;
    let local_repo_dir = LOCAL_REPO_ROOT.join(url_to_dirname(repo_url));
    decompress(
        &local_repo_archive,
        &local_repo_dir,
        options.display_progress,
    )?;
    Ok(Repo::new(local_repo_dir))
}

fn fetch(
    repo_url: &str,
    dest_file: &File,
    builder: Option<reqwest::blocking::ClientBuilder>,
    display_progress: bool,
) -> Result<(), crate::Error> {
    // Create HTTP client
    let client = if let Some(b) = builder {
        b.build()?
    } else {
        reqwest::blocking::Client::new()
    };
    // Get size of the remote archive
    let head_response = client.head(repo_url).send()?;
    let repo_size = head_response.content_length();
    let progress = if !display_progress {
        ProgressBar::hidden()
    } else if let Some(s) = repo_size {
        ProgressBar::new(s)
    } else {
        ProgressBar::new_spinner()
    };
    // Fetch archive and stream to temporary archive
    progress.set_message("Fetching repo...");
    let mut get_response = client.get(repo_url).send()?;
    let mut writer = progress.wrap_write(dest_file);
    io::copy(&mut get_response, &mut writer)?;
    Ok(())
}

fn decompress(archive: &File, dest_dir: &Path, display_progress: bool) -> Result<(), crate::Error> {
    let progress = if !display_progress {
        ProgressBar::hidden()
    } else {
        ProgressBar::no_length()
    };
    progress.set_message("Decompressing repo...");
    let reader = progress.wrap_read(archive);
    // TODO: this assumes this will always be a tarball
    let gz = GzDecoder::new(reader);
    let mut archive = Archive::new(gz);
    archive.unpack(dest_dir)?;
    Ok(())
}

fn url_to_dirname(url: &str) -> String {
    let stripped = url
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    stripped.replace('/', "-")
}
