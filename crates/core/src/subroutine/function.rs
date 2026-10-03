use crate::subroutine::Subroutine;

pub trait Function: Subroutine {
    fn return_type(&self) -> &str;
}

pub trait AssociatedFunction: Function {
    fn scope(&self) -> &[&str];
}

pub trait Method: AssociatedFunction {
    fn receiver(&self) -> Option<&str> {
        self.scope().last().map(|v| *v)
    }
}
