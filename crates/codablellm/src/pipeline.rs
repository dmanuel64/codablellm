use crate::types::{errors::PipelineError, pipeline};

struct Manager {
    stage: pipeline::Stage,
}

pub fn run(repo_url: String, mode: pipeline::Mode) -> Result<(), PipelineError> {
    run_with_options(&pipeline::Options { repo_url, mode })
}

pub fn run_with_options(options: &pipeline::Options) -> Result<(), PipelineError> {
    todo!()
}
