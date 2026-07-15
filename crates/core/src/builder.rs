use bollard::{
    Docker, body_full,
    plugin::ContainerCreateBody,
    query_parameters::{BuildImageOptionsBuilder, CreateContainerOptionsBuilder},
};
use futures_util::TryStreamExt;
use std::path::Path;
use thiserror::Error;

use crate::repo::Repository;

const DOCKERFILE_SOURCE: &str = include_str!("../assets/builder.Dockerfile");
const BUILDER_IMAGE_TAG: &str = "codablellm-builder:latest";

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to create tarball containing builder Dockerfile")]
    TarballError(#[source] std::io::Error),
    #[error("failed to connect to Docker")]
    DockerConnectionError(#[source] bollard::errors::Error),
    #[error("failed to create builder container")]
    BuilderCreationError(#[source] bollard::errors::Error),
    #[error("failed to run build command: {0}")]
    BuildError(#[source] bollard::errors::Error),
}

async fn create_builder_image(conn: &Docker) -> Result<(), Error> {
    let mut builder = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header
        .set_path("Dockerfile")
        .map_err(|e| Error::TarballError(e))?;
    header.set_size(DOCKERFILE_SOURCE.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder
        .append(&header, DOCKERFILE_SOURCE.as_bytes())
        .map_err(|e| Error::TarballError(e))?;
    let tar_bytes = builder.into_inner().map_err(|e| Error::TarballError(e))?;

    let options = BuildImageOptionsBuilder::default()
        .dockerfile("Dockerfile")
        .t(BUILDER_IMAGE_TAG)
        .rm(true)
        .build();

    _ = conn.build_image(options, None, Some(body_full(tar_bytes.into())));
    Ok(())
}

async fn create_builder_container(conn: &Docker) -> Result<String, Error> {
    let name = "codablellm-builder";
    let options = CreateContainerOptionsBuilder::default().name(name).build();
    let config = ContainerCreateBody {
        ..Default::default()
    };
    conn.create_container(Some(options), config)
        .await
        .map_err(|e| Error::BuilderCreationError(e))?;
    Ok(name.to_string())
}

/// Builds a repository in a builder container given a build command, and paths to where the expected build artifacts reside
pub async fn build(repo: &Repository, commands: &[&str], artifacts: &[&Path]) -> Result<(), Error> {
    let conn = Docker::connect_with_defaults().map_err(|e| Error::DockerConnectionError(e))?;
    create_builder_image(&conn).await?;
    let name = create_builder_container(&conn).await?;
    conn.start_container(&name, None)
        .await
        .map_err(|e| Error::BuildError(e))
}
