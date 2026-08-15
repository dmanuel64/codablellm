use std::{
    ffi::OsStr,
    fmt::Display,
    ops::Range,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::Language;

// TODO: metadata isn't the most accurate name to include name & definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metadata {
    name: String,
    definition: String,
    extra: Option<serde_json::Value>,
    source: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    #[serde(skip)]
    bytes_range: Range<usize>,
    line_range: Range<usize>,
    column_range: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Function {
    // TODO: make these variants types so they have non-public access qualifiers
    Source {
        metadata: Metadata,
        language: Language,
        location: Location,
    },
    Decompiled {
        metadata: Metadata,
    },
    Assembly {
        metadata: Metadata,
        location: Location,
    },
}

impl Function {
    pub fn new_source(
        name: String,
        definition: String,
        source: Option<PathBuf>,
        language: Language,
        bytes_range: Range<usize>,
        line_range: Range<usize>,
        column_range: Range<usize>,
    ) -> Self {
        Self::Source {
            metadata: Metadata {
                name,
                definition,
                extra: None,
                source,
            },
            language,
            location: Location {
                bytes_range,
                line_range,
                column_range,
            },
        }
    }

    pub fn new_assembly(
        name: String,
        definition: String,
        source: Option<PathBuf>,
        bytes_range: Range<usize>,
        line_range: Range<usize>,
        column_range: Range<usize>,
    ) -> Self {
        Self::Assembly {
            metadata: Metadata {
                name,
                definition,
                extra: None,
                source,
            },
            location: Location {
                bytes_range,
                line_range,
                column_range,
            },
        }
    }

    pub fn new_decompiled(name: String, definition: String, source: Option<PathBuf>) -> Self {
        Self::Decompiled {
            metadata: Metadata {
                name,
                definition,
                extra: None,
                source,
            },
        }
    }

    pub fn metadata(&self) -> &Metadata {
        match self {
            Function::Source { metadata, .. } => metadata,
            Function::Decompiled { metadata, .. } => metadata,
            Function::Assembly { metadata, .. } => metadata,
        }
    }

    fn metadata_mut(&mut self) -> &mut Metadata {
        match self {
            Function::Source { metadata, .. } => metadata,
            Function::Decompiled { metadata, .. } => metadata,
            Function::Assembly { metadata, .. } => metadata,
        }
    }

    pub fn name(&self) -> &str {
        &self.metadata().name
    }

    pub fn definition(&self) -> &str {
        &self.metadata().definition
    }

    pub fn source(&self) -> Option<&Path> {
        self.metadata().source.as_ref().map(PathBuf::as_path)
    }

    pub fn language(&self) -> String {
        match self {
            Function::Source { language, .. } => language.to_string(),
            Function::Decompiled { .. } => String::from("C"),
            Function::Assembly { .. } => String::from("Assembly"),
        }
    }

    pub fn extra(&self) -> &Option<serde_json::Value> {
        &self.metadata().extra
    }

    pub fn extra_mut(&mut self) -> &mut Option<serde_json::Value> {
        &mut self.metadata_mut().extra
    }

    pub fn edit(&mut self, new_name: Option<String>, new_definition: String) {
        match self {
            Function::Source {
                metadata, location, ..
            }
            | Function::Assembly { metadata, location } => {
                let Metadata {
                    name, definition, ..
                } = metadata;
                let Location {
                    bytes_range,
                    line_range,
                    column_range,
                } = location;
                line_range.end = line_range.start + new_definition.lines().count();
                column_range.end = new_definition
                    .lines()
                    .map(str::len)
                    .max()
                    .unwrap_or_default();
                bytes_range.end = bytes_range.start + new_definition.bytes().len();
                if let Some(n) = new_name {
                    *name = n;
                }
                *definition = new_definition;
            }
            Function::Decompiled { metadata } => todo!("support for decompiled code edits"),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}::{}",
            self.source()
                .and_then(Path::file_name)
                .map(OsStr::to_string_lossy)
                .unwrap_or_else(|| String::from("<LOCAL>").into()),
            self.name()
        )
    }
}

#[cfg(feature = "rhai")]
impl rhai::CustomType for Function {
    fn build(mut builder: rhai::TypeBuilder<Self>) {
        builder
            .with_name("Function")
            .with_get_set(
                "name",
                |func: &mut Self| func.name().to_string(),
                |func: &mut Self, val: String| {
                    func.metadata_mut().name = val;
                },
            )
            .with_get("language", |func: &mut Self| func.language())
            .with_get_set(
                "definition",
                |func: &mut Self| func.definition().to_string(),
                |func: &mut Self, val: String| {
                    func.edit(None, val);
                },
            );
    }
}
