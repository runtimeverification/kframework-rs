use super::ffi;
use super::Sort;
use std::ffi::{CString, NulError};

/// Safe wrapper around a `kore_symbol *`. Drops via `kore_symbol_free`.
///
/// The underlying C++ symbol is held via a `shared_ptr`; dropping this
/// wrapper only releases the C struct, so patterns built from this symbol
/// stay valid for the rest of their lifetime.
pub struct Symbol {
    pub(crate) symbol: *mut ffi::kore_symbol,
}

impl Symbol {
    /// Build a fresh symbol by name (e.g. `Lblfoo`). Add any formal sort
    /// arguments with [`add_formal_argument`].
    pub fn new(name: &str) -> Result<Self, NulError> {
        let c_name = CString::new(name)?;
        let sym = unsafe { ffi::kore_symbol_new(c_name.as_ptr()) };
        Ok(Self { symbol: sym })
    }

    /// Append one formal sort argument to the symbol's signature.
    pub fn add_formal_argument(&mut self, sort: &Sort) {
        unsafe { ffi::kore_symbol_add_formal_argument(self.symbol, sort.sort) };
    }
}

impl Drop for Symbol {
    fn drop(&mut self) {
        unsafe { ffi::kore_symbol_free(self.symbol) };
    }
}
