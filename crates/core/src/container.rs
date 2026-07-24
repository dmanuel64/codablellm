use bollard::{
    Docker,
    plugin::ContainerCreateBody,
    query_parameters::{CreateContainerOptionsBuilder, CreateImageOptionsBuilder},
};
use futures_util::StreamExt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to connect to Docker")]
    Connection(#[source] bollard::errors::Error),
    #[error("failed to pull image")]
    ImagePull(#[source] bollard::errors::Error),
    #[error("failed to create container")]
    ContainerCreation(#[source] bollard::errors::Error),
}

pub fn connect_runtime() -> Result<Docker, Error> {
    Docker::connect_with_defaults().map_err(Error::Connection)
}

/// Resolves the given image, pulling it from the registry if it isn't
/// already present locally.
pub async fn pull_image(conn: &Docker, image: &str, platform: &str) -> Result<String, Error> {
    if conn.inspect_image(image).await.is_ok() {
        return Ok(image.to_string());
    }
    let options = CreateImageOptionsBuilder::default()
        .from_image(image)
        .platform(platform)
        .build();
    let mut pull = conn.create_image(Some(options), None, None);
    while let Some(result) = pull.next().await {
        result.map_err(Error::ImagePull)?;
    }
    Ok(image.to_string())
}

pub async fn create(conn: &Docker, name: &str, image: &str) -> Result<String, Error> {
    let options = CreateContainerOptionsBuilder::default().name(name).build();
    let config = ContainerCreateBody {
        image: Some(image.to_string()),
        ..Default::default()
    };
    conn.create_container(Some(options), config)
        .await
        .map_err(Error::ContainerCreation)?;
    Ok(name.to_string())
}
