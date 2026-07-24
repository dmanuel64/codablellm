use serde::{Deserialize, Serialize};
use std::path::Path;
use strum::{Display, EnumIter};
use thiserror::Error;

use crate::{container, repo::Repository};

#[derive(Debug, Clone, Copy, Display, EnumIter)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[cfg_attr(
    feature = "value-enums",
    derive(clap::ValueEnum, Serialize, Deserialize),
    serde(rename_all = "lowercase"),
    clap(rename_all = "lowercase")
)]
pub enum Target {
    Ubuntu,
    Alpine,
    Windows,
}

impl Target {
    pub fn operating_system(&self) -> OperatingSystem {
        match self {
            Target::Alpine | Target::Ubuntu => OperatingSystem::Linux,
            Target::Windows => OperatingSystem::Windows,
        }
    }

    pub fn platform(&self, arch: &Architecture) -> String {
        format!(
            "{}/{}",
            self.operating_system().to_string(),
            arch.to_string()
        )
    }

    fn image(&self, version: Option<&str>) -> String {
        let version = version.unwrap_or("latest");
        format!(
            "dmanuel99/codablellm-builder:{}-{version}",
            self.to_string()
        )
    }
}

#[derive(Debug, Clone, Copy, Display, EnumIter)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
pub enum OperatingSystem {
    Linux,
    Windows,
}

#[derive(Debug, Clone, Copy, Display, EnumIter)]
#[strum(serialize_all = "lowercase", ascii_case_insensitive)]
#[cfg_attr(
    feature = "value-enums",
    derive(clap::ValueEnum, Serialize, Deserialize),
    serde(rename_all = "lowercase"),
    clap(rename_all = "lowercase")
)]
pub enum Architecture {
    #[cfg_attr(feature = "value-enums", serde(alias = "x86_64"))]
    Amd64,
    Arm64,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Container(#[from] container::Error),
    #[error("failed to run build command: {0}")]
    BuildError(#[source] bollard::errors::Error),
}

const CONTAINER_NAME: &str = "codablellm-builder";

/// Builds a repository in a builder container given a build command, and paths to where the expected build artifacts reside
pub async fn build(
    repo: &Repository,
    commands: &[&str],
    artifacts: &[&Path],
    target: Target,
    arch: Architecture,
) -> Result<(), Error> {
    let conn = container::connect_runtime()?;
    let image = container::pull_image(&conn, &target.image(None), &target.platform(&arch)).await?;
    let name = container::create(&conn, CONTAINER_NAME, &image).await?;
    conn.start_container(&name, None)
        .await
        .map_err(Error::BuildError)
}
