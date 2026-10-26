use codablellm_core::{
    ExtractorOptions, dataset,
    source::{AnyCode, C, Code, Subroutine},
};
use color_eyre::eyre::Result;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn install_error_handlers() -> Result<()> {
    color_eyre::install()?;
    human_panic::setup_panic!();
    Ok(())
}

fn main() -> Result<()> {
    install_error_handlers()?;
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
    Ok(())
}
