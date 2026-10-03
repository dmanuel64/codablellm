mod c;
mod cpp;
mod go;
mod javascript;
mod python;
mod rust;
mod typescript;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub use c::CFunction;
pub use cpp::CppFunction;
pub use go::GoFunction;
pub use javascript::JavaScriptFunction;
pub use python::PythonFunction;
pub use rust::RustFunction;
pub use typescript::TypeScriptFunction;

use crate::{
    code::{Code, callable::Descriptor},
    language::Language,
};

pub trait Function {
    fn descriptor(&self) -> &Descriptor;

    fn name(&self) -> &str {
        &self.descriptor().name
    }

    fn definition(&self) -> &Code {
        &self.descriptor().definition
    }

    fn language(&self) -> Language {
        self.descriptor().language
    }

    fn location(&self) -> &Option<crate::code::callable::Location> {
        &self.descriptor().location
    }
}

impl Display for dyn Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = self
            .location()
            .as_ref()
            .map(crate::code::callable::Location::to_string)
            .unwrap_or_else(|| String::from("<MEM>"));
        let func = self.name();
        write!(f, "{func} ({location})")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceFunction {
    C(CFunction),
    Cpp(CppFunction),
    Python(PythonFunction),
    JavaScript(JavaScriptFunction),
    TypeScript(TypeScriptFunction),
    Go(GoFunction),
    Rust(RustFunction),
}

impl Function for SourceFunction {
    fn descriptor(&self) -> &Descriptor {
        match self {
            SourceFunction::C(func) => func.descriptor(),
            SourceFunction::Cpp(func) => func.descriptor(),
            SourceFunction::Python(func) => func.descriptor(),
            SourceFunction::JavaScript(func) => func.descriptor(),
            SourceFunction::TypeScript(func) => func.descriptor(),
            SourceFunction::Go(func) => func.descriptor(),
            SourceFunction::Rust(func) => func.descriptor(),
        }
    }
}

impl SourceFunction {
    pub fn as_c(&self) -> Option<&CFunction> {
        if let SourceFunction::C(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_cpp(&self) -> Option<&CppFunction> {
        if let SourceFunction::Cpp(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_python(&self) -> Option<&PythonFunction> {
        if let SourceFunction::Python(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_javascript(&self) -> Option<&JavaScriptFunction> {
        if let SourceFunction::JavaScript(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_typescript(&self) -> Option<&TypeScriptFunction> {
        if let SourceFunction::TypeScript(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_go(&self) -> Option<&GoFunction> {
        if let SourceFunction::Go(func) = self {
            Some(func)
        } else {
            None
        }
    }

    pub fn as_rust(&self) -> Option<&RustFunction> {
        if let SourceFunction::Rust(func) = self {
            Some(func)
        } else {
            None
        }
    }
}

impl Display for SourceFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceFunction::C(func) => write!(f, "{func}"),
            SourceFunction::Cpp(func) => write!(f, "{func}"),
            SourceFunction::Python(func) => write!(f, "{func}"),
            SourceFunction::JavaScript(func) => write!(f, "{func}"),
            SourceFunction::TypeScript(func) => write!(f, "{func}"),
            SourceFunction::Go(func) => write!(f, "{func}"),
            SourceFunction::Rust(func) => write!(f, "{func}"),
        }
    }
}
