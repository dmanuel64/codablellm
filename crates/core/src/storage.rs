use directories::ProjectDirs;
use fs_extra::{dir, file};
use reqwest::blocking::{Client, ClientBuilder, RequestBuilder};
use std::{
    io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{LazyLock, OnceLock},
};
use thiserror::Error;
use url::Url;

pub(crate) static APP_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("io.github", "dmanuel64", "codablellm")
        .expect("a home directory to be found on the host system")
});


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

#[derive(Debug, Clone)]
pub enum Format {
    Zip,
    Tarball,
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
        RequestBuilder::new()
        Self {
            url,
            request_options,
        }
    }
}

#[derive(Debug, Clone)]
pub enum FileSource {
    Local(PathBuf),
    Url(RemoteFile),
}

impl FromStr for FileSource {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(url) = Url::parse(s) {
            Ok(FileSource::Url(RemoteFile::new(url)))
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
            FileSource::Url(url) => todo!(),
        }
    }
}

impl FileSource {}
