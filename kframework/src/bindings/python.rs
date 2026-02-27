use crate::kore::{Id, Sort};
use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyDict, PyString, PyTuple},
};

#[pyclass(subclass, name = "Sort")]
pub struct PySort {
    wrapped: Box<Sort>,
}

fn sort_to_pysort(py: Python<'_>, sort: &Sort) -> PyResult<Py<PySort>> {
    match sort {
        Sort::Var(id) => {
            let id: &Bound<'_, PyString> = &id.clone().value().into_pyobject(py)?;
            SortVar::new_(py, id)
        }
        Sort::App { id, args } => {
            let id: &Bound<'_, PyString> = &id.clone().value().into_pyobject(py)?;
            let children_: Vec<_> = args
                .iter()
                .map(|sort| sort_to_pysort(py, sort))
                .collect::<Result<Vec<_>, _>>()?;
            let args = PyTuple::new(py, children_)?;
            SortApp::new_(py, id, &args)
        }
    }
}

#[pymethods]
impl PySort {
    #[staticmethod]
    fn parse<'py>(py: Python<'py>, s: &str) -> PyResult<Py<Self>> {
        use crate::kore::Parser;

        let sort: Sort = Parser::new(s)
            .and_then(|mut p| p.sort())
            .map_err(PyValueError::new_err)?;

        sort_to_pysort(py, &sort)
    }
}

#[pyclass(extends = PySort)]
pub struct SortVar {
    #[pyo3(get)]
    name: Py<PyString>,
}

#[pymethods]
impl SortVar {
    #[new]
    fn new_(py: Python<'_>, id: &Bound<'_, PyString>) -> PyResult<Py<PySort>> {
        let id_rust = Id::new(id.to_string()).map_err(PyValueError::new_err)?;
        let sort = Sort::Var(id_rust);

        let super_ = PySort {
            wrapped: sort.into(),
        };

        let self_ = Self {
            name: id.clone().unbind(),
        };

        Ok(Bound::new(py, (self_, super_))?.into_super().unbind())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let var_annotations = PyDict::new(py);
        var_annotations.set_item("name", py.get_type::<PyString>())?;
        Ok(var_annotations)
    }
}

#[pyclass(extends = PySort)]
pub struct SortApp {
    #[pyo3(get)]
    name: Py<PyString>,
    #[pyo3(get)]
    args: Py<PyTuple>,
}

#[pymethods]
impl SortApp {
    #[new]
    fn new_<'py>(
        py: Python<'py>,
        id: &Bound<'py, PyString>,
        args: &Bound<'py, PyTuple>,
    ) -> PyResult<Py<PySort>> {
        let id_rust = Id::new(id.to_string()).map_err(PyValueError::new_err)?;

        let args_vec: Vec<Sort> = args
            .iter()
            .map(|obj| {
                obj.cast_into::<PySort>()
                    .map(|bound| (*bound.borrow().wrapped).clone())
            })
            .collect::<Result<Vec<_>, _>>()?;

        let sort = Sort::App {
            id: id_rust,
            args: args_vec,
        };

        let super_ = PySort {
            wrapped: sort.into(),
        };

        let self_ = Self {
            name: id.clone().unbind(),
            args: args.clone().unbind(),
        };

        Ok(Bound::new(py, (self_, super_))?.into_super().unbind())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let app_annotations = PyDict::new(py);
        app_annotations.set_item("name", py.get_type::<PyString>())?;
        app_annotations.set_item("args", py.get_type::<PyTuple>())?;
        Ok(app_annotations)
    }
}
