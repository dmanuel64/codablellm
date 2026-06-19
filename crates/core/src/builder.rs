use futures_util::{TryStreamExt};
use std::path::Path;
use bollard::{Docker, body_full, query_parameters::BuildImageOptionsBuilder};
use thiserror::Error;

use crate::repo::Repo;

const DOCKERFILE_SOURCE: &str = include_str!("../assets/builder.Dockerfile");
const BUILDER_IMAGE_TAG: &str = "codablellm-builder:latest";

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to build")]
    Build,
}

async fn create_builder(conn: &Docker) -> Result<(), bollard::errors::Error> {
    let mut builder = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_path("Dockerfile")?;
    header.set_size(DOCKERFILE_SOURCE.len() as u64);
    header.set_mode(0o644);
    header.set_cksum();
    builder.append(&header, DOCKERFILE_SOURCE.as_bytes())?;
    let tar_bytes = builder.into_inner()?;

    let options = BuildImageOptionsBuilder::default()
        .dockerfile("Dockerfile")
        .t(BUILDER_IMAGE_TAG)
        .rm(true)
        .build();

    let mut stream = conn.build_image(options, None, Some(body_full(tar_bytes.into())));
    while let Some(msg) = stream.try_next().await? {
        if let Some(line) = msg.stream {
            print!("{line}");
        }
    }
    Ok(())
}

pub async fn build(repo: &Repo, command: &str, artifacts: &[&Path]) -> Result<(), Error> {
    let conn = Docker::connect_with_defaults()?;
    let tar_bytes = bu
    conn.build_image(options, credentials, tar)
}
