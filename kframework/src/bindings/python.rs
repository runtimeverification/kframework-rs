use std::collections::HashMap;

use crate::kore::{Id, Sort};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{PyBool, PyDict, PyFloat, PyInt, PyNone, PyString, PyTuple},
};
use serde_json::Number;

/// Convert a [`serde_json::Value`] to a [`PyAny`]
fn serde_value_to_pyobject(py: Python<'_>, value: &serde_json::Value) -> PyResult<Py<PyAny>> {
    match value {
        serde_json::Value::Null => Ok(PyNone::get(py).to_owned().unbind().into()),
        serde_json::Value::Bool(b) => Ok(PyBool::new(py, *b).to_owned().unbind().into()),
        serde_json::Value::Number(number) => match number {
            _ if number.is_i64() => number
                .as_i64()
                .map(|i| PyInt::new(py, i).unbind().into_any()),
            _ if number.is_u64() => number
                .as_u64()
                .map(|i| PyInt::new(py, i).unbind().into_any()),
            _ if number.is_f64() => number
                .as_f64()
                .map(|i| PyFloat::new(py, i).unbind().into_any()),
            _ => None,
        }
        .ok_or(PyValueError::new_err(format!(
            "Can't create python integer from serde_json Number: {}",
            number
        ))),
        serde_json::Value::String(s) => Ok(PyString::new(py, s.as_str()).unbind().into()),
        serde_json::Value::Array(values) => {
            let py_values = values
                .iter()
                .map(|value| serde_value_to_pyobject(py, value))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(PyTuple::new(py, py_values)?.unbind().into())
        }
        serde_json::Value::Object(map) => {
            let res = PyDict::new(py);
            for (key, val) in map.iter() {
                let val = serde_value_to_pyobject(py, val)?;
                res.set_item(key, val)?;
            }
            Ok(res.into_any().unbind())
        }
    }
}

/// Convert a [`PyAny`] to a [`serde_json::Value`]
fn pyobject_to_serde_value(obj: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    if obj.cast::<PyNone>().is_ok() {
        return Ok(serde_json::Value::Null);
    }
    if let Ok(b) = obj.cast::<PyBool>() {
        return Ok(serde_json::Value::from(b.is_true()));
    }
    if let Ok(n) = obj.cast::<PyInt>() {
        let i: Number = n
            .extract::<u64>()
            .map(Number::from)
            .or_else(|_| n.extract::<i64>().map(Number::from))?;
        return Ok(serde_json::Value::from(i));
    }
    if let Ok(f) = obj.cast::<PyFloat>() {
        let i: Number = f
            .extract::<f64>()
            .map(Number::from_f64)?
            .ok_or(PyValueError::new_err(
                "Infinite and NaN are not supported JSON float values",
            ))?;
        return Ok(serde_json::Value::from(i));
    }
    if let Ok(s) = obj.cast::<PyString>() {
        return Ok(serde_json::Value::from(s.to_str()?));
    }
    if let Ok(t) = obj.cast::<PyTuple>() {
        let values = t
            .iter()
            .map(|obj| pyobject_to_serde_value(&obj))
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(serde_json::Value::from(values));
    }
    if let Ok(m) = obj.cast::<PyDict>() {
        let mut map = HashMap::new();
        for (key, obj) in m.iter() {
            let key = key.cast::<PyString>()?.to_str()?;
            let value = pyobject_to_serde_value(&obj)?;
            map.insert(key.to_string(), value);
        }
        Ok(serde_json::Value::from_iter(map))
    } else {
        Err(PyValueError::new_err(format!(
            "Error deserializing dict: {:?}",
            obj
        )))
    }
}

impl<'py> IntoPyObject<'py> for Sort {
    type Target = PySort;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Sort::Var(_) => {
                let wrapped = SortVar::make_wrapped(py, &self)?;
                let self_ = PySort {
                    wrapped: self.into(),
                };

                Ok(Bound::new(py, (wrapped, self_))?.cast_into::<PySort>()?)
            }
            Sort::App { .. } => {
                let wrapped = SortApp::make_wrapped(py, &self)?;
                let self_ = PySort {
                    wrapped: self.into(),
                };

                Ok(Bound::new(py, (wrapped, self_))?.cast_into::<PySort>()?)
            }
        }
    }
}

/// [`PySort`]
#[pyclass(subclass, name = "Sort")]
pub struct PySort {
    wrapped: Box<Sort>,
}

#[pymethods]
impl PySort {
    #[staticmethod]
    fn parse<'py>(py: Python<'py>, s: &str) -> PyResult<Bound<'py, Self>> {
        use crate::kore::Parser;

        let sort: Sort = Parser::new(s)
            .and_then(|mut p| p.sort())
            .map_err(PyValueError::new_err)?;

        sort.into_pyobject(py)
    }

    #[staticmethod]
    fn from_dict<'py>(dict: &Bound<'py, PyAny>) -> PyResult<Bound<'py, Self>> {
        let value = pyobject_to_serde_value(dict)?;
        let sort: Sort =
            serde_json::from_value(value).map_err(|e| PyValueError::new_err(e.to_string()))?;

        sort.into_pyobject(dict.py())
    }

    fn dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let value = serde_json::to_value(self.wrapped.as_ref())
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        serde_value_to_pyobject(py, &value)
            .and_then(|obj| Ok(obj.cast_bound::<PyDict>(py)?.clone().unbind()))
    }
}

#[pyclass(extends = PySort)]
pub struct SortVar {
    #[pyo3(get)]
    name: Py<PyString>,
}

impl SortVar {
    fn make_wrapped(py: Python<'_>, sort: &Sort) -> PyResult<SortVar> {
        match sort {
            Sort::Var(id) => {
                let name = PyString::new(py, id.clone().value().as_str());
                Ok(SortVar {
                    name: name.unbind(),
                })
            }
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl SortVar {
    #[new]
    fn new_(py: Python<'_>, name: &Bound<'_, PyString>) -> PyResult<Py<PySort>> {
        let id_rust = Id::new(name.to_string()).map_err(PyValueError::new_err)?;
        let sort = Sort::Var(id_rust);

        let super_ = PySort {
            wrapped: sort.into(),
        };

        let self_ = Self {
            name: name.clone().unbind(),
        };

        Ok(Bound::new(py, (self_, super_))?.into_super().unbind())
    }

    #[pyo3(signature = (name=None))]
    fn r#let(&self, py: Python<'_>, name: Option<String>) -> PyResult<Py<PySort>> {
        let name = name
            .map(|s| PyString::new(py, s.as_str()))
            .unwrap_or(self.name.bind(py).clone());
        Self::new_(py, &name)
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
    sorts: Py<PyTuple>,
}

impl SortApp {
    fn make_wrapped(py: Python<'_>, sort: &Sort) -> PyResult<Self> {
        match sort {
            Sort::App { id, args } => {
                let id: Bound<'_, PyString> = PyString::new(py, id.clone().value().as_str());
                let children_: Vec<_> = args
                    .iter()
                    .map(|sort| sort.clone().into_pyobject(py))
                    .collect::<Result<Vec<_>, _>>()?;
                let args = PyTuple::new(py, children_)?;
                Ok(Self {
                    name: id.unbind(),
                    sorts: args.unbind(),
                })
            }
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl SortApp {
    #[new]
    #[pyo3(signature = (name, sorts=None))]
    fn new_<'py>(
        py: Python<'py>,
        name: &Bound<'py, PyString>,
        sorts: Option<&Bound<'py, PyTuple>>,
    ) -> PyResult<Py<PySort>> {
        let id_rust = Id::new(name.to_string()).map_err(PyValueError::new_err)?;
        let empty = &PyTuple::empty(py);
        let sorts = sorts.unwrap_or(empty);

        let args_vec: Vec<Sort> = sorts
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
            name: name.clone().unbind(),
            sorts: sorts.clone().unbind(),
        };

        Ok(Bound::new(py, (self_, super_))?.into_super().unbind())
    }

    #[pyo3(signature = (name=None, sorts=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        name: Option<String>,
        sorts: Option<&Bound<'_, PyTuple>>,
    ) -> PyResult<Py<PySort>> {
        let name = name
            .map(|s| PyString::new(py, s.as_str()).unbind())
            .unwrap_or(self.name.clone_ref(py));
        let args = sorts.unwrap_or(self.sorts.bind(py));
        Self::new_(py, name.bind(py), Some(args))
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let app_annotations = PyDict::new(py);
        app_annotations.set_item("name", py.get_type::<PyString>())?;
        app_annotations.set_item("sorts", py.get_type::<PyTuple>())?;
        Ok(app_annotations)
    }
}
