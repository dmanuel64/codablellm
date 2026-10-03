use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::code::callable::Descriptor;
use crate::code::callable::associated_function::ScopedFunction;
use crate::code::callable::function::Function;
use super::Method;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CppMethod {
    descriptor: Descriptor,
    scope: Vec<String>,
    receiver: String,
}

impl Function for CppMethod {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl ScopedFunction for CppMethod {
    fn scope(&self) -> &[String] {
        &self.scope
    }
}

impl Method for CppMethod {
    fn receiver(&self) -> &str {
        &self.receiver
    }
}

impl Display for CppMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Method, f)
    }
}
