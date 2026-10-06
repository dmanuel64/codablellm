use serde::{Deserialize, Serialize};

use crate::callable::{AnyCallable, Callable, Language, function::IsFunction};

#[derive(Debug, Clone, PartialEq)]
pub struct C;

impl Language for C {
    const NAME: &'static str = "C";
    type Kind = Kind;
}

impl IsFunction for C {
    fn is_function(callable: &Self::Kind) -> bool {
        matches!(callable, Kind::Function)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Function,
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
