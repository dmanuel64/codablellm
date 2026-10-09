use codablellm_core::source::{AnyCode, C, Source, Subroutine};

fn foo() -> Subroutine<C> {
    unimplemented!()
}

fn bar() -> AnyCode {
    unimplemented!()
}

fn main() {
    let b = bar().as_c().unwrap().callables().first().unwrap();
}
