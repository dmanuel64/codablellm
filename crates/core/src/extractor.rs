use std::{cell::RefCell, fs, io, path::Path, rc::Rc};

use derive_more::Debug;
use globwalk::GlobWalkerBuilder;
use thiserror::Error;

#[cfg(feature = "c")]
use crate::source::C;
use crate::source::{self, ParseOptions};

// TODO: could maybe lazy-lock this, but too lazy to figure out how to make it thread-safe
thread_local! {
    pub static DEFAULT_LOADERS: RefCell<Vec<Loader>> = RefCell::new(
        vec![
            #[cfg(feature = "c")]
            Loader::new::<C>(false),
        ]
    );
}

pub type LoaderFn = dyn Fn(&Path) -> Option<Result<source::CodeFile, LoaderError>>;

#[derive(Debug, Clone)]
pub struct Loader {
    language: &'static str,
    extensions: &'static [&'static str],
    #[debug(skip)]
    inner: Rc<Box<LoaderFn>>,
    pub strict: bool,
}

impl Loader {
    pub fn new<L: source::Language>(strict: bool) -> Self {
        Self {
            language: L::NAME,
            extensions: L::FILE_EXTENSIONS,
            inner: Rc::new(Box::new(|path| {
                source::is_file::<L, _>(path).then(|| {
                    Ok(source::CodeFile {
                        path: path.to_path_buf(),
                        code: source::Code::<L>::parse(&fs::read_to_string(path)?)
                            .unwrap()
                            .into(),
                    })
                })
            })),
            strict,
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
}

#[derive(Debug)]
pub struct Options {
    pub loaders: Vec<Loader>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            loaders: DEFAULT_LOADERS.with_borrow(|loaders| loaders.iter().cloned().collect()),
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
    P: AsRef<Path>,
{
    extract_with_options(path, &Options::default())
}

pub fn extract_with_options<P>(
    path: &P,
    options: &Options,
) -> Result<Vec<Result<source::CodeFile, LoaderError>>, Error>
where
    P: AsRef<Path>,
{
    // Create glob pattern
    let extensions: Vec<_> = options
        .loaders
        .iter()
        .flat_map(|loader| loader.extensions)
        .copied()
        .collect();
    let pattern = format!("**/*.{{{}}}", extensions.join(","));

    // Walk all matched files
    let walker = GlobWalkerBuilder::from_patterns(path, &[pattern]).build()?;
    Ok(walker
        .into_iter()
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
}

fn extract_file(path: &Path, options: &Options) -> Option<Result<source::CodeFile, LoaderError>> {
    options.loaders.iter().find_map(|loader| loader.load(path))
}
