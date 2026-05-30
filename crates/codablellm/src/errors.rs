use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid transition from stage `{stage}` with event `{event}`")]
    InvalidTransition { stage: String, event: String },
    #[error("failed to fetch repo: {0}")]
    Repo(#[from] reqwest::Error),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}
