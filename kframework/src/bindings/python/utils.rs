use std::collections::HashMap;

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{
        IntoPyDict, PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PySequence, PyString, PyTuple,
    },
    IntoPyObjectExt,
};
use serde_json::Number;

// =============================
// serde_json conversion helpers
// =============================

pub struct InnerValue(pub(crate) serde_json::Value);

impl From<InnerValue> for serde_json::Value {
    fn from(value: InnerValue) -> Self {
        value.0
    }
}

impl From<serde_json::Value> for InnerValue {
    fn from(value: serde_json::Value) -> Self {
        InnerValue(value)
    }
}

impl<'py> IntoPyObject<'py> for InnerValue {
    type Target = PyAny;
    type Output = Bound<'py, PyAny>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let value = self.0;
        match value {
            serde_json::Value::Null => Ok(PyNone::get(py).to_owned().into_any()),
            serde_json::Value::Bool(b) => b.into_bound_py_any(py),
            serde_json::Value::Number(number) => match number {
                _ if number.is_i64() => number.as_i64().map(|i| i.into_bound_py_any(py)),
                _ if number.is_u64() => number.as_u64().map(|i| i.into_bound_py_any(py)),
                _ if number.is_f64() => number.as_f64().map(|f| f.into_bound_py_any(py)),
                _ => None,
            }
            .ok_or(PyValueError::new_err(format!(
                "Can't create python integer from serde_json Number: {}",
                number
            )))?,
            serde_json::Value::String(s) => s.into_bound_py_any(py),
            serde_json::Value::Array(values) => values
                .into_iter()
                .map(InnerValue)
                .collect::<Vec<_>>()
                .into_bound_py_any(py),
            serde_json::Value::Object(map) => Ok(map
                .into_iter()
                .map(|(key, val)| (key, InnerValue(val)))
                .into_py_dict(py)?
                .into_any()),
        }
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for InnerValue {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        if obj.cast::<PyNone>().is_ok() {
            return Ok(serde_json::Value::Null.into());
        }
        if let Ok(b) = obj.cast::<PyBool>() {
            return Ok(serde_json::Value::from(b.is_true()).into());
        }
        if let Ok(n) = obj.cast::<PyInt>() {
            let i: Number = n
                .extract::<u64>()
                .map(Number::from)
                .or_else(|_| n.extract::<i64>().map(Number::from))?;
            return Ok(serde_json::Value::from(i).into());
        }
        if let Ok(f) = obj.cast::<PyFloat>() {
            let i: Number =
                f.extract::<f64>()
                    .map(Number::from_f64)?
                    .ok_or(PyValueError::new_err(
                        "Infinite and NaN are not supported JSON float values",
                    ))?;
            return Ok(serde_json::Value::from(i).into());
        }
        if let Ok(s) = obj.cast::<PyString>() {
            return Ok(serde_json::Value::from(s.to_str()?).into());
        }
        if let Ok(t) = obj.cast::<PyTuple>() {
            let values: Vec<serde_json::Value> = t
                .iter()
                .map(|obj| Ok(obj.extract::<InnerValue>()?.into()))
                .collect::<Result<Vec<_>, PyErr>>()?;
            return Ok(serde_json::Value::from(values).into());
        }
        if let Ok(m) = obj.cast::<PyDict>() {
            let mut map = HashMap::new();
            for (key, obj) in m.iter() {
                let key: &str = key.extract()?;
                let value: serde_json::Value = obj.extract::<InnerValue>()?.into();
                map.insert(key.to_string(), value);
            }
            Ok(serde_json::Value::from_iter(map).into())
        } else {
            Err(PyValueError::new_err(format!(
                "Error deserializing dict: {:?}",
                obj
            )))
        }
    }
}

// =========================
// Python helper methods
// =========================

/// Make a [`PyTuple`] from a [`Vec<T>`]
#[inline]
pub fn vec_to_pytuple<'py, T>(py: Python<'py>, the_vec: Vec<T>) -> PyResult<Py<PyTuple>>
where
    T: IntoPyObject<'py>,
{
    Ok(the_vec
        .into_pyobject(py)?
        .cast_into::<PyList>()?
        .to_tuple()
        .into())
}

/// Make a [`PyTuple`] from a [`PySequence`]
#[inline]
pub fn seq_to_tuple(py: Python<'_>, seq: Py<PySequence>) -> PyResult<Py<PyTuple>> {
    Ok(seq.bind(py).to_tuple()?.into())
}

/// Make a [`PyTuple`] from a [`Option<PySequence>`], making an empty tuple if it is [`None`]
#[inline]
pub fn maybe_seq_to_tuple(py: Python<'_>, seq: Option<Py<PySequence>>) -> PyResult<Py<PyTuple>> {
    match seq {
        Some(seq) => seq_to_tuple(py, seq),
        None => Ok(PyTuple::empty(py).unbind()),
    }
}

/// Make a [`PySequence`] from a [`PyTuple`]
#[inline]
pub fn py_tuple_to_sequence(py: Python<'_>, tuple: &Py<PyTuple>) -> Py<PySequence> {
    tuple.bind(py).clone().into_sequence().unbind()
}
