use directories::ProjectDirs;
use flate2::read::GzDecoder;
use fs_extra::{dir, file};
use indicatif::ProgressBar;
use reqwest::blocking::Client;
use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};
use tar::Archive;
use thiserror::Error;
use url::Url;

static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("io.github", "dmanuel64", "codablellm")
        .expect("a home directory to be found on the host system")
});
pub static DATA_DIR: LazyLock<&Path> = LazyLock::new(|| DIRS.data_local_dir());
pub static CONFIG_DIR: LazyLock<&Path> = LazyLock::new(|| DIRS.config_local_dir());
pub static CACHE_DIR: LazyLock<&Path> = LazyLock::new(|| DIRS.cache_dir());
pub static STATE_DIR: LazyLock<&Path> =
    LazyLock::new(|| DIRS.state_dir().unwrap_or_else(|| &*CACHE_DIR));

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] fs_extra::error::Error),
    #[error("{kind} \"{name}\": {kind} already exists")]
    DataExists { kind: &'static str, name: String },
    #[error("{kind} \"{name}\": {kind} does not exist")]
    DataNotFound { kind: &'static str, name: String },
    #[error("could not determine the source type of the data")]
    AmbiguousSource,
    #[error("failed to fetch data: {0}")]
    Fetch(#[from] reqwest::Error),
    #[error("failed to stream data")]
    Streaming(#[source] io::Error),
    #[error("failed to decompress repository")]
    Decompression(#[source] io::Error),
}

// TODO: config doesn't seem the best spot for the io stuff
pub(crate) fn ensure_dir_exists(path: &Path) -> Result<(), Error> {
    dir::create_all(path, false).map_err(Error::from)
}

pub(crate) fn copy_data(
    kind: &'static str,
    src: &Path,
    dest: &Path,
    force: bool,
) -> Result<(), Error> {
    if !force && dest.exists() {
        return Err(Error::DataExists {
            kind,
            name: src.to_string_lossy().to_string(),
        });
    }

    if let Some(parent) = dest.parent() {
        ensure_dir_exists(parent)?;
    }

    let result = if src.is_dir() {
        let options = dir::CopyOptions::new().overwrite(force).content_only(true);
        dir::copy(src, dest, &options)
    } else {
        let options = file::CopyOptions::new().overwrite(force);
        file::copy(src, dest, &options)
    };

    result.map_err(Error::from)?;
    Ok(())
}

pub(crate) fn delete_data(kind: &'static str, path: &Path) -> Result<(), Error> {
    if !path.exists() {
        return Err(Error::DataNotFound {
            kind,
            name: path.to_string_lossy().to_string(),
        });
    }

    let result = if path.is_dir() {
        dir::remove(path)
    } else {
        file::remove(path)
    };

    result.map_err(Error::from)
}

#[derive(Debug, Clone, Default)]
pub struct RequestOptions {}

#[derive(Debug, Clone)]
pub struct RemoteFile {
    pub url: Url,
    pub request_options: RequestOptions,
}

impl RemoteFile {
    pub fn new(url: Url) -> Self {
        Self::new_with_options(url, RequestOptions::default())
    }

    pub fn new_with_options(url: Url, request_options: RequestOptions) -> Self {
        Self {
            url,
            request_options,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FileSource {
    Local(PathBuf),
    Remote(RemoteFile),
}

impl FromStr for FileSource {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(url) = Url::parse(s) {
            Ok(FileSource::Remote(RemoteFile::new(url)))
        } else if s.contains("://") {
            Err(Error::AmbiguousSource)
        } else {
            Ok(FileSource::Local(PathBuf::from(s)))
        }
    }
}

impl TryFrom<FileSource> for PathBuf {
    type Error = Error;

    fn try_from(value: FileSource) -> Result<Self, Self::Error> {
        match value {
            FileSource::Local(path) => Ok(path),
            FileSource::Remote(url) => todo!(),
        }
    }
}

pub(crate) fn download_file(
    src: &RemoteFile,
    dest: &File,
    display_progress: bool,
) -> Result<(), Error> {
    // Get size of the remote archive
    let head_response = HTTP_CLIENT.head(src.url.as_ref()).send()?;
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
    let mut get_response = HTTP_CLIENT.get(src.url.as_ref()).send()?;
    let mut writer = progress.wrap_write(dest);
    io::copy(&mut get_response, &mut writer).map_err(|e| Error::Streaming(e))?;
    Ok(())
}

pub(crate) fn decompress_archive(
    archive: &File,
    dest_dir: &Path,
    display_progress: bool,
) -> Result<(), Error> {
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
    archive
        .unpack(dest_dir)
        .map_err(|e| Error::Decompression(e))?;
    Ok(())
}
