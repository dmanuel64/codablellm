use indicatif::{HumanCount, ParallelProgressIterator};
use rayon::prelude::*;
use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    str::Utf8Error,
};
use thiserror::Error;
use tree_sitter::StreamingIterator;

use crate::{
    ProgressDisplay,
    function::{self, Function, ParsedFunctions},
    language::Language,
    parser::ParsedCode,
    repo::Repository,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to parse source code: {}",
    path.as_ref().map(|p| p.as_display().to_string()).unwrap_or_else(|| "<UNKNOWN>".into()))]
    Parse {
        path: Option<PathBuf>,
        #[source]
        source: Option<io::Error>,
    },
    #[error("failed to decode source code")]
    Decode(#[from] Utf8Error),
    #[error("failed to transform function '{}': {source}", function.name())]
    Transform {
        function: Function,
        source: anyhow::Error,
    },
    #[error("Failed to recognize language from source code file: {}",
    file.file_name().map(|n| n.to_string_lossy()).unwrap_or_else(|| "<UNKNOWN>".into()))]
    UnknownLanguage { file: PathBuf },
    #[error("failed to query S-expression")]
    Query(#[source] tree_sitter::QueryError),
}

pub enum Transform {
    Native(Box<dyn for<'a> Fn(&'a Function) -> anyhow::Result<MaybeChangedFunction<'a>> + Send>),
    #[cfg(feature = "rhai")]
    Rhai {
        file: PathBuf,
    },
}

pub type MaybeChangedFunction<'a> = Cow<'a, Function>;

pub trait MaybeChangedFunctionExt {
    fn is_changed(&self) -> bool;
}

impl MaybeChangedFunctionExt for MaybeChangedFunction<'_> {
    fn is_changed(&self) -> bool {
        matches!(self, Cow::Owned(_))
    }
}

impl Transform {
    pub fn apply<'a>(&self, function: &'a Function) -> Result<MaybeChangedFunction<'a>, Error> {
        match self {
            Transform::Native(f) => f(function),
            #[cfg(feature = "rhai")]
            Transform::Rhai { file } => {
                let mut engine = rhai::Engine::new();
                engine.register_type::<Function>();
                let mut scope = rhai::Scope::new();
                scope.push("function", function.clone());
                engine
                    .run_file_with_scope(&mut scope, file.clone())
                    .map(|_| {
                        let new_function = scope
                            .get_value_ref::<Function>("function")
                            .unwrap_or(function);
                        if new_function.definition() == function.definition() {
                            Cow::Borrowed(function)
                        } else {
                            Cow::Owned(new_function.clone())
                        }
                    })
                    .map_err(anyhow::Error::from)
            }
        }
        .map_err(|source| Error::Transform {
            function: function.clone(),
            source,
        })
    }
}

#[derive(Debug, Default)]
pub struct Options {
    pub progress_display: ProgressDisplay,
    pub headers_as_cpp: bool,
}

pub fn extract(repo: &Repository) -> Result<Vec<Function>, Error> {
    extract_with_options(repo, &Options::default())
}

pub fn extract_with_options(repo: &Repository, options: &Options) -> Result<Vec<Function>, Error> {
    extract_inner(repo, None, options)
}

pub fn transform(repo: &Repository, transform: &Transform) -> Result<Vec<Function>, Error> {
    transform_with_options(repo, transform, &Options::default())
}

pub fn transform_with_options(
    repo: &Repository,
    transform: &Transform,
    options: &Options,
) -> Result<Vec<Function>, Error> {
    extract_inner(repo, Some(transform), options)
}

fn extract_inner(
    repo: &Repository,
    transform: Option<&Transform>,
    Options {
        progress_display,
        headers_as_cpp,
    }: &Options,
) -> Result<Vec<Function>, Error> {
    let progress = progress_display.new_progress_bar(None);
    let source_files = repo.source_files();
    if let (_, Some(num_files)) = source_files.size_hint() {
        tracing::info!(
            "Will attempt to extract {} source code files",
            HumanCount(num_files as u64)
        )
    }
    repo.source_files()
        .par_bridge()
        .progress_with(progress)
        .map(|path| {
            let mut functions = Vec::new();
            tracing::debug!(file = %path.display(), "Extracting source code file");
            match extract_file(&path, transform, *headers_as_cpp) {
                Ok(f) => functions.extend(f),
                Err(error) => {
                    if let Error::UnknownLanguage { file } = error {
                        tracing::warn!(
                            file = %file.display(),
                            "Failed to recognize language from file"
                        );
                    } else {
                        tracing::warn!(?error, file = %path.display(), "Failed to extract source code functions");
                    }
                }
            }
            Ok(functions)
        }).flatten().collect()
}

fn extract_file(
    path: &Path,
    transform: Option<&Transform>,
    headers_as_cpp: bool,
) -> Result<Vec<Function>, Error> {
    let parsed_functions: ParsedFunctions = ParsedCode::try_from(path)?.into();
    if let Some(t) = transform {
        parsed_functions.edit_functions(|f| {
            let r = t.apply(f).unwrap();
            if r.is_changed() {}
        });
    }
    parsed_functions.functions()
}
