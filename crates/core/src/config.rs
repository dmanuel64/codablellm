use directories::ProjectDirs;
use fs_extra::{dir, file};
use std::{path::Path, sync::LazyLock};
use thiserror::Error;

pub(crate) static APP_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("io.github", "dmanuel64", "codablellm")
        .expect("a home directory to be found on the host system")
});

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] fs_extra::error::Error),
    #[error("{kind} \"{name}\": {kind} already exists")]
    DataExists { kind: &'static str, name: String },
    #[error("{kind} \"{name}\": {kind} does not exist")]
    DataNotFound { kind: &'static str, name: String },
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
