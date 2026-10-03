mod cpp;
mod csharp;
mod java;
mod javascript;
mod python;
mod rust;
mod typescript;

use std::fmt::Display;

use serde::{Deserialize, Serialize};

pub use cpp::CppAssociatedFunction;
pub use csharp::CSharpAssociatedFunction;
pub use java::JavaAssociatedFunction;
pub use javascript::JavaScriptAssociatedFunction;
pub use python::PythonAssociatedFunction;
pub use rust::RustAssociatedFunction;
pub use typescript::TypeScriptAssociatedFunction;

use crate::{
    SourceLanguage,
    code::callable::{Descriptor, function::Function},
    language::Language,
};

pub trait ScopedFunction: Function {
    fn scope(&self) -> &[String];
}

impl Display for dyn ScopedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = (self as &dyn Function).to_string();
        let separator = match self.language() {
            Language::Source(SourceLanguage::Cpp | SourceLanguage::Rust) => "::",
            _ => ".",
        };
        let scope = self.scope().join(separator);
        write!(f, "{scope}{separator}{out}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceAssociatedFunction {
    Cpp(CppAssociatedFunction),
    Python(PythonAssociatedFunction),
    JavaScript(JavaScriptAssociatedFunction),
    TypeScript(TypeScriptAssociatedFunction),
    Rust(RustAssociatedFunction),
    Java(JavaAssociatedFunction),
    CSharp(CSharpAssociatedFunction),
}

impl Function for SourceAssociatedFunction {
    fn descriptor(&self) -> &Descriptor {
        match self {
            SourceAssociatedFunction::Cpp(func) => func.descriptor(),
            SourceAssociatedFunction::Python(func) => func.descriptor(),
            SourceAssociatedFunction::JavaScript(func) => func.descriptor(),
            SourceAssociatedFunction::TypeScript(func) => func.descriptor(),
            SourceAssociatedFunction::Rust(func) => func.descriptor(),
            SourceAssociatedFunction::Java(func) => func.descriptor(),
            SourceAssociatedFunction::CSharp(func) => func.descriptor(),
        }
    }
}

impl ScopedFunction for SourceAssociatedFunction {
    fn scope(&self) -> &[String] {
        match self {
            SourceAssociatedFunction::Cpp(func) => func.scope(),
            SourceAssociatedFunction::Python(func) => func.scope(),
            SourceAssociatedFunction::JavaScript(func) => func.scope(),
            SourceAssociatedFunction::TypeScript(func) => func.scope(),
            SourceAssociatedFunction::Rust(func) => func.scope(),
            SourceAssociatedFunction::Java(func) => func.scope(),
            SourceAssociatedFunction::CSharp(func) => func.scope(),
        }
    }
}

impl SourceAssociatedFunction {
    pub fn as_cpp(&self) -> Option<&CppAssociatedFunction> {
        if let SourceAssociatedFunction::Cpp(func) = self { Some(func) } else { None }
    }

    pub fn as_python(&self) -> Option<&PythonAssociatedFunction> {
        if let SourceAssociatedFunction::Python(func) = self { Some(func) } else { None }
    }

    pub fn as_javascript(&self) -> Option<&JavaScriptAssociatedFunction> {
        if let SourceAssociatedFunction::JavaScript(func) = self { Some(func) } else { None }
    }

    pub fn as_typescript(&self) -> Option<&TypeScriptAssociatedFunction> {
        if let SourceAssociatedFunction::TypeScript(func) = self { Some(func) } else { None }
    }

    pub fn as_rust(&self) -> Option<&RustAssociatedFunction> {
        if let SourceAssociatedFunction::Rust(func) = self { Some(func) } else { None }
    }

    pub fn as_java(&self) -> Option<&JavaAssociatedFunction> {
        if let SourceAssociatedFunction::Java(func) = self { Some(func) } else { None }
    }

    pub fn as_csharp(&self) -> Option<&CSharpAssociatedFunction> {
        if let SourceAssociatedFunction::CSharp(func) = self { Some(func) } else { None }
    }
}

impl Display for SourceAssociatedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceAssociatedFunction::Cpp(func) => write!(f, "{func}"),
            SourceAssociatedFunction::Python(func) => write!(f, "{func}"),
            SourceAssociatedFunction::JavaScript(func) => write!(f, "{func}"),
            SourceAssociatedFunction::TypeScript(func) => write!(f, "{func}"),
            SourceAssociatedFunction::Rust(func) => write!(f, "{func}"),
            SourceAssociatedFunction::Java(func) => write!(f, "{func}"),
            SourceAssociatedFunction::CSharp(func) => write!(f, "{func}"),
        }
    }
}
