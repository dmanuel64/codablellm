use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::{Descriptor, Function, Method, ScopedFunction};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeScriptFunction {
    descriptor: Descriptor,
}

impl Function for TypeScriptFunction {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl Display for TypeScriptFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Function, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeScriptMethod {
    descriptor: Descriptor,
    scope: Vec<String>,
    receiver: String,
}

impl Function for TypeScriptMethod {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for TypeScriptMethod {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Method for TypeScriptMethod {
    fn receiver(&self) -> &str {
        &self.receiver
    }
}

impl Display for TypeScriptMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Method, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeScriptAssociatedFunction {
    descriptor: Descriptor,
    scope: Vec<String>,
}

impl Function for TypeScriptAssociatedFunction {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for TypeScriptAssociatedFunction {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Display for TypeScriptAssociatedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn ScopedFunction, f)
    }
}
