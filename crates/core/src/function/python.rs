use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::{Descriptor, Function, Method, ScopedFunction};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonFunction {
    descriptor: Descriptor,
}

impl Function for PythonFunction {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl Display for PythonFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Function, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonMethod {
    descriptor: Descriptor,
    scope: Vec<String>,
    receiver: String,
}

impl Function for PythonMethod {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for PythonMethod {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Method for PythonMethod {
    fn receiver(&self) -> &str {
        &self.receiver
    }
}

impl Display for PythonMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Method, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PythonAssociatedFunction {
    descriptor: Descriptor,
    scope: Vec<String>,
}

impl Function for PythonAssociatedFunction {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for PythonAssociatedFunction {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Display for PythonAssociatedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn ScopedFunction, f)
    }
}
