use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::{Descriptor, Function, Method, ScopedFunction};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JavaMethod {
    descriptor: Descriptor,
    scope: Vec<String>,
    receiver: String,
}

impl Function for JavaMethod {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for JavaMethod {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Method for JavaMethod {
    fn receiver(&self) -> &str {
        &self.receiver
    }
}

impl Display for JavaMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Method, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JavaAssociatedFunction {
    descriptor: Descriptor,
    scope: Vec<String>,
}

impl Function for JavaAssociatedFunction {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for JavaAssociatedFunction {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Display for JavaAssociatedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn ScopedFunction, f)
    }
}
