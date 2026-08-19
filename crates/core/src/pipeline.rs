use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

use indicatif::{MultiProgress, ProgressBar};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumCount, EnumIter, IntoEnumIterator};
use tempfile::{NamedTempFile, env::temp_dir};
use thiserror::Error;
use tokio::fs;
use tracing::instrument;

use crate::{
    ProgressDisplay, builder,
    dataset::{self, Dataset},
    decompiler, extractor,
    function::Function,
    language, mapper,
    repo::{self, Repository},
};

#[derive(Debug, Error)]
pub enum Error {
    // TODO: this is somewhat confusing with repo::Error::Io whenever a directory is not found
    // also - should this stay here or in repo::Error?
    #[error("Invalid repository path: {0}")]
    InvalidRepoPath(PathBuf),
    #[error(transparent)]
    Repository(#[from] repo::Error),
    #[error(transparent)]
    Extractor(#[from] extractor::Error),
    #[error(transparent)]
    Builder(#[from] builder::Error),
    #[error(transparent)]
    Io(#[from] io::Error),
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, Display, Default, EnumCount, EnumIter)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    #[default]
    Loading,
    ExtractSourceCode,
    BuildCode,
    DecompileBinaries,
    MapCode,
    CreateDataset,
}

impl Stage {
    pub fn is_binary_stage(&self) -> bool {
        match self {
            Stage::Loading | Stage::ExtractSourceCode | Stage::CreateDataset => false,
            Stage::BuildCode | Stage::DecompileBinaries | Stage::MapCode => true,
        }
    }

    pub fn iter_source() -> impl Iterator<Item = Stage> {
        Self::iter().filter(|s| !s.is_binary_stage())
    }
}

#[derive(Debug, Clone)]
pub struct BinaryMode {
    pub build_commands: Vec<String>,
    pub binaries: Vec<PathBuf>,
    pub strip: bool,
    pub decompilers: HashSet<String>,
}

#[derive(Debug, Clone)]
pub enum Mode {
    SourceOnly,
    SourceAndBinary(BinaryMode),
}

impl Mode {
    pub fn as_binary_mode(&self) -> BinaryMode {
        let Mode::SourceAndBinary(mode) = self else {
            unreachable!("expected SourceAndBinary mode during a binary stage")
        };
        mode.clone()
    }
}

#[derive(Debug)]
pub struct Options<'a> {
    pub display_progress: bool,
    pub dry_run: bool,
    pub cache_dir: Option<&'a Path>,
    pub save_transformed_repos: bool,
    pub transform_in_place: bool,
    pub state_dir: Option<&'a Path>,
    pub save_state: bool,
    pub extractor_options: extractor::Options,
    pub decompiler_options: decompiler::Options,
    pub mapper_options: mapper::Options,
    pub dataset_options: dataset::Options,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            display_progress: true,
            dry_run: false,
            cache_dir: None,
            state_dir: None,
            extractor_options: extractor::Options::default(),
            decompiler_options: decompiler::Options::default(),
            mapper_options: mapper::Options::default(),
            dataset_options: dataset::Options::default(),
            save_transformed_repos: false,
            save_state: true,
            transform_in_place: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct State {
    stage: Stage,
    repo: Option<Repository>,
    extracted_functions: Option<Vec<Function>>,
}

pub async fn run<PathLike>(path: PathLike, mode: Mode) -> Result<dataset::Dataset, Error>
where
    PathLike: AsRef<Path> + std::fmt::Debug,
{
    run_with_options(path, mode, &Options::default()).await
}

#[instrument(name = "pipeline", skip(options))]
pub async fn run_with_options<PathLike>(
    path: PathLike,
    mode: Mode,
    options: &Options<'_>,
) -> Result<dataset::Dataset, Error>
where
    PathLike: AsRef<Path> + std::fmt::Debug,
{
    tracing::trace!(?options);
    let repo_name = path
        .as_ref()
        .file_name()
        .map(|s| s.to_string_lossy())
        .ok_or_else(|| Error::InvalidRepoPath(path.as_ref().to_path_buf()))?;
    let state_file = options
        .state_dir
        .map_or_else(NamedTempFile::new, NamedTempFile::new_in)?;
    let state_path = if options.save_state {
        let (_, p) = state_file.keep().expect("TODO make error");
        p
    } else {
        state_file.path().to_path_buf()
    };
    let mut state = State::default();

    // Create progress bars
    let overall_progress = Arc::new(MultiProgress::new());
    let stages_progress = if options.display_progress {
        ProgressBar::no_length()
    } else {
        ProgressBar::hidden()
    };
    overall_progress.add(stages_progress.clone());

    // Iterate through states
    let stages = if matches!(mode, Mode::SourceOnly) {
        Box::new(Stage::iter_source()) as Box<dyn Iterator<Item = Stage>>
    } else {
        Box::new(Stage::iter())
    };
    for stage in stages_progress.wrap_iter(stages) {
        match stage {
            Stage::Loading => {
                tracing::info!("Loading repository '{repo_name}'...",);
                state.repo = Some(Repository::new(path.as_ref().to_path_buf())?);
            }
            Stage::ExtractSourceCode => {
                tracing::info!("Extracting source code functions...");
                state.extracted_functions = Some(extractor::extract_with_options(
                    state.repo.as_ref().expect("repo to be initialized"),
                    &extractor::Options {
                        progress_display: ProgressDisplay::Nested(overall_progress.clone()),
                        ..Default::default()
                    },
                )?);
            }
            Stage::BuildCode => todo!(),
            Stage::DecompileBinaries => todo!(),
            Stage::MapCode => todo!(),
            Stage::CreateDataset => todo!(),
        }
        // Save current pipeline state
        tracing::debug!(file = ?state_path, "Saving current pipeline state");
        tracing::trace!(?state);

        match serde_json::to_string(&state) {
            Ok(state_json) => {
                if let Err(error) = fs::write(&state_path, state_json).await {
                    tracing::warn!(?error, file = %state_path.display(), "Failed to save pipeline state");
                }
            }
            Err(error) => {
                tracing::warn!(?error, file = %state_path.display(), "Failed to serialize pipeline state");
            }
        }
    }
    if let Err(error) = fs::remove_file(state_path).await {
        tracing::warn!(?error, "Failed to remove pipeline state")
    }
    todo!()
    //Manager::new(path, mode, options).run().await
}
