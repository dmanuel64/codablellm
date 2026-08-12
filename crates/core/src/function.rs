use std::{
    ffi::OsStr,
    fmt::Display,
    ops::Range,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::Language;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Function {
    // TODO: make these variants types so they have non-public access qualifiers
    Source {
        name: String,
        definition: String,
        language: Language,
        file: PathBuf,
        line_range: Range<usize>,
        column_range: Range<usize>,
        extra: Option<serde_json::Value>,
    },
    Decompiled {
        name: String,
        definition: String,
        binary: PathBuf,
        extra: Option<serde_json::Value>,
    },
    Assembly {
        name: String,
        definition: String,
        binary: PathBuf,
        extra: Option<serde_json::Value>,
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
            Function::Source { definition, .. } => definition,
            Function::Decompiled { definition, .. } => definition,
            Function::Assembly { definition, .. } => definition,
        }
    }

    pub fn definition_mut(&mut self) -> &mut str {
        match self {
            Function::Source { definition, .. } => definition,
            Function::Decompiled { definition, .. } => definition,
            Function::Assembly { definition, .. } => definition,
        }
    }

    pub fn source(&self) -> &Path {
        match self {
            Function::Source { file, .. } => file,
            Function::Decompiled { binary, .. } => binary,
            Function::Assembly { binary, .. } => binary,
        }
    }

    pub fn language(&self) -> String {
        match self {
            Function::Source { language, .. } => language.to_string(),
            Function::Decompiled { .. } => String::from("Pseudo-C (Decompiled Code)"),
            Function::Assembly { .. } => String::from("Assembly"),
        }
    }

    pub fn extra(&self) -> &Option<serde_json::Value> {
        match self {
            Function::Source { extra, .. } => extra,
            Function::Decompiled { extra, .. } => extra,
            Function::Assembly { extra, .. } => extra,
        }
    }

    pub fn extra_mut(&mut self) -> &mut Option<serde_json::Value> {
        match self {
            Function::Source { extra, .. } => extra,
            Function::Decompiled { extra, .. } => extra,
            Function::Assembly { extra, .. } => extra,
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}::{}",
            self.source()
                .file_name()
                .map(OsStr::to_string_lossy)
                .unwrap_or_else(|| String::from("<UNKNOWN>").into()),
            self.name()
        )
    }
}

#[cfg(feature = "rhai")]
impl rhai::CustomType for Function {
    fn build(mut builder: rhai::TypeBuilder<Self>) {
        builder
            .with_name("Function")
            .with_get("name", |func: &mut Self| func.name().to_string())
            .with_get("language", |func: &mut Self| func.language())
            .with_get_set(
                "definition",
                |func: &mut Self| func.definition().to_string(),
                |func: &mut Self, val: String| {
                    let definition = match func {
                        Function::Source { definition, .. } => definition,
                        Function::Decompiled { definition, .. } => definition,
                        Function::Assembly { definition, .. } => definition,
                    };
                    *definition = val;
                },
            );
    }
}
