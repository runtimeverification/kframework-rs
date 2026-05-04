use super::ffi;
use std::ffi::{CString, NulError};

/// Safe wrapper around a `kore_sort *`. Drops via `kore_sort_free`.
///
/// The underlying C++ AST node is held via a `shared_ptr` on the C++ side;
/// dropping this wrapper only releases the C struct that points to it, so
/// any patterns or symbols that reference this sort remain valid.
pub struct Sort {
    pub(crate) sort: *mut ffi::kore_sort,
}

impl Sort {
    /// Build a fresh composite sort by name (e.g. "SortInt"). Add any sort
    /// arguments with [`add_argument`].
    pub fn new_composite(name: &str) -> Result<Self, NulError> {
        let c_name = CString::new(name)?;
        let raw = unsafe { ffi::kore_composite_sort_new(c_name.as_ptr()) };
        Ok(Self { sort: raw })
    }

    /// Append a sort argument (for parameterised sorts).
    pub fn add_argument(&mut self, arg: &Sort) {
        unsafe { ffi::kore_composite_sort_add_argument(self.sort, arg.sort) };
    }
}

impl Drop for Sort {
    fn drop(&mut self) {
        unsafe { ffi::kore_sort_free(self.sort) };
    }
}
