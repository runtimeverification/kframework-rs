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
            use pyo3::{prelude::*, types::IntoPyDict};

            #[pymodule_init]
            fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
                create_sys_module(module, "kframework_py.kore.syntax")?;

                let py = module.py();

                // Each python class has the `__annotations__` attribute set.
                //
                // This allows us to call the `dataclass` decorator on them.
                let dataclass = py.import("dataclasses")?.getattr("dataclass")?;
                let kwargs = Some([("init", false), ("frozen", true)].into_py_dict(py)?);

                // Sort subclasses
                dataclass.call((module.getattr("SortVar")?,), kwargs.as_ref())?;
                dataclass.call((module.getattr("SortApp")?,), kwargs.as_ref())?;

                // Pattern subclasses
                for class_name in [
                    "EVar",
                    "SVar",
                    "String",
                    "App",
                    "LeftAssoc",
                    "RightAssoc",
                    "Top",
                    "Bottom",
                    "DV",
                    "Not",
                    "Implies",
                    "Iff",
                    "And",
                    "Or",
                    "Exists",
                    "Forall",
                    "Mu",
                    "Nu",
                    "Ceil",
                    "Floor",
                    "Equals",
                    "In",
                    "Next",
                    "Rewrites",
                ] {
                    dataclass.call((module.getattr(class_name)?,), kwargs.as_ref())?;
                }

                // Symbol (standalone) + Sentence subclasses
                dataclass.call((module.getattr("Symbol")?,), kwargs.as_ref())?;
                for class_name in [
                    "Import",
                    "SortDecl",
                    "SymbolDecl",
                    "AliasDecl",
                    "Axiom",
                    "Claim",
                ] {
                    dataclass.call((module.getattr(class_name)?,), kwargs.as_ref())?;
                }

                // Module + Definition (standalone)
                dataclass.call((module.getattr("Module")?,), kwargs.as_ref())?;
                dataclass.call((module.getattr("Definition")?,), kwargs.as_ref())?;

                Ok(())
            }

            #[pymodule_export]
            use kframework::bindings::python::{
                AliasDecl, And, App, Axiom, Bottom, Ceil, Claim, EVar, Equals, Exists, Floor,
                Forall, Iff, Implies, Import, In, KoreDefinition, KoreModule, KoreString,
                LeftAssoc, Mu, Next, Not, Nu, Or, PyPattern, PySVar, PySentence, PySort, Rewrites,
                RightAssoc, SortApp, SortDecl, SortVar, Symbol, SymbolDecl, Top, DV,
            };
        }
    }
}
