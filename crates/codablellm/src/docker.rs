use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use crate::config;

pub static VOLUME_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    config::APP_DIRS
        .runtime_dir()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::temp_dir().join("codablellm"))
});
