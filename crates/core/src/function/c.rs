use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::{Descriptor, Function};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CFunction {
    descriptor: Descriptor,
}

impl Function for CFunction {
    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

impl Display for CFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self as &dyn Function, f)
    }
}
