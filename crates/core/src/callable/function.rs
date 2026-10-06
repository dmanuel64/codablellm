use std::ops::Deref;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub return_type: Option<String>,
}

impl Function {
    pub fn is_void(&self) -> bool {
        self.return_type
            .as_ref()
            .is_none_or(|return_value| return_value.eq_ignore_ascii_case("void"))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalFunction {
    pub parent: Function,
    pub inner: Function,
}

impl Deref for LocalFunction {
    type Target = Function;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scope {
    pub parent: String,
    pub ancestors: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScopedFunction {
    pub scope: Scope,
    pub inner: Function,
}

impl Deref for ScopedFunction {
    type Target = Function;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssociatedFunction(ScopedFunction);
pub type StaticFunction = AssociatedFunction;

impl Deref for AssociatedFunction {
    type Target = ScopedFunction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AssociatedFunction {
    pub fn owner(&self) -> &str {
        &self.scope.parent
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Method(pub AssociatedFunction);

impl Deref for Method {
    type Target = AssociatedFunction;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Method {
    pub fn receiver(&self) -> &str {
        self.owner()
    }
}
