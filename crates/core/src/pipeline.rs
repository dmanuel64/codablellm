use std::{
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use directories::ProjectDirs;
use indicatif::MultiProgress;
use statig::prelude::*;

use crate::extractor;

pub static STATE_DIR: LazyLock<Mutex<PathBuf>> = LazyLock::new(|| {
    let dirs = ProjectDirs::from("io.github", "dmanuel64", "codablellm")
        .expect("a home directory to be found on the host system");
    Mutex::new(
        dirs.state_dir()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| dirs.data_local_dir().join("state")),
    )
});

#[derive(Debug, Default)]
pub struct DatasetGenerator {
    progress: MultiProgress,
}

#[derive(Debug)]
pub enum Event {}

#[state_machine(
    initial = "State::extract_source_code()",
    after_transition = "Self::after_transition",
    state(derive(Debug)),
    superstate(derive(Debug))
)]
impl DatasetGenerator {
    #[state]
    fn extract_source_code(event: &Event) -> Outcome<State> {
        match event {
            _ => Super,
        }
    }

    #[state]
    fn build_dataset(event: &Event) -> Outcome<State> {
        todo!()
    }
}

impl DatasetGenerator {
    fn after_transition(&mut self, source: &State, target: &State, _context: &mut ()) {
        tracing::debug!(?source, ?target, "Dataset generator state updated");
    }
}

#[derive(Default)]
pub struct Options {
    pub extractor: extractor::Options,
}

pub fn run() {
    let mut generator = DatasetGenerator::default().state_machine();
}
