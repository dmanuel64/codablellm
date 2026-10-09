use codablellm_core::source::{C, Subroutine};

fn foo() -> Subroutine<C> {
    unimplemented!()
}

fn main() {
    foo().is_void();
}
