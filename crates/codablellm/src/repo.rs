use std::{path::PathBuf, sync::LazyLock};

use crate::config;

static LOCAL_REPO_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| config::APP_DIRS.config_local_dir().join("repos"));
