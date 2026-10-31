use pyo3::prelude::*;
use pyo3_stub_gen::{define_stub_info_gatherer, reexport_module_members};

#[pymodule]
mod source {
    use codablellm_core::source::{self, AnyCode, C, Callable, Subroutine};
    use pyo3::prelude::*;
    use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

    #[gen_stub_pyclass]
    #[pyclass(
        subclass,
        frozen,
        name = "CodeFile",
        module = "codablellm._core.source"
    )]
    pub struct PyCodeFile(source::CodeFile);

    #[gen_stub_pyclass]
    #[pyclass(subclass, frozen, name = "Code", module = "codablellm._core.source")]
    pub struct PyCode(AnyCode);

    #[gen_stub_pyclass]
    #[pyclass(
        subclass,
        frozen,
        name = "Subroutine",
        module = "codablellm._core.source"
    )]
    pub struct PySubroutine(Box<dyn Callable + Send + Sync>);

    #[gen_stub_pyclass]
    #[pyclass(extends = PySubroutine, frozen, name = "CSubroutine", module = "codablellm._core.source")]
    pub struct PyCSubroutine(Subroutine<C>);

    #[gen_stub_pymethods]
    #[pymethods]
    impl PyCSubroutine {
        #[getter]
        fn return_type(&self) -> Option<String> {
            self.0.return_type().as_ref().map(ToString::to_string)
        }

        #[getter]
        fn is_void(&self) -> bool {
            self.0.is_void()
        }
    }
}

#[pymodule]
mod extractor {
    use std::path::PathBuf;

    use codablellm_core::extractor;
    use pyo3::{exceptions::PyValueError, prelude::*};
    use pyo3_stub_gen::derive::gen_stub_pyfunction;

    use crate::source::PyCodeFile;

    struct PyExtractorError(extractor::Error);

    impl From<PyExtractorError> for PyErr {
        fn from(error: PyExtractorError) -> Self {
            PyValueError::new_err(error.0.to_string())
        }
    }

    impl From<extractor::Error> for PyExtractorError {
        fn from(other: extractor::Error) -> Self {
            Self(other)
        }
    }

    #[gen_stub_pyfunction(module = "codablellm._core.extractor")]
    #[pyfunction]
    fn extract(path: PathBuf) -> Result<PyCodeFile, PyExtractorError> {
        todo!()
    }
}

/// A Python module implemented in Rust. The name of this module must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
mod _core {
    use pyo3::prelude::*;

    #[pymodule_export]
    use crate::extractor;
    #[pymodule_export]
    use crate::source;
}

reexport_module_members!("codablellm", "codablellm._core");
define_stub_info_gatherer!(stub_info);
