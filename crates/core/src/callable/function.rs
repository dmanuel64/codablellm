use crate::callable::{Callable, Language};

pub trait IsFunction: Language {
    fn is_function(callable: &Self::Callable) -> bool;
}

impl<L: IsFunction> Callable<L> {
    pub fn is_function(&self) -> bool {
        L::is_function(self)
    }
}
