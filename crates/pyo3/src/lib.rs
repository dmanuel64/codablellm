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
    pub struct PyCodeFile(pub source::CodeFile);

    #[gen_stub_pyclass]
    #[pyclass(subclass, frozen, name = "Code", module = "codablellm._core.source")]
    pub struct PyCode(pub AnyCode);

    #[gen_stub_pyclass]
    #[pyclass(
        subclass,
        frozen,
        name = "Subroutine",
        module = "codablellm._core.source"
    )]
    pub struct PySubroutine(pub Box<dyn Callable + Send + Sync>);

    #[gen_stub_pyclass]
    #[pyclass(extends = PySubroutine, frozen, name = "CSubroutine", module = "codablellm._core.source")]
    pub struct PyCSubroutine(pub Subroutine<C>);

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
    use std::{num::NonZero, path::PathBuf, sync::LazyLock};

    use codablellm_core::extractor;
    use pyo3::{
        exceptions::{PyBaseException, PyException},
        prelude::*,
    };
    use pyo3_stub_gen::{PyStubType, TypeInfo, create_exception, derive::gen_stub_pyfunction};

    use crate::source::PyCodeFile;

    create_exception!(codablellm._core.extractor, ExtractorError, PyException);
    create_exception!(codablellm._core.extractor, LoaderError, PyException);

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        let py = m.py();
        m.add("ExtractorError", py.get_type::<ExtractorError>())?;
        m.add("LoaderError", py.get_type::<LoaderError>())?;
        Ok(())
    }

    #[derive(IntoPyObject)]
    pub enum PyExtractResult {
        Ok(PyCodeFile),
        Err(Py<PyBaseException>),
    }

    impl PyStubType for PyExtractResult {
        fn type_output() -> TypeInfo {
            PyCodeFile::type_output() | TypeInfo::unqualified("LoaderError")
        }
    }

    #[gen_stub_pyfunction(module = "codablellm._core.extractor")]
    #[pyfunction]
    #[pyo3(signature = (path, max_jobs = None))]
    fn extract(
        py: Python<'_>,
        path: PathBuf,
        max_jobs: Option<NonZero<usize>>,
    ) -> PyResult<Vec<PyExtractResult>> {
        // TODO: eventually support custom loaders from Python Extract functions
        // without locking the GIL; only borrow the GIL when creating Python
        // exceptions
        let results = py
            .detach(|| {
                extractor::extract_with_options(
                    &path,
                    &extractor::Options {
                        max_jobs,
                        ..Default::default()
                    },
                )
            })
            .map_err(|err| ExtractorError::new_err(err.to_string()))?;

        Ok(results
            .into_iter()
            .map(|result| match result {
                Ok(file) => PyExtractResult::Ok(PyCodeFile(file)),
                Err(err) => {
                    PyExtractResult::Err(LoaderError::new_err(err.to_string()).into_value(py))
                }
            })
            .collect())
    }
}

/// A Python module implemented in Rust. The name of this module must match the
/// `lib.name` setting in the `Cargo.toml`, else Python will not be able to
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
