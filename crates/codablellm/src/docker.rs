use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use bollard::{Docker, plugin::ExecConfig};
use tempfile::tempdir;

pub async fn foo() {
    let docker = Docker::connect_with_defaults().unwrap();
    docker.create_volume(config)
    docker.create_exec(container_name, ExecConfig)
}
