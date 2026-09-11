use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::{Descriptor, Function, Method, ScopedFunction};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CSharpMethod {
    descriptor: Descriptor,
    scope: Vec<String>,
    receiver: String,
}

impl Function for CSharpMethod {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for CSharpMethod {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Method for CSharpMethod {
    fn receiver(&self) -> &str {
        &self.receiver
    }
}

impl Display for CSharpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Method, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CSharpAssociatedFunction {
    descriptor: Descriptor,
    scope: Vec<String>,
}

impl Function for CSharpAssociatedFunction {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for CSharpAssociatedFunction {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Display for CSharpAssociatedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn ScopedFunction, f)
    }
}
