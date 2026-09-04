use std::{any::Any, ffi::OsStr, path::Path};

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};

#[typetag::serde(tag = "language")]
pub trait Language: std::fmt::Debug + Any + Send + Sync + DynClone {
    fn name(&self) -> &str;
    fn file_extensions(&self) -> Vec<&str>;

    fn is_source_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(OsStr::to_str) {
            self.file_extensions()
                .iter()
                .find(|e| e.contains(ext))
                .is_some()
        } else {
            false
        }
    }
}
dyn_clone::clone_trait_object!(Language);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct C;

#[typetag::serde]
impl Language for C {
    fn name(&self) -> &str {
        "C"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["c", "h"]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Cpp {
    pub include_h_files: bool,
}

#[typetag::serde]
impl Language for Cpp {
    fn name(&self) -> &str {
        "C++"
    }

    fn file_extensions(&self) -> Vec<&str> {
        let mut exts = vec!["cpp", "cxx", "cc", "c++", "hpp", "hxx", "hh", "h++"];
        if self.include_h_files {
            exts.push("h");
        }
        exts
    }
}

impl Default for Cpp {
    fn default() -> Self {
        Self {
            include_h_files: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Python;

#[typetag::serde]
impl Language for Python {
    fn name(&self) -> &str {
        "Python"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["py", "pyw"]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JavaScript {
    pub include_jsx_files: bool,
    pub include_mjs_files: bool,
    pub include_cjs_files: bool,
}

#[typetag::serde]
impl Language for JavaScript {
    fn name(&self) -> &str {
        "JavaScript"
    }

    fn file_extensions(&self) -> Vec<&str> {
        let mut exts = vec!["js"];
        if self.include_jsx_files {
            exts.push("jsx");
        }
        if self.include_mjs_files {
            exts.push("mjs");
        }
        if self.include_cjs_files {
            exts.push("cjs");
        }
        exts
    }
}

impl Default for JavaScript {
    fn default() -> Self {
        Self {
            include_jsx_files: true,
            include_mjs_files: true,
            include_cjs_files: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TypeScript {
    pub include_tsx: bool,
}

#[typetag::serde]
impl Language for TypeScript {
    fn name(&self) -> &str {
        "TypeScript"
    }

    fn file_extensions(&self) -> Vec<&str> {
        let mut exts = vec!["ts"];
        if self.include_tsx {
            exts.push("tsx");
        }
        exts
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Go;

#[typetag::serde]
impl Language for Go {
    fn name(&self) -> &str {
        "Go"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["go"]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rust;

#[typetag::serde]
impl Language for Rust {
    fn name(&self) -> &str {
        "Rust"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["rs"]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Java;

#[typetag::serde]
impl Language for Java {
    fn name(&self) -> &str {
        "Java"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["java"]
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CSharp;

#[typetag::serde]
impl Language for CSharp {
    fn name(&self) -> &str {
        "C#"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["cs"]
    }
}
