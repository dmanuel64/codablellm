use directories::ProjectDirs;
use std::{fs, io, path::Path, sync::LazyLock};
use thiserror::Error;

pub(crate) static APP_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("io.github", "dmanuel64", "codablellm")
        .expect("a home directory to be found on the host system")
});

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create CodableLLM data directory")]
    DataDirectoryError(#[source] io::Error),
    #[error("failed to copy {kind} data")]
    CopyDataFailed {
        kind: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("cannot create {kind} \"{name}\": {kind} already exists")]
    DataExists { kind: &'static str, name: String },
}

pub(crate) fn ensure_dir_exists(path: &Path) -> Result<(), Error> {
    fs::create_dir_all(path).map_err(|e| Error::DataDirectoryError(e))
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
    fs::copy(src, dest).map_err(|e| Error::CopyDataFailed { kind, source: e })?;
    Ok(())
}
