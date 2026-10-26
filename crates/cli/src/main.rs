use codablellm_core::{
    ExtractorOptions, dataset,
    source::{AnyCode, C, Code, Subroutine},
};
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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
    let code: AnyCode = Code::<C>::parse(test_str).unwrap().into();
    let dataset = dataset::build(vec![code]).unwrap();
    println!("{dataset:?}");
    // let b = bar().as_c().unwrap().callables().first().unwrap();
}
