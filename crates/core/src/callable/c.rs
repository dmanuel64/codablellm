use serde::{Deserialize, Serialize};

use crate::callable::{AnyCallable, Language, function::IsFunction};

#[derive(Debug, Clone, PartialEq)]
pub struct C;

impl Language for C {
    const NAME: &'static str = "C";
    type Callable = Callable;
}

impl IsFunction for C {
    fn is_function(callable: &Self::Callable) -> bool {
        matches!(callable, Callable::Function)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Callable {
    Function,
}

impl From<super::Callable<C>> for AnyCallable {
    fn from(value: super::Callable<C>) -> Self {
        AnyCallable::C(value)
    }
}
