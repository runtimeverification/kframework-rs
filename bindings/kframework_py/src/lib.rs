use pyo3::prelude::*;

// In order to get clean import semantics like "from a.b import c" for the
// bindings modules, the sys.modules dict needs to be updated with "a.b",
// "a.b.c", etc.
//
// We use this helper function to do that in #[pymodule_init] functions for each
// module.
fn create_sys_module(module: &Bound<'_, PyModule>, module_name: &str) -> PyResult<()> {
    module
        .py()
        .import("sys")?
        .getattr("modules")?
        .set_item(module_name, module)
}

/// A Python module implemented in Rust.
#[pymodule]
mod kframework_py {
    use pyo3::prelude::*;

    #[pymodule]
    mod kore {
        use crate::create_sys_module;
        use pyo3::prelude::*;

        #[pymodule_init]
        fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
            create_sys_module(module, "kframework_py.kore")
        }

        #[pymodule]
        mod syntax {
            use crate::create_sys_module;
            use pyo3::{
                prelude::*,
                types::{IntoPyDict, PyDict, PyList, PyString},
            };

            #[pymodule_init]
            fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
                create_sys_module(module, "kframework_py.kore.syntax")?;

                let py = module.py();
                let sortvar = module.getattr("SortVar")?;
                let sortapp = module.getattr("SortApp")?;

                let stringtype = py.get_type::<PyString>();
                let listtype = py.get_type::<PyList>();

                let var_annotations = PyDict::new(py);
                var_annotations.set_item("name", &stringtype)?;
                sortvar.setattr("__annotations__", var_annotations)?;

                let app_annotations = PyDict::new(py);
                app_annotations.set_item("name", &stringtype)?;
                app_annotations.set_item("args", &listtype)?;
                sortapp.setattr("__annotations__", app_annotations)?;

                let dataclass = py.import("dataclasses")?.getattr("dataclass")?;
                let kwargs = Some([("init", false), ("frozen", true)].into_py_dict(py)?);
                dataclass.call((sortvar,), kwargs.as_ref())?;
                dataclass.call((sortapp,), kwargs.as_ref())?;
                Ok(())
            }

            #[pymodule_export]
            use kframework::{
                bindings::python::{PySortNode, SortApp, SortVar},
                kore::Id,
            };
        }
    }
}
