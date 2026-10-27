use std::{
    fs, io,
    num::NonZero,
    path::Path,
    sync::{Arc, LazyLock, RwLock},
};

use derive_more::Debug;
use globwalk::GlobWalkerBuilder;
use rayon::{ThreadPoolBuilder, prelude::*};
use thiserror::Error;

#[cfg(feature = "c")]
use crate::source::C;
use crate::source::{self, ParseOptions};

pub static DEFAULT_LOADERS: LazyLock<RwLock<Vec<Loader>>> = LazyLock::new(|| {
    RwLock::new(vec![
        #[cfg(feature = "c")]
        Loader::new::<C>(),
    ])
});

pub type LoaderFn = dyn Fn(&Path) -> Option<Result<source::CodeFile, LoaderError>> + Send + Sync;

#[derive(Debug, Clone, Default)]
pub struct LoaderOptions {
    pub parse: ParseOptions,
}

#[derive(Debug, Clone)]
pub struct Loader {
    language: &'static str,
    extensions: &'static [&'static str],
    #[debug(skip)]
    inner: Arc<LoaderFn>,
    pub options: LoaderOptions,
}

impl Loader {
    pub fn new<L: source::Language>() -> Self {
        Self::new_with_options::<L>(LoaderOptions::default())
    }

    pub fn new_with_options<L: source::Language>(options: LoaderOptions) -> Self {
        Self {
            language: L::NAME,
            extensions: L::FILE_EXTENSIONS,
            inner: Arc::new(|path| {
                source::is_file::<L, _>(path).then(|| {
                    Ok(source::CodeFile {
                        path: path.to_path_buf(),
                        code: source::Code::<L>::parse(&fs::read_to_string(path)?)
                            .unwrap()
                            .into(),
                    })
                })
            }),
            options,
        }
    }

    pub fn load(&self, path: impl AsRef<Path>) -> Option<Result<source::CodeFile, LoaderError>> {
        let load = self.inner.as_ref();
        load(path.as_ref())
    }
}

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("TODO: message")]
    Io(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("TODO: message")]
    Io(#[from] io::Error),
    #[error("TODO: message")]
    Glob(#[from] globwalk::GlobError),
    #[error("TODO: message")]
    ThreadPool(#[from] rayon::ThreadPoolBuildError),
}

#[derive(Debug)]
pub struct Options {
    pub loaders: Vec<Loader>,
    pub max_jobs: Option<NonZero<usize>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            loaders: DEFAULT_LOADERS.read().unwrap().to_vec(),
            max_jobs: None,
        }
    }
}

impl Options {
    pub fn with_custom_loaders(&mut self, loaders: Vec<Loader>) -> &mut Self {
        self.loaders.extend(loaders);
        self
    }

    #[cfg(all(feature = "c", feature = "cpp"))]
    pub fn with_h_files_as_cpp(&mut self) -> &mut Self {
        use crate::source::Language;

        static NEW_C_FILE_EXTENSIONS: &'static [&'static str] = &[".c"];

        self.loaders.iter_mut().find_map(|loader| {
            (loader.language == C::NAME).then(|| loader.extensions = NEW_C_FILE_EXTENSIONS)
        });
        self
    }
}

pub fn extract<P>(path: &P) -> Result<Vec<Result<source::CodeFile, LoaderError>>, Error>
where
    P: AsRef<Path> + Sync,
{
    extract_with_options(path, &Options::default())
}

pub fn extract_with_options<P>(
    path: &P,
    options: &Options,
) -> Result<Vec<Result<source::CodeFile, LoaderError>>, Error>
where
    P: AsRef<Path> + Sync,
{
    // Create glob pattern
    let extensions: Vec<_> = options
        .loaders
        .iter()
        .flat_map(|loader| loader.extensions)
        .copied()
        .collect();
    let pattern = format!("**/*.{{{}}}", extensions.join(","));

    let extract_dir = || {
        // Walk all matched files
        let walker = GlobWalkerBuilder::from_patterns(path, &[pattern]).build()?;
        Ok(walker
            .par_bridge()
            .filter_map(|entry| {
                let entry = entry.unwrap();
                let path = entry.path();
                let source_file = extract_file(path, options);
                if source_file.is_none() {
                    tracing::warn!(
                        file = path.display().to_string(),
                        "Could determine loader for source code file"
                    );
                }
                source_file
            })
            .collect())
    };

    if let Some(max_jobs) = options.max_jobs {
        // Create thread pool with specified number of threads
        ThreadPoolBuilder::new()
            .num_threads(max_jobs.get())
            .build()?
            .install(extract_dir)
    } else {
        // Use global thread pool
        extract_dir()
    }
}

fn extract_file(path: &Path, options: &Options) -> Option<Result<source::CodeFile, LoaderError>> {
    options.loaders.iter().find_map(|loader| loader.load(path))
}
