use serde::{Deserialize, Serialize};

use crate::callable::{Callable, Language};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub return_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalFunction {
    pub parent: Function,
    pub inner: Function,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnonymousFunction(pub Function);
