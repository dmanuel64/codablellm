use directories::ProjectDirs;
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("io.github", "dmanuel64", "codablellm")
        .expect("a home directory to be found on the host system")
});

pub static CONFIG_DIR: LazyLock<&Path> = LazyLock::new(|| DIRS.config_local_dir());
pub static STATE_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    DIRS.state_dir()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| DIRS.data_local_dir().join("state"))
});
