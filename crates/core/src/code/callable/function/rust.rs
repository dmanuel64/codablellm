use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::code::callable::Descriptor;
use super::Function;

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
