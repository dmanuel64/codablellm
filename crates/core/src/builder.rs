use bollard::{
    Docker,
    plugin::ContainerCreateBody,
    query_parameters::{CreateContainerOptionsBuilder, CreateImageOptionsBuilder},
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::Path;
use strum::{Display, EnumIter};
use thiserror::Error;

use crate::repo::Repository;

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
    #[error("failed to connect to Docker")]
    DockerConnectionError(#[source] bollard::errors::Error),
    #[error("failed to pull builder image")]
    ImagePullError(#[source] bollard::errors::Error),
    #[error("failed to create builder container")]
    BuilderCreationError(#[source] bollard::errors::Error),
    #[error("failed to run build command: {0}")]
    BuildError(#[source] bollard::errors::Error),
}

/// Resolves the builder image for the given target/architecture, pulling it
/// from the registry if it isn't already present locally.
async fn resolve_builder_image(
    conn: &Docker,
    target: Target,
    arch: Architecture,
) -> Result<String, Error> {
    let image = target.image(None);
    if conn.inspect_image(&image).await.is_ok() {
        return Ok(image);
    }
    let options = CreateImageOptionsBuilder::default()
        .from_image(&image)
        .platform(&target.platform(&arch))
        .build();
    let mut pull = conn.create_image(Some(options), None, None);
    while let Some(result) = pull.next().await {
        result.map_err(Error::ImagePullError)?;
    }
    Ok(image)
}

async fn create_builder_container(conn: &Docker, image: &str) -> Result<String, Error> {
    let name = "codablellm-builder";
    let options = CreateContainerOptionsBuilder::default().name(name).build();
    let config = ContainerCreateBody {
        image: Some(image.to_string()),
        ..Default::default()
    };
    conn.create_container(Some(options), config)
        .await
        .map_err(|e| Error::BuilderCreationError(e))?;
    Ok(name.to_string())
}

/// Builds a repository in a builder container given a build command, and paths to where the expected build artifacts reside
pub async fn build(
    repo: &Repository,
    commands: &[&str],
    artifacts: &[&Path],
    target: Target,
    arch: Architecture,
) -> Result<(), Error> {
    let conn = Docker::connect_with_defaults().map_err(|e| Error::DockerConnectionError(e))?;
    let image = resolve_builder_image(&conn, target, arch).await?;
    let name = create_builder_container(&conn, &image).await?;
    conn.start_container(&name, None)
        .await
        .map_err(|e| Error::BuildError(e))
}
