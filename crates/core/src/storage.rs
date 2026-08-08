use directories::ProjectDirs;
use indicatif::{MultiProgress, ProgressBar};
use reqwest::Client;
use std::{
    fmt::Display,
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    str::FromStr,
    sync::LazyLock,
};
use thiserror::Error;
use tokio::{fs, task};
use url::Url;
use zip::{ZipArchive, result::ZipError};

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

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] tokio::io::Error),
    #[error("{kind} \"{name}\": {kind} already exists")]
    DataExists { kind: &'static str, name: String },
    #[error("{kind} \"{name}\": {kind} does not exist")]
    DataNotFound { kind: &'static str, name: String },
    #[error("failed to fetch data: {0}")]
    Fetch(#[from] reqwest::Error),
    #[error("failed to stream data")]
    Streaming(#[source] io::Error),
    #[error("failed to decompress tarball")]
    Tar(#[source] io::Error),
    #[error("failed to decompress zipfile")]
    Zip(#[from] ZipError),
    #[error("unsupported archive type: {ext}")]
    UnsupportedArchive { ext: String },
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

#[derive(Debug, Clone, Default)]
pub struct RequestOptions {}

pub(crate) async fn download_file(
    url: &Url,
    dest: &File,
    display_progress: bool,
) -> Result<(), Error> {
    // Get size of the remote archive
    let head_response = HTTP_CLIENT.head(url.as_str()).send().await?;
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
    let mut get_response = HTTP_CLIENT.get(url.as_str()).send().await?;
    let mut writer = progress.wrap_write(dest);
    while let Some(chunk) = get_response.chunk().await? {
        writer.write_all(&chunk).map_err(Error::Streaming)?;
    }
    Ok(())
}

pub(crate) async fn decompress_archive(
    archive_path: PathBuf,
    dest_dir: PathBuf,
    progress_mgr: Option<MultiProgress>,
) -> Result<(), Error> {
    task::spawn_blocking(move || {
        let mut file = File::open(&archive_path).map_err(Error::from)?;
        // Detect format (GZIP vs ZIP)
        let mut magic = [0u8; 2];
        let _ = file.read_exact(&mut magic);
        // Rewind back to start
        file.seek(SeekFrom::Start(0)).map_err(Error::from)?;

        let progress = match &progress_mgr {
            Some(mp) => mp.add(ProgressBar::no_length()),
            None => ProgressBar::hidden(),
        };
        let reader = progress.wrap_read(file);

        match magic {
            [0x1F, 0x8B] => {
                // Gzip Magic Number -> Process as Tarball (.tar.gz)
                progress.set_message("Decompressing Tarball...");
                let gz = flate2::read::GzDecoder::new(reader);
                let mut archive = tar::Archive::new(gz);
                archive.unpack(&dest_dir).map_err(|e| Error::Tar(e))?;
            }
            [0x50, 0x4B] => {
                // PK Zip Magic Number -> Process as Zipfile (.zip)
                progress.set_message("Extracting Zip Archive...");
                let mut archive = ZipArchive::new(reader)?;
                for i in 0..archive.len() {
                    let mut file = archive.by_index(i)?;
                    let outpath = match file.enclosed_name() {
                        Some(path) => dest_dir.join(path),
                        None => continue, // Skip suspicious/malformed traversal paths
                    };

                    if file.is_dir() {
                        std::fs::create_dir_all(&outpath)?;
                    } else {
                        if let Some(p) = outpath.parent() {
                            std::fs::create_dir_all(p)?;
                        }
                        let mut outfile = std::fs::File::create(&outpath)?;
                        std::io::copy(&mut file, &mut outfile)?;
                    }
                }
            }
            _ => {
                return Err(Error::UnsupportedArchive {
                    ext: archive_path
                        .extension()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                });
            }
        }
        progress.finish_with_message("Decompression complete!");
        Ok(())
    })
    .await? // Catch thread panics safely
}
