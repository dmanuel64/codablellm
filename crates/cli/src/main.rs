use codablellm_core::{
    dataset,
    source::{AnyCode, C, Code, Subroutine},
};

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
    let code: AnyCode = Code::<C>::new(test_str).unwrap().into();
    let dataset = dataset::build(vec![code]).unwrap();
    println!("{dataset:?}");
    // let b = bar().as_c().unwrap().callables().first().unwrap();
}
