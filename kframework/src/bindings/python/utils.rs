use std::collections::HashMap;

use pyo3::{
    exceptions::PyValueError,
    prelude::*,
    types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PySequence, PyString, PyTuple},
};
use serde_json::Number;

/// Convert a [`serde_json::Value`] to a [`PyAny`]
pub fn serde_value_to_pyobject(py: Python<'_>, value: &serde_json::Value) -> PyResult<Py<PyAny>> {
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
pub fn pyobject_to_serde_value(obj: &Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
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
