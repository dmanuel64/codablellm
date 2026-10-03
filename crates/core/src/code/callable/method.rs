mod cpp;
mod csharp;
mod go;
mod java;
mod javascript;
mod python;
mod rust;
mod typescript;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub use cpp::CppMethod;
pub use csharp::CSharpMethod;
pub use go::GoMethod;
pub use java::JavaMethod;
pub use javascript::JavaScriptMethod;
pub use python::PythonMethod;
pub use rust::RustMethod;
pub use typescript::TypeScriptMethod;

use crate::{
    SourceLanguage,
    code::callable::{Descriptor, associated_function::ScopedFunction, function::Function},
    language::Language,
};

pub trait Method: ScopedFunction {
    fn receiver(&self) -> &str;
}

impl Display for dyn Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = ToString::to_string(self as &dyn Function);
        let separator = match self.language() {
            Language::Source(SourceLanguage::Cpp | SourceLanguage::Rust) => "::",
            _ => ".",
        };
        let scope = self.scope().join(separator);
        let receiver = self.receiver();
        write!(f, "{scope}{separator}{receiver}{separator}{out}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceMethod {
    Cpp(CppMethod),
    Python(PythonMethod),
    JavaScript(JavaScriptMethod),
    TypeScript(TypeScriptMethod),
    Go(GoMethod),
    Rust(RustMethod),
    Java(JavaMethod),
    CSharp(CSharpMethod),
}

impl Function for SourceMethod {
    fn descriptor(&self) -> &Descriptor {
        match self {
            SourceMethod::Cpp(func) => func.descriptor(),
            SourceMethod::Python(func) => func.descriptor(),
            SourceMethod::JavaScript(func) => func.descriptor(),
            SourceMethod::TypeScript(func) => func.descriptor(),
            SourceMethod::Go(func) => func.descriptor(),
            SourceMethod::Rust(func) => func.descriptor(),
            SourceMethod::Java(func) => func.descriptor(),
            SourceMethod::CSharp(func) => func.descriptor(),
        }
    }
}

impl ScopedFunction for SourceMethod {
    fn scope(&self) -> &[String] {
        match self {
            SourceMethod::Cpp(func) => func.scope(),
            SourceMethod::Python(func) => func.scope(),
            SourceMethod::JavaScript(func) => func.scope(),
            SourceMethod::TypeScript(func) => func.scope(),
            SourceMethod::Go(func) => func.scope(),
            SourceMethod::Rust(func) => func.scope(),
            SourceMethod::Java(func) => func.scope(),
            SourceMethod::CSharp(func) => func.scope(),
        }
    }
}

impl Method for SourceMethod {
    fn receiver(&self) -> &str {
        match self {
            SourceMethod::Cpp(func) => func.receiver(),
            SourceMethod::Python(func) => func.receiver(),
            SourceMethod::JavaScript(func) => func.receiver(),
            SourceMethod::TypeScript(func) => func.receiver(),
            SourceMethod::Go(func) => func.receiver(),
            SourceMethod::Rust(func) => func.receiver(),
            SourceMethod::Java(func) => func.receiver(),
            SourceMethod::CSharp(func) => func.receiver(),
        }
    }
}

impl SourceMethod {
    pub fn as_cpp(&self) -> Option<&CppMethod> {
        if let SourceMethod::Cpp(func) = self { Some(func) } else { None }
    }

    pub fn as_python(&self) -> Option<&PythonMethod> {
        if let SourceMethod::Python(func) = self { Some(func) } else { None }
    }

    pub fn as_javascript(&self) -> Option<&JavaScriptMethod> {
        if let SourceMethod::JavaScript(func) = self { Some(func) } else { None }
    }

    pub fn as_typescript(&self) -> Option<&TypeScriptMethod> {
        if let SourceMethod::TypeScript(func) = self { Some(func) } else { None }
    }

    pub fn as_go(&self) -> Option<&GoMethod> {
        if let SourceMethod::Go(func) = self { Some(func) } else { None }
    }

    pub fn as_rust(&self) -> Option<&RustMethod> {
        if let SourceMethod::Rust(func) = self { Some(func) } else { None }
    }

    pub fn as_java(&self) -> Option<&JavaMethod> {
        if let SourceMethod::Java(func) = self { Some(func) } else { None }
    }

    pub fn as_csharp(&self) -> Option<&CSharpMethod> {
        if let SourceMethod::CSharp(func) = self { Some(func) } else { None }
    }
}

impl Display for SourceMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceMethod::Cpp(func) => write!(f, "{func}"),
            SourceMethod::Python(func) => write!(f, "{func}"),
            SourceMethod::JavaScript(func) => write!(f, "{func}"),
            SourceMethod::TypeScript(func) => write!(f, "{func}"),
            SourceMethod::Go(func) => write!(f, "{func}"),
            SourceMethod::Rust(func) => write!(f, "{func}"),
            SourceMethod::Java(func) => write!(f, "{func}"),
            SourceMethod::CSharp(func) => write!(f, "{func}"),
        }
    }
}
