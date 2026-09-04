use std::{any::Any, ffi::OsStr, path::Path};

pub trait Language: Any + Send + Sync {
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

#[derive(Debug, Clone, Copy)]
pub struct C;

impl Language for C {
    fn name(&self) -> &str {
        "C"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["c", "h"]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Cpp {
    pub include_h_files: bool,
}

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

#[derive(Debug, Clone, Copy)]
pub struct Python;

impl Language for Python {
    fn name(&self) -> &str {
        "Python"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["py", "pyw"]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct JavaScript {
    pub include_jsx_files: bool,
    pub include_mjs_files: bool,
    pub include_cjs_files: bool,
}

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

#[derive(Debug, Clone, Copy)]
pub struct TypeScript {
    pub include_tsx: bool,
}

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

#[derive(Debug, Clone, Copy)]
pub struct Go;

impl Language for Go {
    fn name(&self) -> &str {
        "Go"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["go"]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rust;

impl Language for Rust {
    fn name(&self) -> &str {
        "Rust"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["rs"]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Java;

impl Language for Java {
    fn name(&self) -> &str {
        "Java"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["java"]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CSharp;

impl Language for CSharp {
    fn name(&self) -> &str {
        "C#"
    }

    fn file_extensions(&self) -> Vec<&str> {
        vec!["cs"]
    }
}
