use crate::kore::Id;
use pyo3::{exceptions::PyValueError, prelude::*};

#[pymethods]
impl Id {
    #[new]
    fn py_new(s: String) -> PyResult<Self> {
        Self::new(s).map_err(|e| PyErr::new::<PyValueError, _>(e))
    }

    #[getter(value)]
    fn py_value(&self) -> &str {
        self.value()
    }
}
