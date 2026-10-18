use codablellm_core::source::{AnyCode, C, Code, Language, Source, Subroutine};

fn foo() -> Subroutine<C> {
    unimplemented!()
}

fn bar() -> AnyCode {
    unimplemented!()
}

fn main() {
    let test_str = "int main() {
    printf(\"Hello, World!\\n\");
    return 0;
}

void bar() {
    
}";
    let code = Code::<C>::new(test_str).unwrap();
    println!("{:#?}", code.subroutines());
    // let b = bar().as_c().unwrap().callables().first().unwrap();
}
