use std::{fs, path::Path};

use crate::source;
#[cfg(feature = "c")]
use crate::source::C;

pub type Loader = dyn Fn(&Path) -> Option<source::CodeFile>;

pub fn load<L>(path: &Path) -> Option<source::CodeFile>
where
    L: source::Language,
{
    source::is_file::<L, _>(path).then(|| source::CodeFile {
        path: path.to_path_buf(),
        code: source::Code::<L>::new(&fs::read_to_string(path).unwrap())
            .unwrap()
            .into(),
    })
}

pub const DEFAULT_LOADERS: &[fn(&Path) -> Option<source::CodeFile>] = &[
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
}

pub fn extract<P>(path: &P) -> Vec<source::CodeFile>
where
    P: AsRef<Path>,
{
    extract_with_options(path, &Options::default())
}

pub fn extract_with_options<P>(path: &P, options: &Options) -> Vec<source::CodeFile>
where
    P: AsRef<Path>,
{
    vec![]
}

fn extract_file(path: &Path, options: &Options) -> source::CodeFile {
    todo!()
}
