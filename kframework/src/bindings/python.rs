use std::collections::HashMap;

use crate::kore::{Id, Pattern, Sentence, SetVarId, Sort, Str, SymbolId, Var};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{PyBool, PyDict, PyFloat, PyInt, PyNone, PyString, PyTuple},
    PyClass,
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

trait Wrappable<RustType>: Sized {
    fn wrap(py: Python<'_>, rust: &RustType) -> PyResult<Self>;
}

fn convert<SubClass, RustType>(
    py: Python<'_>,
    it: RustType,
) -> PyResult<Bound<'_, SubClass::BaseType>>
where
    SubClass: PyClass + Wrappable<RustType>,
    SubClass::BaseType: Wrappable<RustType>,
    (SubClass, SubClass::BaseType): Into<PyClassInitializer<SubClass>>,
{
    let s = SubClass::wrap(py, &it)?;
    let b = SubClass::BaseType::wrap(py, &it)?;

    Ok(Bound::new(py, (s, b))?.cast_into::<SubClass::BaseType>()?)
}

// ==========================================
// Sort bindings
// ==========================================

#[pyclass(subclass, name = "Sort")]
pub struct PySort {
    wrapped: Box<Sort>,
}

impl<'py> IntoPyObject<'py> for Sort {
    type Target = PySort;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Sort::Var(_) => convert::<SortVar, _>(py, self),
            Sort::App { .. } => convert::<SortApp, _>(py, self),
        }
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Sort {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        Ok(obj
            .cast::<PySort>()
            .map(|sort| *sort.borrow().wrapped.clone())?)
    }
}

impl Wrappable<Sort> for PySort {
    fn wrap(_py: Python<'_>, rust: &Sort) -> PyResult<Self> {
        Ok(Self {
            wrapped: rust.clone().into(),
        })
    }
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
    name: String,
}

impl Wrappable<Sort> for SortVar {
    fn wrap(_py: Python<'_>, it: &Sort) -> PyResult<Self> {
        match it {
            Sort::Var(id) => Ok(SortVar {
                name: id.clone().value(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl SortVar {
    #[new]
    fn new_(py: Python<'_>, name: String) -> PyResult<Py<PySort>> {
        let id = Id::new(name).map_err(PyValueError::new_err)?;
        let sort = Sort::Var(id);

        sort.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (name=None))]
    fn r#let(&self, py: Python<'_>, name: Option<String>) -> PyResult<Py<PySort>> {
        let name = name.unwrap_or(self.name.clone());
        Self::new_(py, name)
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
    name: String,
    #[pyo3(get)]
    sorts: Vec<Sort>,
}

impl Wrappable<Sort> for SortApp {
    fn wrap(_py: Python<'_>, it: &Sort) -> PyResult<Self> {
        match it {
            Sort::App { id, args } => Ok(Self {
                name: id.clone().value(),
                sorts: args.to_vec(),
            }),
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
    fn new_(py: Python<'_>, name: String, sorts: Option<Vec<Sort>>) -> PyResult<Py<PySort>> {
        let id = Id::new(name).map_err(PyValueError::new_err)?;

        let sort = Sort::App {
            id,
            args: sorts.unwrap_or(vec![]),
        };

        sort.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (name=None, sorts=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        name: Option<String>,
        sorts: Option<Vec<Sort>>,
    ) -> PyResult<Py<PySort>> {
        let name = name.unwrap_or_else(|| self.name.clone());
        let sorts = sorts.unwrap_or_else(|| self.sorts.clone());
        Self::new_(py, name, Some(sorts))
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let app_annotations = PyDict::new(py);
        app_annotations.set_item("name", py.get_type::<PyString>())?;
        app_annotations.set_item("sorts", py.get_type::<PyTuple>())?;
        Ok(app_annotations)
    }
}

// ==========================================
// Pattern bindings
// ==========================================

#[pyclass(subclass, name = "Pattern")]
pub struct PyPattern {
    wrapped: Box<Pattern>,
}

impl<'py> IntoPyObject<'py> for Pattern {
    type Target = PyPattern;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Pattern::Var(_) => convert::<EVar, _>(py, self),
            Pattern::SVar(_) => convert::<SVar, _>(py, self),
            Pattern::Str(_) => convert::<KoreString, _>(py, self),
            Pattern::App(_) => convert::<App, _>(py, self),
            Pattern::LeftAssoc(_) => convert::<LeftAssoc, _>(py, self),
            Pattern::RightAssoc(_) => convert::<RightAssoc, _>(py, self),
            Pattern::Top(_) => convert::<Top, _>(py, self),
            Pattern::Bottom(_) => convert::<Bottom, _>(py, self),
            Pattern::Dv { .. } => convert::<DV, _>(py, self),
            Pattern::Not { .. } => convert::<Not, _>(py, self),
            Pattern::Implies { .. } => convert::<Implies, _>(py, self),
            Pattern::Iff { .. } => convert::<Iff, _>(py, self),
            Pattern::And { .. } => convert::<And, _>(py, self),
            Pattern::Or { .. } => convert::<Or, _>(py, self),
            Pattern::Exists { .. } => convert::<Exists, _>(py, self),
            Pattern::Forall { .. } => convert::<Forall, _>(py, self),
            Pattern::Mu { .. } => convert::<Mu, _>(py, self),
            Pattern::Nu { .. } => convert::<Nu, _>(py, self),
            Pattern::Ceil { .. } => convert::<Ceil, _>(py, self),
            Pattern::Floor { .. } => convert::<Floor, _>(py, self),
            Pattern::Equals { .. } => convert::<Equals, _>(py, self),
            Pattern::In { .. } => convert::<In, _>(py, self),
            Pattern::Next { .. } => convert::<Next, _>(py, self),
            Pattern::Rewrites { .. } => convert::<Rewrites, _>(py, self),
        }
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Pattern {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        Ok(obj
            .cast::<PyPattern>()
            .map(|pat| *pat.borrow().wrapped.clone())?)
    }
}

impl Wrappable<Pattern> for PyPattern {
    fn wrap(_py: Python<'_>, rust: &Pattern) -> PyResult<Self> {
        Ok(Self {
            wrapped: rust.clone().into(),
        })
    }
}

#[pymethods]
impl PyPattern {
    #[staticmethod]
    fn parse<'py>(py: Python<'py>, s: &str) -> PyResult<Bound<'py, Self>> {
        use crate::kore::Parser;

        let pattern: Pattern = Parser::new(s)
            .and_then(|mut p| p.pattern())
            .map_err(PyValueError::new_err)?;

        pattern.into_pyobject(py)
    }

    #[staticmethod]
    fn from_dict<'py>(dict: &Bound<'py, PyAny>) -> PyResult<Bound<'py, Self>> {
        let value = pyobject_to_serde_value(dict)?;
        let pattern: Pattern =
            serde_json::from_value(value).map_err(|e| PyValueError::new_err(e.to_string()))?;

        pattern.into_pyobject(dict.py())
    }

    fn dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let value = serde_json::to_value(self.wrapped.as_ref())
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        serde_value_to_pyobject(py, &value)
            .and_then(|obj| Ok(obj.cast_bound::<PyDict>(py)?.clone().unbind()))
    }
}

// --- EVar ---

#[pyclass(extends = PyPattern)]
pub struct EVar {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    sort: Sort,
}

impl Wrappable<Pattern> for EVar {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Var(var) => Ok(Self {
                name: var.id.clone().value(),
                sort: var.sort.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl EVar {
    #[new]
    fn new_(py: Python<'_>, name: String, sort: Sort) -> PyResult<Py<PyPattern>> {
        let id = Id::new(name).map_err(PyValueError::new_err)?;
        let pattern = Pattern::Var(Var { id, sort });
        pattern.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("name", py.get_type::<PyString>())?;
        annotations.set_item("sort", py.get_type::<PySort>())?;
        Ok(annotations)
    }
}

// --- SVar ---

#[pyclass(extends = PyPattern, name = "SVar")]
pub struct SVar {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    sort: Sort,
}

impl Wrappable<Pattern> for SVar {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::SVar(svar) => Ok(Self {
                name: svar.id.clone().value(),
                sort: svar.sort.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl SVar {
    #[new]
    fn new_(py: Python<'_>, name: String, sort: Sort) -> PyResult<Py<PyPattern>> {
        let id = SetVarId::new(name).map_err(PyValueError::new_err)?;
        let pattern = Pattern::SVar(crate::kore::SVar { id, sort });
        pattern.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("name", py.get_type::<PyString>())?;
        annotations.set_item("sort", py.get_type::<PySort>())?;
        Ok(annotations)
    }
}

// --- String (KoreString) ---

#[pyclass(extends = PyPattern, name = "String")]
pub struct KoreString {
    #[pyo3(get)]
    value: String,
}

impl Wrappable<Pattern> for KoreString {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Str(s) => Ok(Self {
                value: s.0.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl KoreString {
    #[new]
    fn new_(py: Python<'_>, value: String) -> PyResult<Py<PyPattern>> {
        let pattern = Pattern::Str(Str(value));
        pattern.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("value", py.get_type::<PyString>())?;
        Ok(annotations)
    }
}

// --- App ---

#[pyclass(extends = PyPattern, name = "App")]
pub struct App {
    #[pyo3(get)]
    symbol: String,
    #[pyo3(get)]
    sorts: Vec<Sort>,
    #[pyo3(get)]
    args: Vec<Pattern>,
}

impl Wrappable<Pattern> for App {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::App(app) => Ok(Self {
                symbol: app.symbol.clone().value(),
                sorts: app.sorts.clone(),
                args: app.args.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl App {
    #[new]
    #[pyo3(signature = (symbol, sorts=None, args=None))]
    fn new_(
        py: Python<'_>,
        symbol: String,
        sorts: Option<Vec<Sort>>,
        args: Option<Vec<Pattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let symbol = SymbolId::new(symbol).map_err(PyValueError::new_err)?;
        let pattern = Pattern::App(crate::kore::App {
            symbol,
            sorts: sorts.unwrap_or_default(),
            args: args.unwrap_or_default(),
        });
        pattern.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("symbol", py.get_type::<PyString>())?;
        annotations.set_item("sorts", py.get_type::<PyTuple>())?;
        annotations.set_item("args", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

// --- LeftAssoc ---

#[pyclass(extends = PyPattern)]
pub struct LeftAssoc {
    #[pyo3(get)]
    symbol: String,
    #[pyo3(get)]
    sorts: Vec<Sort>,
    #[pyo3(get)]
    args: Vec<Pattern>,
}

impl Wrappable<Pattern> for LeftAssoc {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::LeftAssoc(app) => Ok(Self {
                symbol: app.symbol.clone().value(),
                sorts: app.sorts.clone(),
                args: app.args.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl LeftAssoc {
    #[new]
    #[pyo3(signature = (symbol, sorts=None, args=None))]
    fn new_(
        py: Python<'_>,
        symbol: String,
        sorts: Option<Vec<Sort>>,
        args: Option<Vec<Pattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let symbol = SymbolId::new(symbol).map_err(PyValueError::new_err)?;
        let pattern = Pattern::LeftAssoc(crate::kore::App {
            symbol,
            sorts: sorts.unwrap_or_default(),
            args: args.unwrap_or_default(),
        });
        pattern.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("symbol", py.get_type::<PyString>())?;
        annotations.set_item("sorts", py.get_type::<PyTuple>())?;
        annotations.set_item("args", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

// --- RightAssoc ---

#[pyclass(extends = PyPattern)]
pub struct RightAssoc {
    #[pyo3(get)]
    symbol: String,
    #[pyo3(get)]
    sorts: Vec<Sort>,
    #[pyo3(get)]
    args: Vec<Pattern>,
}

impl Wrappable<Pattern> for RightAssoc {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::RightAssoc(app) => Ok(Self {
                symbol: app.symbol.clone().value(),
                sorts: app.sorts.clone(),
                args: app.args.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl RightAssoc {
    #[new]
    #[pyo3(signature = (symbol, sorts=None, args=None))]
    fn new_(
        py: Python<'_>,
        symbol: String,
        sorts: Option<Vec<Sort>>,
        args: Option<Vec<Pattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let symbol = SymbolId::new(symbol).map_err(PyValueError::new_err)?;
        let pattern = Pattern::RightAssoc(crate::kore::App {
            symbol,
            sorts: sorts.unwrap_or_default(),
            args: args.unwrap_or_default(),
        });
        pattern.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("symbol", py.get_type::<PyString>())?;
        annotations.set_item("sorts", py.get_type::<PyTuple>())?;
        annotations.set_item("args", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

// --- Top ---

#[pyclass(extends = PyPattern)]
pub struct Top {
    #[pyo3(get)]
    sort: Sort,
}

impl Wrappable<Pattern> for Top {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Top(sort) => Ok(Self { sort: sort.clone() }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Top {
    #[new]
    fn new_(py: Python<'_>, sort: Sort) -> PyResult<Py<PyPattern>> {
        let pattern = Pattern::Top(sort);
        pattern.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        Ok(annotations)
    }
}

// --- Bottom ---

#[pyclass(extends = PyPattern)]
pub struct Bottom {
    #[pyo3(get)]
    sort: Sort,
}

impl Wrappable<Pattern> for Bottom {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Bottom(sort) => Ok(Self { sort: sort.clone() }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Bottom {
    #[new]
    fn new_(py: Python<'_>, sort: Sort) -> PyResult<Py<PyPattern>> {
        let pattern = Pattern::Bottom(sort);
        pattern.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        Ok(annotations)
    }
}

// --- DV ---

#[pyclass(extends = PyPattern)]
pub struct DV {
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    value: Pattern,
}

impl Wrappable<Pattern> for DV {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Dv { sort, value } => Ok(Self {
                sort: sort.clone(),
                value: Pattern::Str(value.clone()),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl DV {
    #[new]
    fn new_(py: Python<'_>, sort: Sort, value: Pattern) -> PyResult<Py<PyPattern>> {
        let str_value = match value {
            Pattern::Str(s) => s,
            _ => {
                return Err(PyTypeError::new_err(
                    "value must be a String pattern",
                ))
            }
        };
        let pattern = Pattern::Dv {
            sort,
            value: str_value,
        };
        pattern.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("value", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Not ---

#[pyclass(extends = PyPattern)]
pub struct Not {
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    pattern: Pattern,
}

impl Wrappable<Pattern> for Not {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Not { sort, op } => Ok(Self {
                sort: sort.clone(),
                pattern: *op.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Not {
    #[new]
    fn new_(py: Python<'_>, sort: Sort, pattern: Pattern) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::Not {
            sort,
            op: Box::new(pattern),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("pattern", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Next ---

#[pyclass(extends = PyPattern)]
pub struct Next {
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    pattern: Pattern,
}

impl Wrappable<Pattern> for Next {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Next { sort, op } => Ok(Self {
                sort: sort.clone(),
                pattern: *op.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Next {
    #[new]
    fn new_(py: Python<'_>, sort: Sort, pattern: Pattern) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::Next {
            sort,
            op: Box::new(pattern),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("pattern", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Implies ---

#[pyclass(extends = PyPattern)]
pub struct Implies {
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    left: Pattern,
    #[pyo3(get)]
    right: Pattern,
}

impl Wrappable<Pattern> for Implies {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Implies { sort, left, right } => Ok(Self {
                sort: sort.clone(),
                left: *left.clone(),
                right: *right.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Implies {
    #[new]
    fn new_(
        py: Python<'_>,
        sort: Sort,
        left: Pattern,
        right: Pattern,
    ) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::Implies {
            sort,
            left: Box::new(left),
            right: Box::new(right),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("left", py.get_type::<PyPattern>())?;
        annotations.set_item("right", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Iff ---

#[pyclass(extends = PyPattern)]
pub struct Iff {
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    left: Pattern,
    #[pyo3(get)]
    right: Pattern,
}

impl Wrappable<Pattern> for Iff {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Iff { sort, left, right } => Ok(Self {
                sort: sort.clone(),
                left: *left.clone(),
                right: *right.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Iff {
    #[new]
    fn new_(
        py: Python<'_>,
        sort: Sort,
        left: Pattern,
        right: Pattern,
    ) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::Iff {
            sort,
            left: Box::new(left),
            right: Box::new(right),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("left", py.get_type::<PyPattern>())?;
        annotations.set_item("right", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Rewrites ---

#[pyclass(extends = PyPattern)]
pub struct Rewrites {
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    left: Pattern,
    #[pyo3(get)]
    right: Pattern,
}

impl Wrappable<Pattern> for Rewrites {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Rewrites { sort, left, right } => Ok(Self {
                sort: sort.clone(),
                left: *left.clone(),
                right: *right.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Rewrites {
    #[new]
    fn new_(
        py: Python<'_>,
        sort: Sort,
        left: Pattern,
        right: Pattern,
    ) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::Rewrites {
            sort,
            left: Box::new(left),
            right: Box::new(right),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("left", py.get_type::<PyPattern>())?;
        annotations.set_item("right", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- And ---

#[pyclass(extends = PyPattern)]
pub struct And {
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    ops: Vec<Pattern>,
}

impl Wrappable<Pattern> for And {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::And { sort, ops } => Ok(Self {
                sort: sort.clone(),
                ops: ops.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl And {
    #[new]
    fn new_(py: Python<'_>, sort: Sort, ops: Vec<Pattern>) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::And { sort, ops };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("ops", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

// --- Or ---

#[pyclass(extends = PyPattern)]
pub struct Or {
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    ops: Vec<Pattern>,
}

impl Wrappable<Pattern> for Or {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Or { sort, ops } => Ok(Self {
                sort: sort.clone(),
                ops: ops.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Or {
    #[new]
    fn new_(py: Python<'_>, sort: Sort, ops: Vec<Pattern>) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::Or { sort, ops };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("ops", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

// --- Exists ---

#[pyclass(extends = PyPattern)]
pub struct Exists {
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    var: Pattern,
    #[pyo3(get)]
    pattern: Pattern,
}

impl Wrappable<Pattern> for Exists {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Exists { sort, var, op } => Ok(Self {
                sort: sort.clone(),
                var: Pattern::Var(var.clone()),
                pattern: *op.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Exists {
    #[new]
    fn new_(
        py: Python<'_>,
        sort: Sort,
        var: Pattern,
        pattern: Pattern,
    ) -> PyResult<Py<PyPattern>> {
        let var = match var {
            Pattern::Var(v) => v,
            _ => {
                return Err(PyTypeError::new_err(
                    "var must be an EVar pattern",
                ))
            }
        };
        let pat = Pattern::Exists {
            sort,
            var,
            op: Box::new(pattern),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("var", py.get_type::<PyPattern>())?;
        annotations.set_item("pattern", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Forall ---

#[pyclass(extends = PyPattern)]
pub struct Forall {
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    var: Pattern,
    #[pyo3(get)]
    pattern: Pattern,
}

impl Wrappable<Pattern> for Forall {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Forall { sort, var, op } => Ok(Self {
                sort: sort.clone(),
                var: Pattern::Var(var.clone()),
                pattern: *op.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Forall {
    #[new]
    fn new_(
        py: Python<'_>,
        sort: Sort,
        var: Pattern,
        pattern: Pattern,
    ) -> PyResult<Py<PyPattern>> {
        let var = match var {
            Pattern::Var(v) => v,
            _ => {
                return Err(PyTypeError::new_err(
                    "var must be an EVar pattern",
                ))
            }
        };
        let pat = Pattern::Forall {
            sort,
            var,
            op: Box::new(pattern),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("var", py.get_type::<PyPattern>())?;
        annotations.set_item("pattern", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Mu ---

#[pyclass(extends = PyPattern)]
pub struct Mu {
    #[pyo3(get)]
    var: Pattern,
    #[pyo3(get)]
    pattern: Pattern,
}

impl Wrappable<Pattern> for Mu {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Mu { var, op } => Ok(Self {
                var: Pattern::SVar(var.clone()),
                pattern: *op.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Mu {
    #[new]
    fn new_(py: Python<'_>, var: Pattern, pattern: Pattern) -> PyResult<Py<PyPattern>> {
        let var = match var {
            Pattern::SVar(v) => v,
            _ => {
                return Err(PyTypeError::new_err(
                    "var must be an SVar pattern",
                ))
            }
        };
        let pat = Pattern::Mu {
            var,
            op: Box::new(pattern),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("var", py.get_type::<PyPattern>())?;
        annotations.set_item("pattern", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Nu ---

#[pyclass(extends = PyPattern)]
pub struct Nu {
    #[pyo3(get)]
    var: Pattern,
    #[pyo3(get)]
    pattern: Pattern,
}

impl Wrappable<Pattern> for Nu {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Nu { var, op } => Ok(Self {
                var: Pattern::SVar(var.clone()),
                pattern: *op.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Nu {
    #[new]
    fn new_(py: Python<'_>, var: Pattern, pattern: Pattern) -> PyResult<Py<PyPattern>> {
        let var = match var {
            Pattern::SVar(v) => v,
            _ => {
                return Err(PyTypeError::new_err(
                    "var must be an SVar pattern",
                ))
            }
        };
        let pat = Pattern::Nu {
            var,
            op: Box::new(pattern),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("var", py.get_type::<PyPattern>())?;
        annotations.set_item("pattern", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Ceil ---

#[pyclass(extends = PyPattern)]
pub struct Ceil {
    #[pyo3(get)]
    op_sort: Sort,
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    pattern: Pattern,
}

impl Wrappable<Pattern> for Ceil {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Ceil { op_sort, sort, op } => Ok(Self {
                op_sort: op_sort.clone(),
                sort: sort.clone(),
                pattern: *op.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Ceil {
    #[new]
    fn new_(
        py: Python<'_>,
        op_sort: Sort,
        sort: Sort,
        pattern: Pattern,
    ) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::Ceil {
            op_sort,
            sort,
            op: Box::new(pattern),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("op_sort", py.get_type::<PySort>())?;
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("pattern", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Floor ---

#[pyclass(extends = PyPattern)]
pub struct Floor {
    #[pyo3(get)]
    op_sort: Sort,
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    pattern: Pattern,
}

impl Wrappable<Pattern> for Floor {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Floor { op_sort, sort, op } => Ok(Self {
                op_sort: op_sort.clone(),
                sort: sort.clone(),
                pattern: *op.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Floor {
    #[new]
    fn new_(
        py: Python<'_>,
        op_sort: Sort,
        sort: Sort,
        pattern: Pattern,
    ) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::Floor {
            op_sort,
            sort,
            op: Box::new(pattern),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("op_sort", py.get_type::<PySort>())?;
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("pattern", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- Equals ---

#[pyclass(extends = PyPattern)]
pub struct Equals {
    #[pyo3(get)]
    op_sort: Sort,
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    left: Pattern,
    #[pyo3(get)]
    right: Pattern,
}

impl Wrappable<Pattern> for Equals {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::Equals {
                op_sort,
                sort,
                left,
                right,
            } => Ok(Self {
                op_sort: op_sort.clone(),
                sort: sort.clone(),
                left: *left.clone(),
                right: *right.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Equals {
    #[new]
    fn new_(
        py: Python<'_>,
        op_sort: Sort,
        sort: Sort,
        left: Pattern,
        right: Pattern,
    ) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::Equals {
            op_sort,
            sort,
            left: Box::new(left),
            right: Box::new(right),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("op_sort", py.get_type::<PySort>())?;
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("left", py.get_type::<PyPattern>())?;
        annotations.set_item("right", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// --- In ---

#[pyclass(extends = PyPattern)]
pub struct In {
    #[pyo3(get)]
    op_sort: Sort,
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    left: Pattern,
    #[pyo3(get)]
    right: Pattern,
}

impl Wrappable<Pattern> for In {
    fn wrap(_py: Python<'_>, it: &Pattern) -> PyResult<Self> {
        match it {
            Pattern::In {
                op_sort,
                sort,
                left,
                right,
            } => Ok(Self {
                op_sort: op_sort.clone(),
                sort: sort.clone(),
                left: *left.clone(),
                right: *right.clone(),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl In {
    #[new]
    fn new_(
        py: Python<'_>,
        op_sort: Sort,
        sort: Sort,
        left: Pattern,
        right: Pattern,
    ) -> PyResult<Py<PyPattern>> {
        let pat = Pattern::In {
            op_sort,
            sort,
            left: Box::new(left),
            right: Box::new(right),
        };
        pat.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("op_sort", py.get_type::<PySort>())?;
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("left", py.get_type::<PyPattern>())?;
        annotations.set_item("right", py.get_type::<PyPattern>())?;
        Ok(annotations)
    }
}

// ==========================================
// Sentence bindings
// ==========================================

// --- Helpers for converting between Rust and Python representations ---

fn apps_to_patterns(apps: &[crate::kore::App]) -> Vec<Pattern> {
    apps.iter().map(|a| Pattern::App(a.clone())).collect()
}

fn ids_to_sort_vars(ids: &[Id]) -> Vec<Sort> {
    ids.iter().map(|id| Sort::Var(id.clone())).collect()
}

fn patterns_to_apps(patterns: Vec<Pattern>) -> PyResult<Vec<crate::kore::App>> {
    patterns
        .into_iter()
        .map(|p| match p {
            Pattern::App(a) => Ok(a),
            _ => Err(PyTypeError::new_err("attrs must be App patterns")),
        })
        .collect()
}

fn sort_vars_to_ids(sorts: Vec<Sort>) -> PyResult<Vec<Id>> {
    sorts
        .into_iter()
        .map(|s| match s {
            Sort::Var(id) => Ok(id),
            _ => Err(PyTypeError::new_err("vars must be SortVar")),
        })
        .collect()
}

// --- Symbol (standalone pyclass, not a Sentence subclass) ---

#[pyclass]
pub struct Symbol {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    vars: Vec<Sort>,
}

#[pymethods]
impl Symbol {
    #[new]
    #[pyo3(signature = (name, vars=None))]
    fn new_(name: String, vars: Option<Vec<Sort>>) -> PyResult<Self> {
        SymbolId::new(name.clone()).map_err(PyValueError::new_err)?;
        let vars = vars.unwrap_or_default();
        for v in &vars {
            if !matches!(v, Sort::Var(_)) {
                return Err(PyTypeError::new_err("vars must be SortVar"));
            }
        }
        Ok(Symbol { name, vars })
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("name", py.get_type::<PyString>())?;
        annotations.set_item("vars", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

/// Helper type for storing Symbol data in Sentence subclass fields.
/// Implements IntoPyObject/FromPyObject to convert to/from the Symbol pyclass.
#[derive(Clone)]
struct SymbolData {
    name: String,
    vars: Vec<Sort>,
}

impl<'py> IntoPyObject<'py> for SymbolData {
    type Target = Symbol;
    type Output = Bound<'py, Symbol>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Bound::new(
            py,
            Symbol {
                name: self.name,
                vars: self.vars,
            },
        )
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for SymbolData {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let sym = obj.cast::<Symbol>()?;
        let borrow = sym.borrow();
        Ok(SymbolData {
            name: borrow.name.clone(),
            vars: borrow.vars.clone(),
        })
    }
}

// --- PySentence (base class) ---

#[pyclass(subclass, name = "Sentence")]
pub struct PySentence {
    wrapped: Box<Sentence>,
}

impl<'py> IntoPyObject<'py> for Sentence {
    type Target = PySentence;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Sentence::Import { .. } => convert::<Import, _>(py, self),
            Sentence::Sort { .. } => convert::<SortDecl, _>(py, self),
            Sentence::Symbol { .. } => convert::<SymbolDecl, _>(py, self),
            Sentence::Alias { .. } => convert::<AliasDecl, _>(py, self),
            Sentence::Axiom { .. } => convert::<Axiom, _>(py, self),
            Sentence::Claim { .. } => convert::<Claim, _>(py, self),
        }
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Sentence {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        Ok(obj
            .cast::<PySentence>()
            .map(|s| *s.borrow().wrapped.clone())?)
    }
}

impl Wrappable<Sentence> for PySentence {
    fn wrap(_py: Python<'_>, rust: &Sentence) -> PyResult<Self> {
        Ok(Self {
            wrapped: rust.clone().into(),
        })
    }
}

#[pymethods]
impl PySentence {
    #[staticmethod]
    fn parse<'py>(py: Python<'py>, s: &str) -> PyResult<Bound<'py, Self>> {
        use crate::kore::Parser;

        let sentence: Sentence = Parser::new(s)
            .and_then(|mut p| p.sentence())
            .map_err(PyValueError::new_err)?;

        sentence.into_pyobject(py)
    }
}

// --- Import ---

#[pyclass(extends = PySentence)]
pub struct Import {
    #[pyo3(get)]
    module_name: String,
    #[pyo3(get)]
    attrs: Vec<Pattern>,
}

impl Wrappable<Sentence> for Import {
    fn wrap(_py: Python<'_>, it: &Sentence) -> PyResult<Self> {
        match it {
            Sentence::Import { module, attrs } => Ok(Self {
                module_name: module.clone().value(),
                attrs: apps_to_patterns(attrs),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Import {
    #[new]
    #[pyo3(signature = (module_name, attrs=None))]
    fn new_(
        py: Python<'_>,
        module_name: String,
        attrs: Option<Vec<Pattern>>,
    ) -> PyResult<Py<PySentence>> {
        let module = Id::new(module_name).map_err(PyValueError::new_err)?;
        let attrs = patterns_to_apps(attrs.unwrap_or_default())?;
        let sentence = Sentence::Import { module, attrs };
        sentence.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("module_name", py.get_type::<PyString>())?;
        annotations.set_item("attrs", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

// --- SortDecl ---

#[pyclass(extends = PySentence)]
pub struct SortDecl {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    vars: Vec<Sort>,
    #[pyo3(get)]
    attrs: Vec<Pattern>,
    #[pyo3(get)]
    hooked: bool,
}

impl Wrappable<Sentence> for SortDecl {
    fn wrap(_py: Python<'_>, it: &Sentence) -> PyResult<Self> {
        match it {
            Sentence::Sort {
                id,
                vars,
                attrs,
                hooked,
            } => Ok(Self {
                name: id.clone().value(),
                vars: ids_to_sort_vars(vars),
                attrs: apps_to_patterns(attrs),
                hooked: *hooked,
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl SortDecl {
    #[new]
    #[pyo3(signature = (name, vars, attrs=None, *, hooked=false))]
    fn new_(
        py: Python<'_>,
        name: String,
        vars: Vec<Sort>,
        attrs: Option<Vec<Pattern>>,
        hooked: bool,
    ) -> PyResult<Py<PySentence>> {
        let id = Id::new(name).map_err(PyValueError::new_err)?;
        let vars = sort_vars_to_ids(vars)?;
        let attrs = patterns_to_apps(attrs.unwrap_or_default())?;
        let sentence = Sentence::Sort {
            id,
            vars,
            attrs,
            hooked,
        };
        sentence.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("name", py.get_type::<PyString>())?;
        annotations.set_item("vars", py.get_type::<PyTuple>())?;
        annotations.set_item("attrs", py.get_type::<PyTuple>())?;
        annotations.set_item("hooked", py.get_type::<PyBool>())?;
        Ok(annotations)
    }
}

// --- SymbolDecl ---

#[pyclass(extends = PySentence)]
pub struct SymbolDecl {
    #[pyo3(get)]
    symbol: SymbolData,
    #[pyo3(get)]
    param_sorts: Vec<Sort>,
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    attrs: Vec<Pattern>,
    #[pyo3(get)]
    hooked: bool,
}

impl Wrappable<Sentence> for SymbolDecl {
    fn wrap(_py: Python<'_>, it: &Sentence) -> PyResult<Self> {
        match it {
            Sentence::Symbol {
                id,
                vars,
                param_sorts,
                sort,
                attrs,
                hooked,
            } => Ok(Self {
                symbol: SymbolData {
                    name: id.clone().value(),
                    vars: ids_to_sort_vars(vars),
                },
                param_sorts: param_sorts.clone(),
                sort: sort.clone(),
                attrs: apps_to_patterns(attrs),
                hooked: *hooked,
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl SymbolDecl {
    #[new]
    #[pyo3(signature = (symbol, param_sorts, sort, attrs=None, *, hooked=false))]
    fn new_(
        py: Python<'_>,
        symbol: SymbolData,
        param_sorts: Vec<Sort>,
        sort: Sort,
        attrs: Option<Vec<Pattern>>,
        hooked: bool,
    ) -> PyResult<Py<PySentence>> {
        let id = SymbolId::new(symbol.name).map_err(PyValueError::new_err)?;
        let vars = sort_vars_to_ids(symbol.vars)?;
        let attrs = patterns_to_apps(attrs.unwrap_or_default())?;
        let sentence = Sentence::Symbol {
            id,
            vars,
            param_sorts,
            sort,
            attrs,
            hooked,
        };
        sentence.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("symbol", py.get_type::<Symbol>())?;
        annotations.set_item("param_sorts", py.get_type::<PyTuple>())?;
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("attrs", py.get_type::<PyTuple>())?;
        annotations.set_item("hooked", py.get_type::<PyBool>())?;
        Ok(annotations)
    }
}

// --- AliasDecl ---

#[pyclass(extends = PySentence)]
pub struct AliasDecl {
    #[pyo3(get)]
    alias: SymbolData,
    #[pyo3(get)]
    param_sorts: Vec<Sort>,
    #[pyo3(get)]
    sort: Sort,
    #[pyo3(get)]
    left: Pattern,
    #[pyo3(get)]
    right: Pattern,
    #[pyo3(get)]
    attrs: Vec<Pattern>,
}

impl Wrappable<Sentence> for AliasDecl {
    fn wrap(_py: Python<'_>, it: &Sentence) -> PyResult<Self> {
        match it {
            Sentence::Alias {
                id,
                vars,
                param_sorts,
                sort,
                left,
                right,
                attrs,
            } => Ok(Self {
                alias: SymbolData {
                    name: id.clone().value(),
                    vars: ids_to_sort_vars(vars),
                },
                param_sorts: param_sorts.clone(),
                sort: sort.clone(),
                left: Pattern::App(left.clone()),
                right: *right.clone(),
                attrs: apps_to_patterns(attrs),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl AliasDecl {
    #[new]
    #[pyo3(signature = (alias, param_sorts, sort, left, right, attrs=None))]
    fn new_(
        py: Python<'_>,
        alias: SymbolData,
        param_sorts: Vec<Sort>,
        sort: Sort,
        left: Pattern,
        right: Pattern,
        attrs: Option<Vec<Pattern>>,
    ) -> PyResult<Py<PySentence>> {
        let id = SymbolId::new(alias.name).map_err(PyValueError::new_err)?;
        let vars = sort_vars_to_ids(alias.vars)?;
        let left = match left {
            Pattern::App(a) => a,
            _ => return Err(PyTypeError::new_err("left must be an App pattern")),
        };
        let attrs = patterns_to_apps(attrs.unwrap_or_default())?;
        let sentence = Sentence::Alias {
            id,
            vars,
            param_sorts,
            sort,
            left,
            right: Box::new(right),
            attrs,
        };
        sentence.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("alias", py.get_type::<Symbol>())?;
        annotations.set_item("param_sorts", py.get_type::<PyTuple>())?;
        annotations.set_item("sort", py.get_type::<PySort>())?;
        annotations.set_item("left", py.get_type::<PyPattern>())?;
        annotations.set_item("right", py.get_type::<PyPattern>())?;
        annotations.set_item("attrs", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

// --- Axiom ---

#[pyclass(extends = PySentence)]
pub struct Axiom {
    #[pyo3(get)]
    vars: Vec<Sort>,
    #[pyo3(get)]
    pattern: Pattern,
    #[pyo3(get)]
    attrs: Vec<Pattern>,
}

impl Wrappable<Sentence> for Axiom {
    fn wrap(_py: Python<'_>, it: &Sentence) -> PyResult<Self> {
        match it {
            Sentence::Axiom {
                vars,
                pattern,
                attrs,
            } => Ok(Self {
                vars: ids_to_sort_vars(vars),
                pattern: *pattern.clone(),
                attrs: apps_to_patterns(attrs),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Axiom {
    #[new]
    #[pyo3(signature = (vars, pattern, attrs=None))]
    fn new_(
        py: Python<'_>,
        vars: Vec<Sort>,
        pattern: Pattern,
        attrs: Option<Vec<Pattern>>,
    ) -> PyResult<Py<PySentence>> {
        let vars = sort_vars_to_ids(vars)?;
        let attrs = patterns_to_apps(attrs.unwrap_or_default())?;
        let sentence = Sentence::Axiom {
            vars,
            pattern: Box::new(pattern),
            attrs,
        };
        sentence.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("vars", py.get_type::<PyTuple>())?;
        annotations.set_item("pattern", py.get_type::<PyPattern>())?;
        annotations.set_item("attrs", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

// --- Claim ---

#[pyclass(extends = PySentence)]
pub struct Claim {
    #[pyo3(get)]
    vars: Vec<Sort>,
    #[pyo3(get)]
    pattern: Pattern,
    #[pyo3(get)]
    attrs: Vec<Pattern>,
}

impl Wrappable<Sentence> for Claim {
    fn wrap(_py: Python<'_>, it: &Sentence) -> PyResult<Self> {
        match it {
            Sentence::Claim {
                vars,
                pattern,
                attrs,
            } => Ok(Self {
                vars: ids_to_sort_vars(vars),
                pattern: *pattern.clone(),
                attrs: apps_to_patterns(attrs),
            }),
            _ => Err(PyTypeError::new_err(
                "Attempted to create wrapped value from wrong base value",
            )),
        }
    }
}

#[pymethods]
impl Claim {
    #[new]
    #[pyo3(signature = (vars, pattern, attrs=None))]
    fn new_(
        py: Python<'_>,
        vars: Vec<Sort>,
        pattern: Pattern,
        attrs: Option<Vec<Pattern>>,
    ) -> PyResult<Py<PySentence>> {
        let vars = sort_vars_to_ids(vars)?;
        let attrs = patterns_to_apps(attrs.unwrap_or_default())?;
        let sentence = Sentence::Claim {
            vars,
            pattern: Box::new(pattern),
            attrs,
        };
        sentence.into_pyobject(py).map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("vars", py.get_type::<PyTuple>())?;
        annotations.set_item("pattern", py.get_type::<PyPattern>())?;
        annotations.set_item("attrs", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

// ==========================================
// Module bindings
// ==========================================

#[pyclass(name = "Module")]
pub struct KoreModule {
    #[pyo3(get)]
    name: String,
    #[pyo3(get)]
    sentences: Vec<Sentence>,
    #[pyo3(get)]
    attrs: Vec<Pattern>,
}

impl<'py> IntoPyObject<'py> for crate::kore::Module {
    type Target = KoreModule;
    type Output = Bound<'py, KoreModule>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Bound::new(
            py,
            KoreModule {
                name: self.id.value(),
                sentences: self.sentences,
                attrs: apps_to_patterns(&self.attrs),
            },
        )
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for crate::kore::Module {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let m = obj.cast::<KoreModule>()?;
        let borrow = m.borrow();
        let id = Id::new(borrow.name.clone()).map_err(PyValueError::new_err)?;
        let attrs = patterns_to_apps(borrow.attrs.clone())?;
        Ok(crate::kore::Module {
            id,
            sentences: borrow.sentences.clone(),
            attrs,
        })
    }
}

#[pymethods]
impl KoreModule {
    #[new]
    #[pyo3(signature = (name, sentences=None, attrs=None))]
    fn new_(
        py: Python<'_>,
        name: String,
        sentences: Option<Vec<Sentence>>,
        attrs: Option<Vec<Pattern>>,
    ) -> PyResult<Py<KoreModule>> {
        let id = Id::new(name.clone()).map_err(PyValueError::new_err)?;
        let attrs_apps = patterns_to_apps(attrs.unwrap_or_default())?;
        let module = crate::kore::Module {
            id,
            sentences: sentences.unwrap_or_default(),
            attrs: attrs_apps,
        };
        module.into_pyobject(py).map(Bound::unbind)
    }

    #[staticmethod]
    fn parse<'py>(py: Python<'py>, s: &str) -> PyResult<Bound<'py, KoreModule>> {
        use crate::kore::Parser;

        let module: crate::kore::Module = Parser::new(s)
            .and_then(|mut p| p.module())
            .map_err(PyValueError::new_err)?;

        module.into_pyobject(py)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("name", py.get_type::<PyString>())?;
        annotations.set_item("sentences", py.get_type::<PyTuple>())?;
        annotations.set_item("attrs", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}

// ==========================================
// Definition bindings
// ==========================================

#[pyclass(name = "Definition")]
pub struct KoreDefinition {
    #[pyo3(get)]
    modules: Vec<crate::kore::Module>,
    #[pyo3(get)]
    attrs: Vec<Pattern>,
}

impl<'py> IntoPyObject<'py> for crate::kore::Definition {
    type Target = KoreDefinition;
    type Output = Bound<'py, KoreDefinition>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Bound::new(
            py,
            KoreDefinition {
                modules: self.modules,
                attrs: apps_to_patterns(&self.attrs),
            },
        )
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for crate::kore::Definition {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let d = obj.cast::<KoreDefinition>()?;
        let borrow = d.borrow();
        let attrs = patterns_to_apps(borrow.attrs.clone())?;
        Ok(crate::kore::Definition {
            modules: borrow.modules.clone(),
            attrs,
        })
    }
}

#[pymethods]
impl KoreDefinition {
    #[new]
    #[pyo3(signature = (modules=None, attrs=None))]
    fn new_(
        py: Python<'_>,
        modules: Option<Vec<crate::kore::Module>>,
        attrs: Option<Vec<Pattern>>,
    ) -> PyResult<Py<KoreDefinition>> {
        let attrs_apps = patterns_to_apps(attrs.unwrap_or_default())?;
        let definition = crate::kore::Definition {
            modules: modules.unwrap_or_default(),
            attrs: attrs_apps,
        };
        definition.into_pyobject(py).map(Bound::unbind)
    }

    #[staticmethod]
    fn parse<'py>(py: Python<'py>, s: &str) -> PyResult<Bound<'py, KoreDefinition>> {
        use crate::kore::Parser;

        let definition: crate::kore::Definition = Parser::new(s)
            .and_then(|mut p| p.definition())
            .map_err(PyValueError::new_err)?;

        definition.into_pyobject(py)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let annotations = PyDict::new(py);
        annotations.set_item("modules", py.get_type::<PyTuple>())?;
        annotations.set_item("attrs", py.get_type::<PyTuple>())?;
        Ok(annotations)
    }
}
