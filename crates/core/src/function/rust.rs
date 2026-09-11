use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::{Descriptor, Function, Method, ScopedFunction};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustFunction {
    descriptor: Descriptor,
}

impl Function for RustFunction {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl Display for RustFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Function, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustMethod {
    descriptor: Descriptor,
    scope: Vec<String>,
    receiver: String,
}

impl Function for RustMethod {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for RustMethod {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Method for RustMethod {
    fn receiver(&self) -> &str {
        &self.receiver
    }
}

impl Display for RustMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Method, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustAssociatedFunction {
    descriptor: Descriptor,
    scope: Vec<String>,
}

impl Function for RustAssociatedFunction {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for RustAssociatedFunction {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Display for RustAssociatedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn ScopedFunction, f)
    }
}
