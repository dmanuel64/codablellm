use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = _core::stub_info()?; // the crate is imported by its lib name
    stub.generate()?;
    Ok(())
}
