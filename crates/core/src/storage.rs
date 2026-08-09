use directories::ProjectDirs;
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};
use thiserror::Error;
use tokio::fs;

static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("io.github", "dmanuel64", "codablellm")
        .expect("a home directory to be found on the host system")
});
pub static DATA_DIR: LazyLock<&Path> = LazyLock::new(|| DIRS.data_local_dir());
pub static CONFIG_DIR: LazyLock<&Path> = LazyLock::new(|| DIRS.config_local_dir());
pub static CACHE_DIR: LazyLock<&Path> = LazyLock::new(|| DIRS.cache_dir());
pub static STATE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    DIRS.state_dir()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| DATA_DIR.join("state"))
});

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] tokio::io::Error),
    #[error("{kind} \"{name}\": {kind} already exists")]
    DataExists { kind: &'static str, name: String },
    #[error("{kind} \"{name}\": {kind} does not exist")]
    DataNotFound { kind: &'static str, name: String },
    #[error(transparent)]
    TokioBlocking(#[from] tokio::task::JoinError),
}

// 1. Ensure directory exists (Native Async)
pub(crate) async fn ensure_dir_exists(path: &Path) -> Result<(), Error> {
    fs::create_dir_all(path).await.map_err(Error::from)
}

// 2. Delete data (Native Async)
pub(crate) async fn delete_data(kind: &'static str, path: &Path) -> Result<(), Error> {
    // Note: path.exists() is blocking. Use fs::try_exists in async.
    if !fs::try_exists(path).await.unwrap_or(false) {
        return Err(Error::DataNotFound {
            kind,
            name: path.to_string_lossy().to_string(),
        });
    }

    let result = if fs::metadata(path).await.map_err(Error::from)?.is_dir() {
        fs::remove_dir_all(path).await // Recursively deletes a directory
    } else {
        fs::remove_file(path).await
    };

    result.map_err(Error::from)
}

// 3. Copy data (Native Async with a helper for recursive directory copy)
pub(crate) async fn copy_data(
    kind: &'static str,
    src: &Path,
    dest: &Path,
    force: bool,
) -> Result<(), Error> {
    if !force && fs::try_exists(dest).await.unwrap_or(false) {
        return Err(Error::DataExists {
            kind,
            name: src.to_string_lossy().to_string(),
        });
    }

    if let Some(parent) = dest.parent() {
        ensure_dir_exists(parent).await?;
    }

    let is_dir = fs::metadata(src).await.map_err(Error::from)?.is_dir();

    if is_dir {
        // Replicates fs_extra's content_only behavior asynchronously
        copy_dir_contents(src, dest, force).await?;
    } else {
        if force || !fs::try_exists(dest).await.unwrap_or(false) {
            fs::copy(src, dest).await.map_err(Error::from)?;
        }
    }

    Ok(())
}

// Helper function to recursively copy directory contents asynchronously
async fn copy_dir_contents(src: &Path, dest: &Path, force: bool) -> Result<(), Error> {
    fs::create_dir_all(dest).await.map_err(Error::from)?;
    let mut entries = fs::read_dir(src).await.map_err(Error::from)?;

    while let Some(entry) = entries.next_entry().await.map_err(Error::from)? {
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let new_dest = dest.join(file_name);

        if entry.file_type().await.map_err(Error::from)?.is_dir() {
            // Box::pin is needed only for async recursion definitions
            Box::pin(copy_dir_contents(&entry_path, &new_dest, force)).await?;
        } else {
            if force || !fs::try_exists(&new_dest).await.unwrap_or(false) {
                fs::copy(&entry_path, &new_dest)
                    .await
                    .map_err(Error::from)?;
            }
        }
    }
    Ok(())
}
