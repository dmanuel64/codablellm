use crate::source;

pub type TransformFn = dyn Fn(&mut source::AnyCode);
