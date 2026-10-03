use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::code::callable::Descriptor;
use crate::code::callable::associated_function::ScopedFunction;
use crate::code::callable::function::Function;
use super::Method;

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
