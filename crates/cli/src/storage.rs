use directories::ProjectDirs;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::LazyLock,
};

static DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("io.github", "dmanuel64", "codablellm")
        .expect("a home directory to be found on the host system")
});

pub static DATA_DIR: LazyLock<&Path> = LazyLock::new(|| DIRS.data_local_dir());
pub static CONFIG_DIR: LazyLock<&Path> = LazyLock::new(|| DIRS.config_local_dir());
pub static CACHE_DIR: LazyLock<&Path> = LazyLock::new(|| DIRS.cache_dir());
pub static STATE_DIR: LazyLock<Cow<'_, Path>> = LazyLock::new(|| {
    DIRS.state_dir()
        .map(Cow::from)
        .unwrap_or_else(|| DATA_DIR.join("state").into())
});
