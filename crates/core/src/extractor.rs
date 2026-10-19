use std::{fs, io, iter, path::Path};

use thiserror::Error;

use crate::source;
#[cfg(feature = "c")]
use crate::source::C;

pub type Loader = dyn Fn(&Path) -> Option<Result<source::CodeFile, LoaderError>>;

#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("TODO: message")]
    Io(#[from] io::Error),
}

pub fn load<L>(path: &Path) -> Option<Result<source::CodeFile, LoaderError>>
where
    L: source::Language,
{
    source::is_file::<L, _>(path).then(|| {
        Ok(source::CodeFile {
            path: path.to_path_buf(),
            code: source::Code::<L>::new(&fs::read_to_string(path)?)
                .unwrap()
                .into(),
        })
    })
}

pub const DEFAULT_LOADERS: &[fn(&Path) -> Option<Result<source::CodeFile, LoaderError>>] = &[
    #[cfg(feature = "c")]
    load::<C>,
];

pub struct Options {
    pub loaders: Vec<Box<Loader>>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            loaders: DEFAULT_LOADERS
                .iter()
                .map(|b| Box::new(b) as Box<Loader>)
                .collect(),
        }
    }
}

impl Options {
    pub fn with_custom_loaders(mut self, loaders: Vec<Box<Loader>>) -> Self {
        self.loaders.extend(loaders);
        self
    }

    #[cfg(all(feature = "c", feature = "cpp"))]
    pub fn with_h_files_as_cpp(mut self) -> Self {
        todo!()
    }
}

pub fn extract<P>(path: &P) -> Result<Vec<Result<source::CodeFile, LoaderError>>, io::Error>
where
    P: AsRef<Path>,
{
    extract_with_options(path, &Options::default())
}

pub fn extract_with_options<P>(
    path: &P,
    options: &Options,
) -> Result<Vec<Result<source::CodeFile, LoaderError>>, io::Error>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();
    if path.is_dir() {
        Ok(path
            .read_dir()?
            .map(|entry| extract_with_options(&entry?.path(), options))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect())
    } else {
        Ok(extract_file(path, options)
            .map(|source_file| iter::once(source_file).collect())
            .unwrap_or_else(|| iter::empty().collect()))
    }
}

fn extract_file(path: &Path, options: &Options) -> Option<Result<source::CodeFile, LoaderError>> {
    options.loaders.iter().find_map(|loader| loader(path))
}
