mod c;
mod function;

use std::{ops::Range, path::PathBuf};

use serde::{Deserialize, Serialize};

pub use function::{AssociatedFunction, Function, Method};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Descriptor {
    pub name: String,
    pub definition: String,
    pub span: Option<Span>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub source: PathBuf,
    #[serde(skip)]
    pub bytes_range: Range<usize>,
    pub line_range: Range<usize>,
    pub column_range: Range<usize>,
}

#[typetag::serde(tag = "type")]
pub trait Subroutine {
    fn descriptor(&self) -> &Descriptor;

    fn name(&self) -> &str {
        &self.descriptor().name
    }

    fn definition(&self) -> &str {
        &self.descriptor().definition
    }
}
