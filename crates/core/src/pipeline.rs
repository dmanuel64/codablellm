use std::{
    path::{Path, PathBuf},
    sync::{LazyLock, Mutex},
};

use directories::ProjectDirs;
use statig::prelude::*;

use crate::{extractor, source};

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
pub struct DatasetGenerator;

#[derive(Debug)]
pub enum Event {
    ExtractSourceCode {
        path: PathBuf,
        options: extractor::Options,
    },
    BuildDataset {
        path: PathBuf,
    },
}

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
            Event::ExtractSourceCode { path, options } => {
                let source_files = extractor::extract_with_options(path, options)
                    .inspect_err(|error| tracing::error!(%error, "Failed to extract repository"))
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|source_file| {
                        source_file
                            .inspect_err(
                                |error| tracing::error!(%error, "Failed to load source code file"),
                            )
                            .ok()
                    })
                    .collect();
                Outcome::Transition(State::BuildDataset { source_files })
            }
            _ => Super,
        }
    }

    #[state(entry_action = "enter_build_dataset")]
    fn build_dataset(source_files: &Vec<source::CodeFile>, event: &Event) -> Outcome<State> {
        Outcome::Handled
    }

    #[action]
    fn enter_build_dataset(source_files: &Vec<source::CodeFile>) {}
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

pub fn run(path: PathBuf) {
    run_with_options(path, Options::default());
}

pub fn run_with_options(path: PathBuf, options: Options) {
    let mut generator = DatasetGenerator::default().state_machine();
    generator.handle(&Event::ExtractSourceCode {
        path,
        options: options.extractor,
    });
    // generator.handle(&Event::BuildDataset { path });
}
