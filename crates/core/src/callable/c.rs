#![cfg(feature = "c")]
use serde::{Deserialize, Serialize};

use crate::callable::{AnyCallable, Callable, Language, function::Function};

#[derive(Debug, Clone, PartialEq)]
pub struct C;

impl Language for C {
    const NAME: &'static str = "C";
    type Kind = Kind;
}

impl AsRef<Function> for Callable<C> {
    fn as_ref(&self) -> &Function {
        let Kind::Function(function) = self.kind();
        function
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Kind {
    Function(Function),
}

impl From<Callable<C>> for AnyCallable {
    fn from(value: Callable<C>) -> Self {
        AnyCallable::C(value)
    }
}

impl Callable<C> {
    pub fn test_3(&self) {
        unreachable!()
    }
}
