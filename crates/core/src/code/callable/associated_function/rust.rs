use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::code::callable::Descriptor;
use crate::code::callable::function::Function;
use super::ScopedFunction;

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
