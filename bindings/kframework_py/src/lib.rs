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
            use pyo3::prelude::*;

            #[pymodule_init]
            fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
                create_sys_module(module, "kframework_py.kore.syntax")
            }

            #[pymodule_export]
            use kframework::kore::Id;
        }
    }
}
