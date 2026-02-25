#![allow(unused)]

use std::{collections::HashMap, sync::Arc};

use crate::kore::{Id, Sort};
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{PyString, PyTuple},
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

#[pyclass(unsendable)]
#[derive(Default)]
pub struct PySortArena {
    inners: Vec<Arc<Sort>>,
    index_of: HashMap<*const Sort, usize>,
    children: Vec<Box<[usize]>>,
    cache: HashMap<usize, Py<PyAny>>,
}

impl PySortArena {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, sort: Arc<Sort>) -> usize {
        let idx = self.inners.len();
        self.inners.push(sort.clone());
        self.index_of
            .insert(Arc::<Sort>::into_raw(sort.clone()), idx);
        self.children.push(Box::new([]));

        if let Sort::App { args, .. } = &*sort {
            let children = args
                .iter()
                .map(|arg| self.add(arg.clone()))
                .collect::<Box<_>>();
            self.children[idx] = children;
        }

        idx
    }
}

#[pymethods]
impl PySortArena {
    fn get<'py>(
        slf: &Bound<'py, Self>,
        py: Python<'py>,
        idx: usize,
    ) -> PyResult<Bound<'py, PyAny>> {
        let slf_ref = slf.borrow();

        // Check the cache
        if let Some(res) = &slf_ref.cache.get(&idx) {
            return Ok(res.bind(py).clone());
        }

        // Otherwise create the python object for the node, update the cache
        // with it (if possible), and return it
        let sort = slf_ref.inners.get(idx).expect("Invalid node index");
        let node = PySortNode {
            inner: slf.clone().unbind(),
            idx,
        };
        let res: Bound<'_, PyAny> = match &**sort {
            Sort::Var(id) => {
                let var = SortVar {
                    name: PyString::new(py, id.value()).unbind(),
                };
                Bound::new(py, (var, node))?.into_any()
            }
            Sort::App { id, .. } => {
                let children_idxs = slf_ref.children.get(idx).expect("Invalid node index");
                let children = children_idxs
                    .iter()
                    .map(|idx| PySortArena::get(slf, py, *idx))
                    .collect::<Result<Vec<_>, _>>()?;
                let app = SortApp {
                    name: PyString::new(py, id.value()).unbind(),
                    args: PyTuple::new(py, children)?.into(),
                };
                Bound::new(py, (app, node))?.into_any()
            }
        };

        drop(slf_ref);

        if let Ok(mut slf_ref_mut) = slf.try_borrow_mut() {
            slf_ref_mut.cache.insert(idx, res.clone().unbind());
        }

        Ok(res)
    }
}

#[pyclass(subclass, name = "Sort")]
pub struct PySortNode {
    inner: Py<PySortArena>,
    idx: usize,
}

#[pymethods]
impl PySortNode {
    #[staticmethod]
    fn parse(py: Python<'_>, s: &str) -> PyResult<Py<PyAny>> {
        use crate::kore::Parser;
        let sort: Sort = Parser::new(s)
            .and_then(|mut p| p.sort())
            .map_err(PyValueError::new_err)?;
        let mut arena = PySortArena::new();
        let idx = arena.add(sort.into());
        let arena_bound = Bound::new(py, arena)?;
        PySortArena::get(&arena_bound, py, idx).map(|a| a.into_any().unbind())
    }
}

#[pyclass(extends = PySortNode)]
pub struct SortVar {
    #[pyo3(get)]
    name: Py<PyString>,
}

#[pymethods]
impl SortVar {
    /*
    #[new]
    fn new_(py: Python<'_>, id: Py<PyAny>) -> PyResult<Py<PyAny>> {
        let view = if let Ok(id) = id.extract::<Id>(py) {
            let sort = Sort::Var(id);
            Ok(sort.into_view())
        } else if let Ok(id_str) = id.cast_bound::<PyString>(py) {
            let id = Id::new(id_str.to_string()).map_err(PyValueError::new_err)?;
            let sort = Sort::Var(id);
            Ok(sort.into_view())
        } else {
            Err(PyTypeError::new_err(""))
        }?;
        let view_bound = Bound::new(py, view)?;
        create_node(&view_bound, 0)
    }
    */
}

#[pyclass(extends = PySortNode)]
pub struct SortApp {
    #[pyo3(get)]
    name: Py<PyString>,
    #[pyo3(get)]
    args: Py<PyTuple>,
}
