#![allow(unused)]

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::kore::{Id, Sort};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{PyDict, PyList, PyString, PyTuple},
};

#[pymethods]
impl Id {
    #[new]
    fn py_new(s: String) -> PyResult<Self> {
        Self::new(s).map_err(PyValueError::new_err)
    }

    #[getter(value)]
    fn py_value(&self) -> &str {
        self.value()
    }
}

impl TryFrom<&Bound<'_, PyAny>> for Id {
    type Error = PyErr;

    fn try_from(value: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(id) = value.extract::<Id>() {
            return Ok(id);
        };
        let id_str = value.cast::<PyString>()?;
        let id = Id::new(id_str.to_string()).map_err(PyValueError::new_err)?;
        Ok(id)
    }
}

/// [`PySortArena`]
///
/// An arena containing pointers to allocated [`Sort`]s, as well as
/// some metadata about those objects and the arena itself.
#[pyclass]
#[derive(Default)]
pub struct PySortArena {
    inners: Vec<Arc<Sort>>,
    // A mapping of *const Sort (as usize) to indices of already existing members
    index_of: HashMap<usize, usize>,
    // The python objects corresponding to the rust types, if they exist
    cached_pys: Vec<Option<Py<PySort>>>,
}

impl PySortArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, sort: Arc<Sort>) -> usize {
        // Check if this sort has already been added to the arena
        if let Some(idx) = self.index_of.get(&(Arc::<Sort>::as_ptr(&sort) as usize)) {
            return *idx;
        }

        // Otherwise, allocate space for the new sort
        let idx = self.inners.len();
        self.inners.push(sort.clone());
        self.index_of
            .insert(Arc::<Sort>::as_ptr(&sort) as usize, idx);
        self.cached_pys.push(None);

        if let Sort::App { args, .. } = &*sort {
            let children = args
                .iter()
                // Add this sort's children to the arena as well
                .map(|arg| self.add(arg.clone()))
                .collect::<Box<_>>();
        }

        idx
    }

    pub fn get(&self, idx: usize) -> Arc<Sort> {
        self.inners.get(idx).cloned().expect("Invalid node index")
    }
}

/// [`PySort`]
///
/// The base class for the python Sort types.
///
/// This is just a pointer to a [`PySortArena`] and the
/// index in the arena that represents this object.
#[pyclass(subclass, name = "Sort")]
pub struct PySort {
    inner: Py<PySortArena>,
    idx: usize,
}

impl PySort {
    fn get_inner(&self, py: Python<'_>) -> Arc<Sort> {
        let arena = self.inner.bind(py).borrow();
        arena.get(self.idx)
    }

    fn get_arena<'py>(py: Python<'py>) -> PyResult<Bound<'py, PySortArena>> {
        let cls = py.get_type::<PySort>();
        let arena_attr = cls.getattr("__sort_arena__")?;
        Ok(arena_attr.cast_into()?)
    }

    /// Create a [`PySort`] from a [`Sort`]
    ///
    /// This relies on the [`PySort::__sort_arena__`] attribute to hold
    /// all allocations of both the rust types and their python representations.
    ///
    /// This also updates the arena's cache with the created object
    fn create(py: Python<'_>, sort: Arc<Sort>) -> PyResult<Bound<'_, Self>> {
        let arena_bound = Self::get_arena(py)?;

        let mut arena = arena_bound.borrow_mut();
        let idx = arena.add(sort.clone());

        // Check the arena's cache
        if let Some(res) = arena.cached_pys.get(idx).expect("Invalid node index") {
            return Ok(res.bind(py).clone());
        }

        // Nested calls to `create` are coming up, so we drop
        // the mutable borrow of the arena here to ensure interior mutability
        drop(arena);

        let node = Self {
            inner: arena_bound.clone().into(),
            idx,
        };

        let res: Bound<'_, PySort> = match &*sort {
            Sort::Var(id) => {
                let var = SortVar {
                    name: PyString::new(py, id.value()).unbind(),
                };
                Bound::new(py, (var, node))?.into_super()
            }
            Sort::App { id, args } => {
                let children = args
                    .iter()
                    .map(|sort| Self::create(py, sort.clone()))
                    .collect::<Result<Vec<_>, _>>()?;
                let args = PyTuple::new(py, children)?;
                let app = SortApp {
                    name: PyString::new(py, id.value()).unbind(),
                    args: args.into(),
                };
                Bound::new(py, (app, node))?.into_super()
            }
        };

        // Update the arena's cache
        let mut arena_mut = arena_bound.borrow_mut();
        assert!(arena_mut
            .cached_pys
            .get(idx)
            .expect("Invalid node index")
            .is_none());
        arena_mut.cached_pys[idx] = Some(res.clone().unbind());

        Ok(res)
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

        Self::create(py, sort.into())
    }

    /// [`PySort::__sort_arena__`]
    ///
    /// A class variable that holds onto a single persistent [`PySortArena`]
    /// which holds all allocations of [`Sort`] and their corresponding [`PySort`]
    /// during the lifetime of a python process
    #[classattr]
    fn __sort_arena__(py: Python<'_>) -> PyResult<Bound<'_, PySortArena>> {
        Bound::new(py, PySortArena::new())
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
    fn new_(py: Python<'_>, id: &Bound<'_, PyAny>) -> PyResult<Py<Self>> {
        let id = Id::try_from(id)?;
        let sort = Sort::Var(id);

        Ok(PySort::create(py, sort.into())?
            .cast_into::<Self>()?
            .into())
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
        id: &Bound<'_, PyAny>,
        args: &Bound<'_, PyTuple>,
    ) -> PyResult<Py<PyAny>> {
        let id = Id::try_from(id)?;

        let args: Vec<Arc<Sort>> = args
            .iter()
            .map(|obj| {
                let node = obj.cast_into::<PySort>();
                node.map(|arg| arg.borrow().get_inner(py))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let sort = Sort::App {
            id,
            args: args.into(),
        };

        Ok(PySort::create(py, sort.into())?
            .cast_into::<Self>()?
            .into())
    }

    #[classattr]
    fn __annotations__(py: Python<'_>) -> PyResult<Bound<'_, PyDict>> {
        let app_annotations = PyDict::new(py);
        app_annotations.set_item("name", py.get_type::<PyString>())?;
        app_annotations.set_item("args", py.get_type::<PyTuple>())?;
        Ok(app_annotations)
    }
}
