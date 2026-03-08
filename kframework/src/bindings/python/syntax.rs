use super::utils::{
    maybe_seq_to_tuple, py_tuple_to_sequence, seq_to_tuple, vec_to_pytuple, InnerValue,
};
use crate::kore::{Id, Pattern, SVar, Sentence, SetVarId, Sort, Str, SymbolId, Var};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{IntoPyDict, PyBool, PyDict, PySequence, PyString, PyTuple},
    PyClass,
};

trait Wrappable<RustType>: Sized
where
    RustType: Clone + std::fmt::Debug,
{
    fn wrap_into(py: Python<'_>, rust: RustType) -> PyResult<Self>;

    fn as_wrap(py: Python<'_>, rust: &RustType) -> PyResult<Self> {
        Self::wrap_into(py, rust.clone())
    }

    fn error(rust: RustType) -> PyResult<Self> {
        Err(PyTypeError::new_err(format!(
            "Error wrapping rust value into python object: {:?}",
            rust
        )))
    }
}

fn convert<SubClass, RustType>(
    py: Python<'_>,
    it: RustType,
) -> PyResult<Bound<'_, SubClass::BaseType>>
where
    RustType: Clone + std::fmt::Debug,
    SubClass: PyClass + Wrappable<RustType>,
    SubClass::BaseType: Wrappable<RustType>,
    (SubClass, SubClass::BaseType): Into<PyClassInitializer<SubClass>>,
{
    let s = SubClass::as_wrap(py, &it)?;
    convert_with_wrapped(py, it, s)
}

fn convert_with_wrapped<SubClass, RustType>(
    py: Python<'_>,
    it: RustType,
    wrapped: SubClass,
) -> PyResult<Bound<'_, SubClass::BaseType>>
where
    RustType: Clone + std::fmt::Debug,
    SubClass: PyClass + Wrappable<RustType>,
    SubClass::BaseType: Wrappable<RustType>,
    (SubClass, SubClass::BaseType): Into<PyClassInitializer<SubClass>>,
{
    let b = SubClass::BaseType::wrap_into(py, it)?;

    Ok(Bound::new(py, (wrapped, b))?.cast_into::<SubClass::BaseType>()?)
}

// ==========================================
// Into/FromPyObject impls for kore::syntax structs
// ==========================================

impl<'a, 'py> FromPyObject<'a, 'py> for Id {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let sortvar_try = obj.cast::<SortVar>();
        if let Ok(sortvar) = sortvar_try {
            let id: Id = sortvar.borrow().name.extract(obj.py())?;
            return Ok(id);
        }
        let s = obj.cast::<PyString>()?.to_string();
        Id::new(s).map_err(PyValueError::new_err)
    }
}

impl<'py> IntoPyObject<'py> for Id {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, &self.value()))
    }
}

impl<'py> Id {
    fn into_pysortvar(self, py: Python<'py>) -> PyResult<Py<SortVar>> {
        Sort::Var(self)
            .into_pyobject(py)?
            .extract()
            .map_err(Into::into)
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for SetVarId {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let s = obj.cast::<PyString>()?.to_string();
        s.try_into().map_err(PyValueError::new_err)
    }
}

impl<'py> IntoPyObject<'py> for SetVarId {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, &self.value()))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Str {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let s = obj.cast::<PyString>()?.to_string();
        Str::from_kore(&s).map_err(PyValueError::new_err)
    }
}

impl<'py> IntoPyObject<'py> for Str {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, &self.0))
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for SymbolId {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let s = obj.cast::<PyString>()?.to_string();
        s.try_into().map_err(PyValueError::new_err)
    }
}

impl<'py> IntoPyObject<'py> for SymbolId {
    type Target = PyString;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyString::new(py, &self.value()))
    }
}

impl<'py> IntoPyObject<'py> for Var {
    type Target = EVar;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Var(self);
        Ok(pat.into_pyobject(py)?.cast_into()?)
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for Var {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let var = obj.cast::<EVar>()?.borrow();
        let pat = *var.as_super().wrapped.clone();
        let Pattern::Var(var) = pat else {
            return Err(PyTypeError::new_err(format!(
                "Error converting python object into Var: {:?}",
                obj
            )));
        };
        Ok(var)
    }
}

impl<'py> IntoPyObject<'py> for SVar {
    type Target = PySVar;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::SVar(self);
        Ok(pat.into_pyobject(py)?.cast_into()?)
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for SVar {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let var = obj.cast::<PySVar>()?.borrow();
        let pat = *var.as_super().wrapped.clone();
        let Pattern::SVar(svar) = pat else {
            return Err(PyTypeError::new_err(format!(
                "Error converting python object into SVar, {:?}",
                obj
            )));
        };
        Ok(svar)
    }
}

impl<'py> IntoPyObject<'py> for crate::kore::App {
    type Target = App;
    type Output = Bound<'py, App>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::App(self);
        Ok(pat.into_pyobject(py)?.cast_into()?)
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for crate::kore::App {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let var = obj.cast::<App>()?.borrow();
        let pat = *var.as_super().wrapped.clone();
        let Pattern::App(app) = pat else {
            return Err(PyTypeError::new_err(format!(
                "Error converting python object into App: {:?}",
                obj
            )));
        };
        Ok(app)
    }
}

// ==========================================
// Sort bindings
// ==========================================

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
        let value: serde_json::Value = dict.extract::<InnerValue>()?.into();
        let sort: Sort =
            serde_json::from_value(value).map_err(|e| PyValueError::new_err(e.to_string()))?;

        sort.into_pyobject(dict.py())
    }

    fn dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let value = serde_json::to_value(self.wrapped.as_ref())
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(InnerValue(value)
            .into_pyobject(py)?
            .cast_into::<PyDict>()?
            .into())
    }
}

impl Wrappable<Sort> for PySort {
    fn wrap_into(_py: Python<'_>, rust: Sort) -> PyResult<Self> {
        Ok(Self {
            wrapped: rust.into(),
        })
    }
}

macro_rules! annotations {
    ( $py:ident, $( ($field:literal, $type:ident) ),* ) => {
        [
            $(
                ($field, $py.get_type::<$type>()),
            )*
        ].into_py_dict($py)
    }
}

#[pyclass(extends = PySort, get_all)]
pub struct SortVar {
    name: Py<PyString>,
}

#[pymethods]
impl SortVar {
    #[new]
    fn new_(py: Python<'_>, name: Py<PyString>) -> PyResult<Py<PySort>> {
        Self { name }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (name=None))]
    fn r#let(&self, py: Python<'_>, name: Option<Py<PyString>>) -> PyResult<Py<PySort>> {
        let name = name.unwrap_or_else(|| self.name.clone_ref(py));
        Self::new_(py, name)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("name", PyString))
    }
}

impl Wrappable<Sort> for SortVar {
    fn wrap_into(py: Python<'_>, it: Sort) -> PyResult<Self> {
        if let Sort::Var(id) = it {
            Ok(SortVar {
                name: id.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for SortVar {
    type Target = PySort;
    type Output = Bound<'py, PySort>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let id: Id = self.name.extract(py)?;
        let var = Sort::Var(id);
        convert_with_wrapped(py, var, self)
    }
}

#[pyclass(extends = PySort, get_all)]
pub struct SortApp {
    name: Py<PyString>,
    sorts: Py<PyTuple>,
}

#[pymethods]
impl SortApp {
    #[new]
    #[pyo3(signature = (name, sorts=None))]
    fn new_(
        py: Python<'_>,
        name: Py<PyString>,
        sorts: Option<Py<PySequence>>,
    ) -> PyResult<Py<PySort>> {
        Self {
            name,
            sorts: maybe_seq_to_tuple(py, sorts)?,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (name=None, sorts=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        name: Option<Py<PyString>>,
        sorts: Option<Py<PySequence>>,
    ) -> PyResult<Py<PySort>> {
        let name = name.unwrap_or_else(|| self.name.clone_ref(py));
        let sorts = sorts.or_else(|| Some(py_tuple_to_sequence(py, &self.sorts)));
        Self::new_(py, name, sorts)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("name", PyString), ("sorts", PyTuple))
    }
}

impl Wrappable<Sort> for SortApp {
    fn wrap_into(py: Python<'_>, it: Sort) -> PyResult<Self> {
        if let Sort::App { id, args } = it {
            Ok(Self {
                name: id.into_pyobject(py)?.into(),
                sorts: args.into_pyobject(py)?.extract()?,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for SortApp {
    type Target = PySort;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let app = Sort::App {
            id: self.name.extract(py)?,
            args: self.sorts.extract(py)?,
        };
        convert_with_wrapped(py, app, self)
    }
}

// ==========================================
// Pattern bindings
// ==========================================

impl<'py> IntoPyObject<'py> for Pattern {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            Pattern::Var(_) => convert::<EVar, _>(py, self),
            Pattern::SVar(_) => convert::<PySVar, _>(py, self),
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

#[pyclass(subclass, name = "Pattern")]
pub struct PyPattern {
    wrapped: Box<Pattern>,
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
        let value: serde_json::Value = dict.extract::<InnerValue>()?.into();
        let pattern: Pattern =
            serde_json::from_value(value).map_err(|e| PyValueError::new_err(e.to_string()))?;

        pattern.into_pyobject(dict.py())
    }

    fn dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let value = serde_json::to_value(self.wrapped.as_ref())
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(InnerValue(value)
            .into_pyobject(py)?
            .cast_into::<PyDict>()?
            .into())
    }

    fn let_patterns(slf: Bound<'_, Self>, pats: Vec<Pattern>) -> PyResult<Py<Self>> {
        slf.call_method1("let_patterns", (pats,))
            .map(|b| Ok(b.cast_into::<Self>()?.unbind()))?
    }
}

impl Wrappable<Pattern> for PyPattern {
    fn wrap_into(_py: Python<'_>, rust: Pattern) -> PyResult<Self> {
        Ok(Self {
            wrapped: rust.into(),
        })
    }
}

// --- EVar ---

#[pyclass(extends = PyPattern, get_all)]
pub struct EVar {
    name: Py<PyString>,
    sort: Py<PySort>,
}

#[pymethods]
impl EVar {
    #[new]
    fn new_(py: Python<'_>, name: Py<PyString>, sort: Py<PySort>) -> PyResult<Py<PyPattern>> {
        Self { name, sort }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (name=None, sort=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        name: Option<Py<PyString>>,
        sort: Option<Py<PySort>>,
    ) -> PyResult<Py<PyPattern>> {
        let name = name.unwrap_or_else(|| self.name.clone_ref(py));
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        Self::new_(py, name, sort)
    }

    fn let_patterns(
        slf: Bound<'_, Self>,
        _patterns: Bound<'_, PyTuple>,
    ) -> PyResult<Py<PyPattern>> {
        Ok(slf.cast_into()?.into())
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> Py<PyTuple> {
        PyTuple::empty(py).into()
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("name", PyString), ("sort", PySort))
    }
}

impl Wrappable<Pattern> for EVar {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Var(var) = it {
            Ok(Self {
                name: var.id.into_pyobject(py)?.into(),
                sort: var.sort.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for EVar {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let var = Var {
            id: self.name.extract(py)?,
            sort: self.sort.extract(py)?,
        };
        let pat = Pattern::Var(var);
        convert_with_wrapped(py, pat, self)
    }
}

// --- SVar ---

#[pyclass(extends = PyPattern, name = "SVar", get_all)]
pub struct PySVar {
    name: Py<PyString>,
    sort: Py<PySort>,
}

#[pymethods]
impl PySVar {
    #[new]
    fn new_(py: Python<'_>, name: Py<PyString>, sort: Py<PySort>) -> PyResult<Py<PyPattern>> {
        Self { name, sort }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (name=None, sort=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        name: Option<Py<PyString>>,
        sort: Option<Py<PySort>>,
    ) -> PyResult<Py<PyPattern>> {
        let name = name.unwrap_or_else(|| self.name.clone_ref(py));
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        Self::new_(py, name, sort)
    }

    fn let_patterns(
        slf: Bound<'_, Self>,
        _patterns: Bound<'_, PyTuple>,
    ) -> PyResult<Py<PyPattern>> {
        Ok(slf.cast_into()?.into())
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> Py<PyTuple> {
        PyTuple::empty(py).into()
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("name", PyString), ("sort", PySort))
    }
}

impl Wrappable<Pattern> for PySVar {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::SVar(svar) = it {
            Ok(Self {
                name: svar.id.into_pyobject(py)?.into(),
                sort: svar.sort.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for PySVar {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let svar = SVar {
            id: self.name.extract(py)?,
            sort: self.sort.extract(py)?,
        };
        let pat = Pattern::SVar(svar);
        convert_with_wrapped(py, pat, self)
    }
}

// --- String (KoreString) ---

#[pyclass(extends = PyPattern, name = "String", get_all)]
pub struct KoreString {
    value: Py<PyString>,
}

#[pymethods]
impl KoreString {
    #[new]
    fn new_(py: Python<'_>, value: Py<PyString>) -> PyResult<Py<PyPattern>> {
        Self { value }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (value=None))]
    fn r#let(&self, py: Python<'_>, value: Option<Py<PyString>>) -> PyResult<Py<PyPattern>> {
        let value = value.unwrap_or_else(|| self.value.clone_ref(py));
        Self::new_(py, value)
    }

    fn let_patterns(
        slf: Bound<'_, Self>,
        _patterns: Bound<'_, PyTuple>,
    ) -> PyResult<Py<PyPattern>> {
        Ok(slf.cast_into()?.into())
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> Py<PyTuple> {
        PyTuple::empty(py).into()
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("value", PyString))
    }
}

impl Wrappable<Pattern> for KoreString {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Str(s) = it {
            Ok(Self {
                value: s.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for KoreString {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let str = self.value.extract(py)?;
        let pat = Pattern::Str(str);
        convert_with_wrapped(py, pat, self)
    }
}

// --- App ---

#[pyclass(extends = PyPattern, name = "App", get_all)]
pub struct App {
    symbol: Py<PyString>,
    sorts: Py<PyTuple>,
    args: Py<PyTuple>,
}

#[pymethods]
impl App {
    #[new]
    #[pyo3(signature = (symbol, sorts=None, args=None))]
    fn new_(
        py: Python<'_>,
        symbol: Py<PyString>,
        sorts: Option<Py<PySequence>>,
        args: Option<Py<PySequence>>,
    ) -> PyResult<Py<PyPattern>> {
        Self {
            symbol,
            sorts: maybe_seq_to_tuple(py, sorts)?,
            args: maybe_seq_to_tuple(py, args)?,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (symbol=None, sorts=None, args=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        symbol: Option<Py<PyString>>,
        sorts: Option<Py<PySequence>>,
        args: Option<Py<PySequence>>,
    ) -> PyResult<Py<PyPattern>> {
        let symbol = symbol.unwrap_or_else(|| self.symbol.clone_ref(py));
        let sorts = sorts.or_else(|| Some(py_tuple_to_sequence(py, &self.sorts)));
        let args = args.or_else(|| Some(py_tuple_to_sequence(py, &self.args)));
        Self::new_(py, symbol, sorts, args)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        slf.borrow().r#let(slf.py(), None, None, Some(patterns))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> Py<PyTuple> {
        self.args.clone_ref(py)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("symbol", PyString),
            ("sorts", PyTuple),
            ("args", PyTuple)
        )
    }
}

impl Wrappable<Pattern> for App {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::App(app) = it {
            Ok(Self {
                symbol: app.symbol.into_pyobject(py)?.into(),
                sorts: vec_to_pytuple(py, app.sorts)?,
                args: vec_to_pytuple(py, app.args)?,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for App {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let app = crate::kore::App {
            symbol: self.symbol.extract(py)?,
            sorts: self.sorts.extract(py)?,
            args: self.args.extract(py)?,
        };
        let pat = Pattern::App(app);
        convert_with_wrapped(py, pat, self)
    }
}

// --- LeftAssoc ---

#[pyclass(extends = PyPattern, get_all)]
pub struct LeftAssoc {
    symbol: Py<PyString>,
    sorts: Py<PyTuple>,
    args: Py<PyTuple>,
}

#[pymethods]
impl LeftAssoc {
    #[new]
    #[pyo3(signature = (symbol, sorts=None, args=None))]
    fn new_(
        py: Python<'_>,
        symbol: Py<PyString>,
        sorts: Option<Py<PySequence>>,
        args: Option<Py<PySequence>>,
    ) -> PyResult<Py<PyPattern>> {
        Self {
            symbol,
            sorts: maybe_seq_to_tuple(py, sorts)?,
            args: maybe_seq_to_tuple(py, args)?,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (symbol=None, sorts=None, args=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        symbol: Option<Py<PyString>>,
        sorts: Option<Py<PySequence>>,
        args: Option<Py<PySequence>>,
    ) -> PyResult<Py<PyPattern>> {
        let symbol = symbol.unwrap_or_else(|| self.symbol.clone_ref(py));
        let sorts = sorts.or_else(|| Some(py_tuple_to_sequence(py, &self.sorts)));
        let args = args.or_else(|| Some(py_tuple_to_sequence(py, &self.args)));
        Self::new_(py, symbol, sorts, args)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let kwargs = &[("args", patterns)].into_py_dict(slf.py())?;
        let res = slf.call_method("let", (), Some(kwargs))?;
        Ok(res.cast_into::<PyPattern>()?.unbind())
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> Py<PyTuple> {
        self.args.clone_ref(py)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("symbol", PyString),
            ("sorts", PyTuple),
            ("args", PyTuple)
        )
    }
}

impl Wrappable<Pattern> for LeftAssoc {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::LeftAssoc(app) = it {
            Ok(Self {
                symbol: app.symbol.into_pyobject(py)?.into(),
                sorts: vec_to_pytuple(py, app.sorts)?,
                args: vec_to_pytuple(py, app.args)?,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for LeftAssoc {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let app = crate::kore::App {
            symbol: self.symbol.extract(py)?,
            sorts: self.sorts.extract(py)?,
            args: self.args.extract(py)?,
        };
        let pat = Pattern::LeftAssoc(app);
        convert_with_wrapped(py, pat, self)
    }
}

// --- RightAssoc ---

#[pyclass(extends = PyPattern, get_all)]
pub struct RightAssoc {
    symbol: Py<PyString>,
    sorts: Py<PyTuple>,
    args: Py<PyTuple>,
}

#[pymethods]
impl RightAssoc {
    #[new]
    #[pyo3(signature = (symbol, sorts=None, args=None))]
    fn new_(
        py: Python<'_>,
        symbol: Py<PyString>,
        sorts: Option<Py<PySequence>>,
        args: Option<Py<PySequence>>,
    ) -> PyResult<Py<PyPattern>> {
        Self {
            symbol,
            sorts: maybe_seq_to_tuple(py, sorts)?,
            args: maybe_seq_to_tuple(py, args)?,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (symbol=None, sorts=None, args=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        symbol: Option<Py<PyString>>,
        sorts: Option<Py<PySequence>>,
        args: Option<Py<PySequence>>,
    ) -> PyResult<Py<PyPattern>> {
        let symbol = symbol.unwrap_or_else(|| self.symbol.clone_ref(py));
        let sorts = sorts.or_else(|| Some(py_tuple_to_sequence(py, &self.sorts)));
        let args = args.or_else(|| Some(py_tuple_to_sequence(py, &self.args)));
        Self::new_(py, symbol, sorts, args)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let kwargs = &[("args", patterns)].into_py_dict(slf.py())?;
        let res = slf.call_method("let", (), Some(kwargs))?;
        Ok(res.cast_into::<PyPattern>()?.unbind())
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> Py<PyTuple> {
        self.args.clone_ref(py)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("symbol", PyString),
            ("sorts", PyTuple),
            ("args", PyTuple)
        )
    }
}

impl Wrappable<Pattern> for RightAssoc {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::RightAssoc(app) = it {
            Ok(Self {
                symbol: app.symbol.into_pyobject(py)?.into(),
                sorts: vec_to_pytuple(py, app.sorts)?,
                args: vec_to_pytuple(py, app.args)?,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for RightAssoc {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let app = crate::kore::App {
            symbol: self.symbol.extract(py)?,
            sorts: self.sorts.extract(py)?,
            args: self.args.extract(py)?,
        };
        let pat = Pattern::RightAssoc(app);
        convert_with_wrapped(py, pat, self)
    }
}

// --- Top ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Top {
    sort: Py<PySort>,
}

#[pymethods]
impl Top {
    #[new]
    fn new_(py: Python<'_>, sort: Py<PySort>) -> PyResult<Py<PyPattern>> {
        Self { sort }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None))]
    fn r#let(&self, py: Python<'_>, sort: Option<Py<PySort>>) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        Self::new_(py, sort)
    }

    fn let_patterns(slf: Bound<'_, Self>, _patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        Ok(slf.cast_into()?.into())
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> Py<PyTuple> {
        PyTuple::empty(py).into()
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("sort", PySort))
    }
}

impl Wrappable<Pattern> for Top {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Top(sort) = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Top {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Top(self.sort.extract(py)?);
        convert_with_wrapped(py, pat, self)
    }
}

// --- Bottom ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Bottom {
    sort: Py<PySort>,
}

#[pymethods]
impl Bottom {
    #[new]
    fn new_(py: Python<'_>, sort: Py<PySort>) -> PyResult<Py<PyPattern>> {
        Self { sort }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None))]
    fn r#let(&self, py: Python<'_>, sort: Option<Py<PySort>>) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        Self::new_(py, sort)
    }

    fn let_patterns(slf: Bound<'_, Self>, _patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        Ok(slf.cast_into()?.into())
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> Py<PyTuple> {
        PyTuple::empty(py).into()
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("sort", PySort))
    }
}

impl Wrappable<Pattern> for Bottom {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Bottom(sort) = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Bottom {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Bottom(self.sort.extract(py)?);
        convert_with_wrapped(py, pat, self)
    }
}

// --- DV ---

#[pyclass(extends = PyPattern, get_all)]
pub struct DV {
    sort: Py<PySort>,
    value: Py<PyString>,
}

#[pymethods]
impl DV {
    #[new]
    fn new_(py: Python<'_>, sort: Py<PySort>, value: Py<PyString>) -> PyResult<Py<PyPattern>> {
        Self { sort, value }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None, value=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        sort: Option<Py<PySort>>,
        value: Option<Py<PyString>>,
    ) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let value = value.unwrap_or_else(|| self.value.clone_ref(py));
        Self::new_(py, sort, value)
    }

    fn let_patterns(slf: Bound<'_, Self>, _patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        Ok(slf.cast_into()?.into())
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> Py<PyTuple> {
        PyTuple::empty(py).into()
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("sort", PySort), ("value", PyPattern))
    }
}

impl Wrappable<Pattern> for DV {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Dv { sort, value } = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
                value: value.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for DV {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Dv {
            sort: self.sort.extract(py)?,
            value: self.value.extract(py)?,
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Not ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Not {
    sort: Py<PySort>,
    pattern: Py<PyPattern>,
}

#[pymethods]
impl Not {
    #[new]
    fn new_(py: Python<'_>, sort: Py<PySort>, pattern: Py<PyPattern>) -> PyResult<Py<PyPattern>> {
        Self { sort, pattern }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None, pattern=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        sort: Option<Py<PySort>>,
        pattern: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let pattern = pattern.unwrap_or_else(|| self.pattern.clone_ref(py));
        Self::new_(py, sort, pattern)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [pattern] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, Some(pattern))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.pattern.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("sort", PySort), ("pattern", PyPattern))
    }
}

impl Wrappable<Pattern> for Not {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Not { sort, op } = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
                pattern: op.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Not {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Not {
            sort: self.sort.extract(py)?,
            op: self.pattern.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Next ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Next {
    sort: Py<PySort>,
    pattern: Py<PyPattern>,
}

#[pymethods]
impl Next {
    #[new]
    fn new_(py: Python<'_>, sort: Py<PySort>, pattern: Py<PyPattern>) -> PyResult<Py<PyPattern>> {
        Self { sort, pattern }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None, pattern=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        sort: Option<Py<PySort>>,
        pattern: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let pattern = pattern.unwrap_or_else(|| self.pattern.clone_ref(py));
        Self::new_(py, sort, pattern)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [pattern] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, Some(pattern))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.pattern.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("sort", PySort), ("pattern", PyPattern))
    }
}

impl Wrappable<Pattern> for Next {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Next { sort, op } = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
                pattern: op.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Next {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Next {
            sort: self.sort.extract(py)?,
            op: self.pattern.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Implies ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Implies {
    sort: Py<PySort>,
    left: Py<PyPattern>,
    right: Py<PyPattern>,
}

#[pymethods]
impl Implies {
    #[new]
    fn new_(
        py: Python<'_>,
        sort: Py<PySort>,
        left: Py<PyPattern>,
        right: Py<PyPattern>,
    ) -> PyResult<Py<PyPattern>> {
        Self { sort, left, right }
            .into_pyobject(py)
            .map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None, left=None, right=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        sort: Option<Py<PySort>>,
        left: Option<Py<PyPattern>>,
        right: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let left = left.unwrap_or_else(|| self.left.clone_ref(py));
        let right = right.unwrap_or_else(|| self.right.clone_ref(py));
        Self::new_(py, sort, left, right)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [left, right] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, Some(left), Some(right))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.left.clone_ref(py), self.right.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("sort", PySort),
            ("left", PyPattern),
            ("right", PyPattern)
        )
    }
}

impl Wrappable<Pattern> for Implies {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Implies { sort, left, right } = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
                left: left.into_pyobject(py)?.into(),
                right: right.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Implies {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Implies {
            sort: self.sort.extract(py)?,
            left: self.left.extract::<Pattern>(py)?.into(),
            right: self.right.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Iff ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Iff {
    sort: Py<PySort>,
    left: Py<PyPattern>,
    right: Py<PyPattern>,
}

#[pymethods]
impl Iff {
    #[new]
    fn new_(
        py: Python<'_>,
        sort: Py<PySort>,
        left: Py<PyPattern>,
        right: Py<PyPattern>,
    ) -> PyResult<Py<PyPattern>> {
        Self { sort, left, right }
            .into_pyobject(py)
            .map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None, left=None, right=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        sort: Option<Py<PySort>>,
        left: Option<Py<PyPattern>>,
        right: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let left = left.unwrap_or_else(|| self.left.clone_ref(py));
        let right = right.unwrap_or_else(|| self.right.clone_ref(py));
        Self::new_(py, sort, left, right)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [left, right] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, Some(left), Some(right))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.left.clone_ref(py), self.right.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("sort", PySort),
            ("left", PyPattern),
            ("right", PyPattern)
        )
    }
}

impl Wrappable<Pattern> for Iff {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Iff { sort, left, right } = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
                left: left.into_pyobject(py)?.into(),
                right: right.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Iff {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Iff {
            sort: self.sort.extract(py)?,
            left: self.left.extract::<Pattern>(py)?.into(),
            right: self.right.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Rewrites ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Rewrites {
    sort: Py<PySort>,
    left: Py<PyPattern>,
    right: Py<PyPattern>,
}

#[pymethods]
impl Rewrites {
    #[new]
    fn new_(
        py: Python<'_>,
        sort: Py<PySort>,
        left: Py<PyPattern>,
        right: Py<PyPattern>,
    ) -> PyResult<Py<PyPattern>> {
        Self { sort, left, right }
            .into_pyobject(py)
            .map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None, left=None, right=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        sort: Option<Py<PySort>>,
        left: Option<Py<PyPattern>>,
        right: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let left = left.unwrap_or_else(|| self.left.clone_ref(py));
        let right = right.unwrap_or_else(|| self.right.clone_ref(py));
        Self::new_(py, sort, left, right)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [left, right] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, Some(left), Some(right))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.left.clone_ref(py), self.right.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("sort", PySort),
            ("left", PyPattern),
            ("right", PyPattern)
        )
    }
}

impl Wrappable<Pattern> for Rewrites {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Rewrites { sort, left, right } = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
                left: left.into_pyobject(py)?.into(),
                right: right.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Rewrites {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Rewrites {
            sort: self.sort.extract(py)?,
            left: self.left.extract::<Pattern>(py)?.into(),
            right: self.right.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- And ---

#[pyclass(extends = PyPattern, get_all)]
pub struct And {
    sort: Py<PySort>,
    ops: Py<PyTuple>,
}

#[pymethods]
impl And {
    #[new]
    fn new_(py: Python<'_>, sort: Py<PySort>, ops: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        Self {
            sort,
            ops: seq_to_tuple(py, ops)?,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None, ops=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        sort: Option<Py<PySort>>,
        ops: Option<Py<PySequence>>,
    ) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let ops = ops.unwrap_or_else(|| py_tuple_to_sequence(py, &self.ops));
        Self::new_(py, sort, ops)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        slf.borrow().r#let(slf.py(), None, Some(patterns))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(self.ops.clone_ref(py))
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("sort", PySort), ("ops", PyTuple))
    }
}

impl Wrappable<Pattern> for And {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::And { sort, ops } = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
                ops: vec_to_pytuple(py, ops)?,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for And {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::And {
            sort: self.sort.extract(py)?,
            ops: self.ops.extract(py)?,
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Or ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Or {
    sort: Py<PySort>,
    ops: Py<PyTuple>,
}

#[pymethods]
impl Or {
    #[new]
    fn new_(py: Python<'_>, sort: Py<PySort>, ops: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        Self {
            sort,
            ops: seq_to_tuple(py, ops)?,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None, ops=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        sort: Option<Py<PySort>>,
        ops: Option<Py<PySequence>>,
    ) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let ops = ops.unwrap_or_else(|| py_tuple_to_sequence(py, &self.ops));
        Self::new_(py, sort, ops)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        slf.borrow().r#let(slf.py(), None, Some(patterns))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(self.ops.clone_ref(py))
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("sort", PySort), ("ops", PyTuple))
    }
}

impl Wrappable<Pattern> for Or {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Or { sort, ops } = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
                ops: vec_to_pytuple(py, ops)?,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Or {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Or {
            sort: self.sort.extract(py)?,
            ops: self.ops.extract(py)?,
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Exists ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Exists {
    sort: Py<PySort>,
    var: Py<EVar>,
    pattern: Py<PyPattern>,
}

#[pymethods]
impl Exists {
    #[new]
    fn new_(
        py: Python<'_>,
        sort: Py<PySort>,
        var: Py<EVar>,
        pattern: Py<PyPattern>,
    ) -> PyResult<Py<PyPattern>> {
        Self { sort, var, pattern }
            .into_pyobject(py)
            .map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None, var=None, pattern=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        sort: Option<Py<PySort>>,
        var: Option<Py<EVar>>,
        pattern: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let var = var.unwrap_or_else(|| self.var.clone_ref(py));
        let pattern = pattern.unwrap_or_else(|| self.pattern.clone_ref(py));
        Self::new_(py, sort, var, pattern)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [pattern] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, None, Some(pattern))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.pattern.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("sort", PySort),
            ("var", PyPattern),
            ("pattern", PyPattern)
        )
    }
}

impl Wrappable<Pattern> for Exists {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Exists { sort, var, op } = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
                var: var.into_pyobject(py)?.into(),
                pattern: op.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Exists {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Exists {
            sort: self.sort.extract(py)?,
            var: self.var.extract(py)?,
            op: self.pattern.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Forall ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Forall {
    sort: Py<PySort>,
    var: Py<EVar>,
    pattern: Py<PyPattern>,
}

#[pymethods]
impl Forall {
    #[new]
    fn new_(
        py: Python<'_>,
        sort: Py<PySort>,
        var: Py<EVar>,
        pattern: Py<PyPattern>,
    ) -> PyResult<Py<PyPattern>> {
        Self { sort, var, pattern }
            .into_pyobject(py)
            .map(Bound::unbind)
    }

    #[pyo3(signature = (sort=None, var=None, pattern=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        sort: Option<Py<PySort>>,
        var: Option<Py<EVar>>,
        pattern: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let var = var.unwrap_or_else(|| self.var.clone_ref(py));
        let pattern = pattern.unwrap_or_else(|| self.pattern.clone_ref(py));
        Self::new_(py, sort, var, pattern)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [pattern] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, None, Some(pattern))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.pattern.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("sort", PySort),
            ("var", PyPattern),
            ("pattern", PyPattern)
        )
    }
}

impl Wrappable<Pattern> for Forall {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Forall { sort, var, op } = it {
            Ok(Self {
                sort: sort.into_pyobject(py)?.into(),
                var: var.into_pyobject(py)?.into(),
                pattern: op.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Forall {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Forall {
            sort: self.sort.extract(py)?,
            var: self.var.extract(py)?,
            op: self.pattern.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Mu ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Mu {
    var: Py<PySVar>,
    pattern: Py<PyPattern>,
}

#[pymethods]
impl Mu {
    #[new]
    fn new_(py: Python<'_>, var: Py<PySVar>, pattern: Py<PyPattern>) -> PyResult<Py<PyPattern>> {
        Self { var, pattern }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (var=None, pattern=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        var: Option<Py<PySVar>>,
        pattern: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let var = var.unwrap_or_else(|| self.var.clone_ref(py));
        let pattern = pattern.unwrap_or_else(|| self.pattern.clone_ref(py));
        Self::new_(py, var, pattern)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [pattern] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, Some(pattern))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.pattern.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("var", PyPattern), ("pattern", PyPattern))
    }
}

impl Wrappable<Pattern> for Mu {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Mu { var, op } = it {
            Ok(Self {
                var: var.into_pyobject(py)?.into(),
                pattern: op.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Mu {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Mu {
            var: self.var.extract(py)?,
            op: self.pattern.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Nu ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Nu {
    var: Py<PySVar>,
    pattern: Py<PyPattern>,
}

#[pymethods]
impl Nu {
    #[new]
    fn new_(py: Python<'_>, var: Py<PySVar>, pattern: Py<PyPattern>) -> PyResult<Py<PyPattern>> {
        Self { var, pattern }.into_pyobject(py).map(Bound::unbind)
    }

    #[pyo3(signature = (var=None, pattern=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        var: Option<Py<PySVar>>,
        pattern: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let var = var.unwrap_or_else(|| self.var.clone_ref(py));
        let pattern = pattern.unwrap_or_else(|| self.pattern.clone_ref(py));
        Self::new_(py, var, pattern)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [pattern] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, Some(pattern))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.pattern.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("var", PyPattern), ("pattern", PyPattern))
    }
}

impl Wrappable<Pattern> for Nu {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Nu { var, op } = it {
            Ok(Self {
                var: var.into_pyobject(py)?.into(),
                pattern: op.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Nu {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Nu {
            var: self.var.extract(py)?,
            op: self.pattern.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Ceil ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Ceil {
    op_sort: Py<PySort>,
    sort: Py<PySort>,
    pattern: Py<PyPattern>,
}

#[pymethods]
impl Ceil {
    #[new]
    fn new_(
        py: Python<'_>,
        op_sort: Py<PySort>,
        sort: Py<PySort>,
        pattern: Py<PyPattern>,
    ) -> PyResult<Py<PyPattern>> {
        Self {
            op_sort,
            sort,
            pattern,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (op_sort=None, sort=None, pattern=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        op_sort: Option<Py<PySort>>,
        sort: Option<Py<PySort>>,
        pattern: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let op_sort = op_sort.unwrap_or_else(|| self.op_sort.clone_ref(py));
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let pattern = pattern.unwrap_or_else(|| self.pattern.clone_ref(py));
        Self::new_(py, op_sort, sort, pattern)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [pattern] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, None, Some(pattern))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.pattern.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("op_sort", PySort),
            ("sort", PySort),
            ("pattern", PyPattern)
        )
    }
}

impl Wrappable<Pattern> for Ceil {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Ceil { op_sort, sort, op } = it {
            Ok(Self {
                op_sort: op_sort.into_pyobject(py)?.into(),
                sort: sort.into_pyobject(py)?.into(),
                pattern: op.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Ceil {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Ceil {
            op_sort: self.op_sort.extract(py)?,
            sort: self.sort.extract(py)?,
            op: self.pattern.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Floor ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Floor {
    op_sort: Py<PySort>,
    sort: Py<PySort>,
    pattern: Py<PyPattern>,
}

#[pymethods]
impl Floor {
    #[new]
    fn new_(
        py: Python<'_>,
        op_sort: Py<PySort>,
        sort: Py<PySort>,
        pattern: Py<PyPattern>,
    ) -> PyResult<Py<PyPattern>> {
        Self {
            op_sort,
            sort,
            pattern,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (op_sort=None, sort=None, pattern=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        op_sort: Option<Py<PySort>>,
        sort: Option<Py<PySort>>,
        pattern: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let op_sort = op_sort.unwrap_or_else(|| self.op_sort.clone_ref(py));
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let pattern = pattern.unwrap_or_else(|| self.pattern.clone_ref(py));
        Self::new_(py, op_sort, sort, pattern)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [pattern] = patterns.extract(slf.py())?;
        slf.borrow().r#let(slf.py(), None, None, Some(pattern))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.pattern.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("op_sort", PySort),
            ("sort", PySort),
            ("pattern", PyPattern)
        )
    }
}

impl Wrappable<Pattern> for Floor {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Floor { op_sort, sort, op } = it {
            Ok(Self {
                op_sort: op_sort.into_pyobject(py)?.into(),
                sort: sort.into_pyobject(py)?.into(),
                pattern: op.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Floor {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Floor {
            op_sort: self.op_sort.extract(py)?,
            sort: self.sort.extract(py)?,
            op: self.pattern.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- Equals ---

#[pyclass(extends = PyPattern, get_all)]
pub struct Equals {
    op_sort: Py<PySort>,
    sort: Py<PySort>,
    left: Py<PyPattern>,
    right: Py<PyPattern>,
}

#[pymethods]
impl Equals {
    #[new]
    fn new_(
        py: Python<'_>,
        op_sort: Py<PySort>,
        sort: Py<PySort>,
        left: Py<PyPattern>,
        right: Py<PyPattern>,
    ) -> PyResult<Py<PyPattern>> {
        Self {
            op_sort,
            sort,
            left,
            right,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (op_sort=None, sort=None, left=None, right=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        op_sort: Option<Py<PySort>>,
        sort: Option<Py<PySort>>,
        left: Option<Py<PyPattern>>,
        right: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let op_sort = op_sort.unwrap_or_else(|| self.op_sort.clone_ref(py));
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let left = left.unwrap_or_else(|| self.left.clone_ref(py));
        let right = right.unwrap_or_else(|| self.right.clone_ref(py));
        Self::new_(py, op_sort, sort, left, right)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [left, right] = patterns.extract(slf.py())?;
        slf.borrow()
            .r#let(slf.py(), None, None, Some(left), Some(right))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.left.clone_ref(py), self.right.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("op_sort", PySort),
            ("sort", PySort),
            ("left", PyPattern),
            ("right", PyPattern)
        )
    }
}

impl Wrappable<Pattern> for Equals {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::Equals {
            op_sort,
            sort,
            left,
            right,
        } = it
        {
            Ok(Self {
                op_sort: op_sort.into_pyobject(py)?.into(),
                sort: sort.into_pyobject(py)?.into(),
                left: left.into_pyobject(py)?.into(),
                right: right.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Equals {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::Equals {
            op_sort: self.op_sort.extract(py)?,
            sort: self.sort.extract(py)?,
            left: self.left.extract::<Pattern>(py)?.into(),
            right: self.right.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// --- In ---

#[pyclass(extends = PyPattern, get_all)]
pub struct In {
    op_sort: Py<PySort>,
    sort: Py<PySort>,
    left: Py<PyPattern>,
    right: Py<PyPattern>,
}

#[pymethods]
impl In {
    #[new]
    fn new_(
        py: Python<'_>,
        op_sort: Py<PySort>,
        sort: Py<PySort>,
        left: Py<PyPattern>,
        right: Py<PyPattern>,
    ) -> PyResult<Py<PyPattern>> {
        Self {
            op_sort,
            sort,
            left,
            right,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[pyo3(signature = (op_sort=None, sort=None, left=None, right=None))]
    fn r#let(
        &self,
        py: Python<'_>,
        op_sort: Option<Py<PySort>>,
        sort: Option<Py<PySort>>,
        left: Option<Py<PyPattern>>,
        right: Option<Py<PyPattern>>,
    ) -> PyResult<Py<PyPattern>> {
        let op_sort = op_sort.unwrap_or_else(|| self.op_sort.clone_ref(py));
        let sort = sort.unwrap_or_else(|| self.sort.clone_ref(py));
        let left = left.unwrap_or_else(|| self.left.clone_ref(py));
        let right = right.unwrap_or_else(|| self.right.clone_ref(py));
        Self::new_(py, op_sort, sort, left, right)
    }

    fn let_patterns(slf: Bound<'_, Self>, patterns: Py<PySequence>) -> PyResult<Py<PyPattern>> {
        let [left, right] = patterns.extract(slf.py())?;
        slf.borrow()
            .r#let(slf.py(), None, None, Some(left), Some(right))
    }

    #[getter]
    fn patterns(&self, py: Python<'_>) -> PyResult<Py<PyTuple>> {
        Ok(PyTuple::new(py, [self.left.clone_ref(py), self.right.clone_ref(py)])?.into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("op_sort", PySort),
            ("sort", PySort),
            ("left", PyPattern),
            ("right", PyPattern)
        )
    }
}

impl Wrappable<Pattern> for In {
    fn wrap_into(py: Python<'_>, it: Pattern) -> PyResult<Self> {
        if let Pattern::In {
            op_sort,
            sort,
            left,
            right,
        } = it
        {
            Ok(Self {
                op_sort: op_sort.into_pyobject(py)?.into(),
                sort: sort.into_pyobject(py)?.into(),
                left: left.into_pyobject(py)?.into(),
                right: right.into_pyobject(py)?.into(),
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for In {
    type Target = PyPattern;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let pat = Pattern::In {
            op_sort: self.op_sort.extract(py)?,
            sort: self.sort.extract(py)?,
            left: self.left.extract::<Pattern>(py)?.into(),
            right: self.right.extract::<Pattern>(py)?.into(),
        };
        convert_with_wrapped(py, pat, self)
    }
}

// ==========================================
// Sentence bindings
// ==========================================

// --- Symbol (standalone pyclass, not a Sentence subclass) ---

#[pyclass(get_all)]
pub struct Symbol {
    name: Py<PyString>,
    vars: Py<PyTuple>,
}

impl<'py> Symbol {
    fn as_rust_fields(&self, py: Python<'py>) -> PyResult<(SymbolId, Vec<Id>)> {
        let id: SymbolId = self.name.extract(py)?;
        let vars: Vec<Id> = self
            .vars
            .bind(py)
            .iter()
            .map(|var| var.extract())
            .collect::<Result<Vec<_>, _>>()?;
        Ok((id, vars))
    }
}

#[pymethods]
impl Symbol {
    #[new]
    #[pyo3(signature = (name, vars=None))]
    fn new_(py: Python<'_>, name: Py<PyString>, vars: Option<Py<PyTuple>>) -> PyResult<Py<Self>> {
        let vars = vars.unwrap_or_else(|| PyTuple::empty(py).into());
        let _name_checked: SymbolId = name.extract(py)?;
        let _vars_checked = vars.bind(py).iter().map(|var| var.cast_into::<SortVar>()).collect::<Result<Vec<_>,_>>()?;
        Ok(Symbol {
            name,
            vars,
        }
        .into_pyobject(py)?
        .into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("name", PyString), ("vars", PyTuple))
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
    fn wrap_into(_py: Python<'_>, rust: Sentence) -> PyResult<Self> {
        Ok(Self {
            wrapped: rust.into(),
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

#[pyclass(extends = PySentence, get_all)]
pub struct Import {
    module_name: Py<PyString>,
    attrs: Vec<crate::kore::App>,
}

impl Wrappable<Sentence> for Import {
    fn wrap_into(py: Python<'_>, it: Sentence) -> PyResult<Self> {
        if let Sentence::Import { module, attrs } = it {
            Ok(Self {
                module_name: module.into_pyobject(py)?.into(),
                attrs,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Import {
    type Target = PySentence;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let module = self.module_name.extract(py)?;
        let sen = Sentence::Import {
            module,
            attrs: self.attrs.to_vec(),
        };
        convert_with_wrapped(py, sen, self)
    }
}

#[pymethods]
impl Import {
    #[new]
    #[pyo3(signature = (module_name, attrs=None))]
    fn new_(
        py: Python<'_>,
        module_name: Py<PyString>,
        attrs: Option<Vec<crate::kore::App>>,
    ) -> PyResult<Py<PySentence>> {
        Self {
            module_name,
            attrs: attrs.unwrap_or_default(),
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(py, ("module_name", PyString), ("attrs", PyTuple))
    }
}

// --- SortDecl ---

#[pyclass(extends = PySentence, get_all)]
pub struct SortDecl {
    name: Py<PyString>,
    vars: Vec<Py<SortVar>>,
    attrs: Vec<crate::kore::App>,
    hooked: bool,
}

impl<'py> IntoPyObject<'py> for SortDecl {
    type Target = PySentence;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let id = self.name.extract(py)?;
        let vars: Vec<Id> = self
            .vars
            .iter()
            .map(|var| var.extract(py))
            .collect::<Result<_, _>>()?;
        let sen = Sentence::Sort {
            id,
            vars,
            attrs: self.attrs.to_vec(),
            hooked: self.hooked,
        };
        convert_with_wrapped(py, sen, self)
    }
}

impl Wrappable<Sentence> for SortDecl {
    fn wrap_into(py: Python<'_>, it: Sentence) -> PyResult<Self> {
        if let Sentence::Sort {
            id,
            vars,
            attrs,
            hooked,
        } = it
        {
            let vars: Vec<Py<SortVar>> = vars
                .into_iter()
                .map(|id| id.into_pysortvar(py))
                .collect::<Result<_, PyErr>>()?;
            Ok(Self {
                name: id.into_pyobject(py)?.into(),
                vars,
                attrs: attrs.to_vec(),
                hooked,
            })
        } else {
            Self::error(it)
        }
    }
}

#[pymethods]
impl SortDecl {
    #[new]
    #[pyo3(signature = (name, vars, attrs=None, *, hooked=false))]
    fn new_(
        py: Python<'_>,
        name: Py<PyString>,
        vars: Vec<Id>,
        attrs: Option<Vec<crate::kore::App>>,
        hooked: bool,
    ) -> PyResult<Py<PySentence>> {
        Self {
            name,
            vars: vars
                .into_iter()
                .map(|id| id.into_pysortvar(py))
                .collect::<Result<_, _>>()?,
            attrs: attrs.unwrap_or_default(),
            hooked,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("name", PyString),
            ("vars", PyTuple),
            ("attrs", PyTuple),
            ("hooked", PyBool)
        )
    }
}

// --- SymbolDecl ---

#[pyclass(extends = PySentence, get_all)]
pub struct SymbolDecl {
    symbol: Py<Symbol>,
    param_sorts: Vec<Sort>,
    sort: Sort,
    attrs: Vec<crate::kore::App>,
    hooked: bool,
}

impl Wrappable<Sentence> for SymbolDecl {
    fn wrap_into(py: Python<'_>, it: Sentence) -> PyResult<Self> {
        if let Sentence::Symbol {
            id,
            vars,
            param_sorts,
            sort,
            attrs,
            hooked,
        } = it
        {
            let vars: Vec<Py<SortVar>> = vars
                .into_iter()
                .map(|id| id.into_pysortvar(py))
                .collect::<Result<_, PyErr>>()?;
            let symbol = Symbol {
                name: id.into_pyobject(py)?.into(),
                vars,
            };
            Ok(Self {
                symbol: Py::new(py, symbol)?,
                param_sorts,
                sort,
                attrs,
                hooked,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for SymbolDecl {
    type Target = PySentence;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let SymbolDecl {
            symbol,
            param_sorts,
            sort,
            attrs,
            hooked,
        } = &self;
        let (id, vars) = symbol.bind(py).borrow().as_rust_fields(py)?;
        let sen = Sentence::Symbol {
            id,
            vars,
            param_sorts: param_sorts.to_vec(),
            sort: sort.clone(),
            attrs: attrs.to_vec(),
            hooked: *hooked,
        };
        convert_with_wrapped(py, sen, self)
    }
}

#[pymethods]
impl SymbolDecl {
    #[new]
    #[pyo3(signature = (symbol, param_sorts, sort, attrs=None, *, hooked=false))]
    fn new_(
        py: Python<'_>,
        symbol: Py<Symbol>,
        param_sorts: Vec<Sort>,
        sort: Sort,
        attrs: Option<Vec<crate::kore::App>>,
        hooked: bool,
    ) -> PyResult<Py<PySentence>> {
        Self {
            symbol,
            param_sorts,
            sort,
            attrs: attrs.unwrap_or_default(),
            hooked,
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("symbol", Symbol),
            ("param_sorts", PyTuple),
            ("sort", PySort),
            ("attrs", PyTuple),
            ("hooked", PyBool)
        )
    }
}

// --- AliasDecl ---

#[pyclass(extends = PySentence, get_all)]
pub struct AliasDecl {
    alias: Py<Symbol>,
    param_sorts: Vec<Sort>,
    sort: Sort,
    left: crate::kore::App,
    right: Pattern,
    attrs: Vec<crate::kore::App>,
}

impl Wrappable<Sentence> for AliasDecl {
    fn wrap_into(py: Python<'_>, it: Sentence) -> PyResult<Self> {
        if let Sentence::Alias {
            id,
            vars,
            param_sorts,
            sort,
            left,
            right,
            attrs,
        } = it
        {
            let vars: Vec<Py<SortVar>> = vars
                .into_iter()
                .map(|id| id.into_pysortvar(py))
                .collect::<Result<_, PyErr>>()?;
            let symbol = Symbol {
                name: id.into_pyobject(py)?.into(),
                vars,
            };
            Ok(Self {
                alias: Py::new(py, symbol)?,
                param_sorts,
                sort,
                left,
                right: *right,
                attrs,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for AliasDecl {
    type Target = PySentence;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let AliasDecl {
            alias,
            param_sorts,
            sort,
            left,
            right,
            attrs,
        } = &self;
        let (id, vars) = alias.bind(py).borrow().as_rust_fields(py)?;
        let sen = Sentence::Alias {
            id,
            vars,
            param_sorts: param_sorts.to_vec(),
            sort: sort.clone(),
            left: left.clone(),
            right: right.clone().into(),
            attrs: attrs.to_vec(),
        };
        convert_with_wrapped(py, sen, self)
    }
}

#[pymethods]
impl AliasDecl {
    #[new]
    #[pyo3(signature = (alias, param_sorts, sort, left, right, attrs=None))]
    fn new_(
        py: Python<'_>,
        alias: Py<Symbol>,
        param_sorts: Vec<Sort>,
        sort: Sort,
        left: crate::kore::App,
        right: Pattern,
        attrs: Option<Vec<crate::kore::App>>,
    ) -> PyResult<Py<PySentence>> {
        Self {
            alias,
            param_sorts,
            sort,
            left,
            right,
            attrs: attrs.unwrap_or_default(),
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("alias", Symbol),
            ("param_sorts", PyTuple),
            ("sort", PySort),
            ("left", PyPattern),
            ("right", PyPattern),
            ("attrs", PyTuple)
        )
    }
}

// --- Axiom ---

#[pyclass(extends = PySentence, get_all)]
pub struct Axiom {
    vars: Vec<Py<SortVar>>,
    pattern: Pattern,
    attrs: Vec<crate::kore::App>,
}

impl Wrappable<Sentence> for Axiom {
    fn wrap_into(py: Python<'_>, it: Sentence) -> PyResult<Self> {
        if let Sentence::Axiom {
            vars,
            pattern,
            attrs,
        } = it
        {
            let vars: Vec<Py<SortVar>> = vars
                .into_iter()
                .map(|id| id.into_pysortvar(py))
                .collect::<Result<_, PyErr>>()?;
            Ok(Self {
                vars,
                pattern: *pattern,
                attrs,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Axiom {
    type Target = PySentence;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let vars: Vec<Id> = self
            .vars
            .iter()
            .map(|var| var.extract(py))
            .collect::<Result<_, _>>()?;
        let sen = Sentence::Axiom {
            vars,
            pattern: self.pattern.clone().into(),
            attrs: self.attrs.to_vec(),
        };
        convert_with_wrapped(py, sen, self)
    }
}

#[pymethods]
impl Axiom {
    #[new]
    #[pyo3(signature = (vars, pattern, attrs=None))]
    fn new_(
        py: Python<'_>,
        vars: Vec<Id>,
        pattern: Pattern,
        attrs: Option<Vec<crate::kore::App>>,
    ) -> PyResult<Py<PySentence>> {
        Self {
            vars: vars
                .into_iter()
                .map(|id| id.into_pysortvar(py))
                .collect::<Result<_, _>>()?,
            pattern,
            attrs: attrs.unwrap_or_default(),
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    fn r#let(
        &self,
        py: Python<'_>,
        vars: Option<Vec<Id>>,
        pattern: Option<Pattern>,
        attrs: Option<Vec<crate::kore::App>>,
    ) -> PyResult<Py<PySentence>> {
        let vars = if let Some(vars) = vars {
            vars
        } else {
            self.vars
                .iter()
                .map(|v| v.extract(py))
                .collect::<Result<_, _>>()?
        };
        let pattern = pattern.unwrap_or_else(|| self.pattern.clone());
        let attrs = attrs.or_else(|| Some(self.attrs.clone()));
        Self::new_(py, vars, pattern, attrs)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("vars", PyTuple),
            ("pattern", PyPattern),
            ("attrs", PyTuple)
        )
    }
}

// --- Claim ---

#[pyclass(extends = PySentence, get_all)]
pub struct Claim {
    vars: Vec<Py<SortVar>>,
    pattern: Pattern,
    attrs: Vec<crate::kore::App>,
}

impl Wrappable<Sentence> for Claim {
    fn wrap_into(py: Python<'_>, it: Sentence) -> PyResult<Self> {
        if let Sentence::Claim {
            vars,
            pattern,
            attrs,
        } = it
        {
            let vars: Vec<Py<SortVar>> = vars
                .into_iter()
                .map(|id| id.into_pysortvar(py))
                .collect::<Result<_, PyErr>>()?;
            Ok(Self {
                vars,
                pattern: *pattern,
                attrs,
            })
        } else {
            Self::error(it)
        }
    }
}

impl<'py> IntoPyObject<'py> for Claim {
    type Target = PySentence;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let vars: Vec<Id> = self
            .vars
            .iter()
            .map(|var| var.extract(py))
            .collect::<Result<_, _>>()?;
        let sen = Sentence::Claim {
            vars,
            pattern: self.pattern.clone().into(),
            attrs: self.attrs.to_vec(),
        };
        convert_with_wrapped(py, sen, self)
    }
}

#[pymethods]
impl Claim {
    #[new]
    #[pyo3(signature = (vars, pattern, attrs=None))]
    fn new_(
        py: Python<'_>,
        vars: Vec<Id>,
        pattern: Pattern,
        attrs: Option<Vec<crate::kore::App>>,
    ) -> PyResult<Py<PySentence>> {
        Self {
            vars: vars
                .into_iter()
                .map(|id| id.into_pysortvar(py))
                .collect::<Result<_, _>>()?,
            pattern,
            attrs: attrs.unwrap_or_default(),
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    fn r#let(
        &self,
        py: Python<'_>,
        vars: Option<Vec<Id>>,
        pattern: Option<Pattern>,
        attrs: Option<Vec<crate::kore::App>>,
    ) -> PyResult<Py<PySentence>> {
        let vars = if let Some(vars) = vars {
            vars
        } else {
            self.vars
                .iter()
                .map(|v| v.extract(py))
                .collect::<Result<_, _>>()?
        };
        let pattern = pattern.unwrap_or_else(|| self.pattern.clone());
        let attrs = attrs.or_else(|| Some(self.attrs.clone()));
        Self::new_(py, vars, pattern, attrs)
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        annotations!(
            py,
            ("vars", PyTuple),
            ("pattern", PyPattern),
            ("attrs", PyTuple)
        )
    }
}

// ==========================================
// Module bindings
// ==========================================

#[pyclass(name = "Module", get_all)]
pub struct KoreModule {
    name: Py<PyString>,
    sentences: Vec<Sentence>,
    attrs: Vec<crate::kore::App>,
}

impl<'py> IntoPyObject<'py> for crate::kore::Module {
    type Target = KoreModule;
    type Output = Bound<'py, KoreModule>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Bound::new(
            py,
            KoreModule {
                name: self.id.into_pyobject(py)?.into(),
                sentences: self.sentences,
                attrs: self.attrs,
            },
        )
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for crate::kore::Module {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let m = obj.cast::<KoreModule>()?;
        let borrow = m.borrow();
        let id: Id = borrow.name.extract(obj.py())?;
        let attrs = borrow.attrs.clone();
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
        name: Py<PyString>,
        sentences: Option<Vec<Sentence>>,
        attrs: Option<Vec<crate::kore::App>>,
    ) -> PyResult<Py<KoreModule>> {
        crate::kore::Module {
            id: name.extract(py)?,
            sentences: sentences.unwrap_or_default(),
            attrs: attrs.unwrap_or_default(),
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    fn r#let(
        &self,
        py: Python<'_>,
        name: Option<Py<PyString>>,
        sentences: Option<Vec<Sentence>>,
        attrs: Option<Vec<crate::kore::App>>,
    ) -> PyResult<Py<KoreModule>> {
        let name = name.unwrap_or_else(|| self.name.clone_ref(py));
        let sentences = sentences.or_else(|| Some(self.sentences.clone()));
        let attrs = attrs.or_else(|| Some(self.attrs.clone()));
        Self::new_(py, name, sentences, attrs)
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
        annotations!(
            py,
            ("name", PyString),
            ("sentences", PyTuple),
            ("attrs", PyTuple)
        )
    }
}

// ==========================================
// Definition bindings
// ==========================================

#[pyclass(name = "Definition", get_all)]
pub struct KoreDefinition {
    modules: Vec<crate::kore::Module>,
    attrs: Vec<crate::kore::App>,
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
                attrs: self.attrs,
            },
        )
    }
}

impl<'a, 'py> FromPyObject<'a, 'py> for crate::kore::Definition {
    type Error = PyErr;

    fn extract(obj: Borrowed<'a, 'py, PyAny>) -> Result<Self, Self::Error> {
        let d = obj.cast::<KoreDefinition>()?;
        let borrow = d.borrow();
        let attrs = borrow.attrs.clone();
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
        attrs: Option<Vec<crate::kore::App>>,
    ) -> PyResult<Py<KoreDefinition>> {
        crate::kore::Definition {
            modules: modules.unwrap_or_default(),
            attrs: attrs.unwrap_or_default(),
        }
        .into_pyobject(py)
        .map(Bound::unbind)
    }

    fn r#let(
        &self,
        py: Python<'_>,
        modules: Option<Vec<crate::kore::Module>>,
        attrs: Option<Vec<crate::kore::App>>,
    ) -> PyResult<Py<KoreDefinition>> {
        let modules = modules.or_else(|| Some(self.modules.clone()));
        let attrs = attrs.or_else(|| Some(self.attrs.clone()));
        Self::new_(py, modules, attrs)
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
        annotations!(py, ("modules", PyTuple), ("attrs", PyTuple))
    }
}
