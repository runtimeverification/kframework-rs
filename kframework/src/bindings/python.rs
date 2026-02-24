#![allow(unused)]

use std::collections::HashMap;

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

/// A metatada struct for the root node of a Sort
#[pyclass(unsendable)]
struct PySortView {
    root: Box<Sort>,
    /// A list of pointers to all sub-trees of the root
    nodes: Vec<*const Sort>,
    /// A list of children indices given a node index
    children: Vec<Box<[usize]>>,
}

impl PySortView {
    fn get(&self, idx: usize) -> Option<&Sort> {
        self.nodes.get(idx).map(|p| unsafe { &**p })
    }
}

impl Sort {
    fn into_view(self) -> PySortView {
        let root = Box::new(self);

        let mut nodes = Vec::new();
        let mut stack = vec![root.as_ref()];

        while let Some(n) = stack.pop() {
            nodes.push(n as *const Sort);

            if let Sort::App { args, .. } = n {
                stack.extend(args)
            };
        }

        let mut index_of = HashMap::new();

        for (i, &ptr) in nodes.iter().enumerate() {
            index_of.insert(ptr, i);
        }

        let mut children: Vec<Box<[usize]>> = Vec::with_capacity(nodes.len());
        for &p in &nodes {
            let s = unsafe { &*p };
            let child_idxs = match s {
                Sort::Var(_) => Vec::new(),
                Sort::App { args, .. } => args
                    .iter()
                    .map(|c| {
                        let cp = c as *const Sort;
                        *index_of.get(&cp).expect("Pointer didn't exist in lookup")
                    })
                    .collect(),
            };
            children.push(child_idxs.into_boxed_slice());
        }

        PySortView {
            root,
            nodes,
            children,
        }
    }
}

#[pyclass(subclass, name = "Sort")]
pub struct PySortNode {
    inner: Py<PySortView>,
    idx: usize,
}

fn create_node(view: &Bound<'_, PySortView>, idx: usize) -> PyResult<Py<PyAny>> {
    let py = view.py();
    let view_ref = view.borrow();
    let node = view_ref.get(idx).expect("Invalid node index");

    let sort = PySortNode {
        inner: view.clone().unbind(),
        idx,
    };
    let res: Py<PyAny> = match node {
        Sort::Var(id) => {
            let var = SortVar {
                name: PyString::new(py, id.value()).unbind(),
            };
            Py::new(py, (var, sort))?.into()
        }
        Sort::App { id, .. } => {
            let children_idxs = view_ref.children.get(idx).expect("Invalid node index");
            let children = children_idxs
                .iter()
                .map(|idx| create_node(view, *idx))
                .collect::<PyResult<Vec<Py<PyAny>>>>()?;
            let app = SortApp {
                name: PyString::new(py, id.value()).unbind(),
                args: PyTuple::new(py, children)?.into(),
            };
            Py::new(py, (app, sort))?.into()
        }
    };
    Ok(res)
}

#[pymethods]
impl PySortNode {
    #[staticmethod]
    fn parse(py: Python<'_>, s: &str) -> PyResult<Py<Self>> {
        use crate::kore::Parser;
        let sort: Sort = Parser::new(s)
            .and_then(|mut p| p.sort())
            .map_err(PyValueError::new_err)?;
        let view = Bound::new(py, sort.into_view())?;
        let res = create_node(&view, 0)?;
        Ok(res.extract(py)?)
    }
}

#[pyclass(extends = PySortNode)]
pub struct SortVar {
    #[pyo3(get)]
    name: Py<PyString>,
}

#[pymethods]
impl SortVar {
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
}

#[pyclass(extends = PySortNode)]
pub struct SortApp {
    #[pyo3(get)]
    name: Py<PyString>,
    #[pyo3(get)]
    args: Py<PyTuple>,
}
