use thiserror::Error;

use crate::types::pipeline;

#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("invalid transition from stage `{stage:?}` with event `{event:?}`")]
    InvalidTransition {
        stage: pipeline::Stage,
        event: pipeline::Event,
    },
}
