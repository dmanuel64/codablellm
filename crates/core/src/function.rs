use std::{ops::Range, path::PathBuf};

use crate::Language;

pub enum Function {
    Source {
        name: String,
        definition: String,
        language: Language,
        file: PathBuf,
        line_range: Range<usize>,
        column_range: Range<usize>,
    },
    Decompiled {
        name: String,
        definition: String,
        binary: PathBuf,
    },
    Assembly {
        name: String,
        definition: String,
        binary: PathBuf,
    },
}

impl Function {
    pub fn name(&self) -> &str {
        match self {
            Function::Source { name, .. } => &name,
            Function::Decompiled { name, .. } => &name,
            Function::Assembly { name, .. } => &name,
        }
    }

    pub fn definition(&self) -> &str {
        match self {
            Function::Source { definition, .. } => &definition,
            Function::Decompiled { definition, .. } => &definition,
            Function::Assembly { definition, .. } => &definition,
        }
    }
}
