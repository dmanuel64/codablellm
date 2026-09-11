use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::{Descriptor, Function, Method, ScopedFunction};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoFunction {
    descriptor: Descriptor,
}

impl Function for GoFunction {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl Display for GoFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Function, f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoMethod {
    descriptor: Descriptor,
    scope: Vec<String>,
    receiver: String,
}

impl Function for GoMethod {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for GoMethod {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Method for GoMethod {
    fn receiver(&self) -> &str {
        &self.receiver
    }
}

impl Display for GoMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Method, f)
    }
}
