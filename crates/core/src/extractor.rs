use indicatif::{HumanCount, ParallelProgressIterator};
use rayon::prelude::*;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::{
    ProgressDisplay,
    function::{Function, ParsedFunctions},
    parser::{self, ParsedCode},
    repo::Repository,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Parser(#[from] parser::Error),
    #[error("failed to transform function '{}': {source}", function.name())]
    Transform {
        function: Function,
        source: anyhow::Error,
    },
}

pub enum Transform {
    Native(Box<dyn for<'a> Fn(&'a Function) -> anyhow::Result<MaybeChangedFunction<'a>> + Send + Sync>),
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
    let functions = repo
        .source_files()
        .par_bridge()
        .progress_with(progress)
        .map(|path| {
            tracing::debug!(file = %path.display(), "Extracting source code file");
            match extract_file(&path, transform, *headers_as_cpp) {
                Ok(f) => f,
                Err(Error::Parser(parser::Error::UnknownLanguage { file })) => {
                    tracing::warn!(
                        file = %file.display(),
                        "Failed to recognize language from file"
                    );
                    Vec::new()
                }
                Err(error) => {
                    tracing::warn!(?error, file = %path.display(), "Failed to extract source code functions");
                    Vec::new()
                }
            }
        })
        .flatten()
        .collect();
    Ok(functions)
}

fn extract_file(
    path: &Path,
    transform: Option<&Transform>,
    // TODO: wire this into ParsedCode::new so ambiguous .h files can be
    // parsed as C++ instead of C.
    _headers_as_cpp: bool,
) -> Result<Vec<Function>, Error> {
    let mut parsed_functions: ParsedFunctions = ParsedCode::try_from(path)?.into();
    if let Some(t) = transform {
        parsed_functions.edit(|f| match t.apply(f) {
            Ok(new_function) if new_function.is_changed() => {
                let new_name = new_function.name().to_string();
                let new_definition = new_function.definition().to_string();
                f.edit(Some(new_name), new_definition);
            }
            Ok(_) => {}
            Err(error) => {
                tracing::warn!(?error, function = %f, "Failed to apply transform to function");
            }
        })?;
    }
    Ok(parsed_functions.functions()?.to_vec())
}
