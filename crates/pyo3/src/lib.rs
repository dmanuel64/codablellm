use pyo3::prelude::*;

#[pyfunction]
fn hello_world() -> String {
    todo!()
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(hello_world, m)?)?;
    Ok(())
}
