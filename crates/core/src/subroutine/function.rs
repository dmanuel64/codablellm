use std::fmt::Display;

use crate::subroutine::Subroutine;

pub trait Function: Subroutine {
    fn return_type(&self) -> &str;
}

impl Display for dyn Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out = self as &dyn Subroutine;
        write!(f, "{out}")
    }
}

pub trait AssociatedFunction: Function {
    fn scope(&self) -> &[&str];
}

impl Display for dyn AssociatedFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let full_scope: Vec<&str> = self
            .scope()
            .iter()
            .copied()
            .chain(std::iter::once(self.name()))
            .collect();
        let scope_str = full_scope.join("::");
        if let Some(location) = &self.descriptor().location {
            write!(f, "{scope_str} ({})", location)
        } else {
            write!(f, "{scope_str}")
        }
    }
}

pub trait Method: AssociatedFunction {
    fn receiver(&self) -> &str;
}

impl Display for dyn Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let full_scope: Vec<&str> = self
            .scope()
            .iter()
            .copied()
            .chain(std::iter::once(self.receiver()))
            .chain(std::iter::once(self.name()))
            .collect();
        let scope_str = full_scope.join("::");
        if let Some(location) = &self.descriptor().location {
            write!(f, "{scope_str} ({})", location)
        } else {
            write!(f, "{scope_str}")
        }
    }
}
