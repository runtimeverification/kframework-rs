use super::ffi;
use super::{Block, Sort, Symbol};
use libc;
use std::ffi::{c_void, CStr, CString, NulError};
use std::fmt;
use std::str::FromStr;

/// A safe wrapper around a pointer to kllvm's kore AST.
///
/// kllvm expects the caller to free this object when it's
/// finished with it. `Drop` is implemented to do this.
pub struct Pattern {
    pub(crate) pattern: *const ffi::kore_pattern,
}

impl Pattern {
    pub fn new(s: &str) -> Result<Pattern, <Pattern as FromStr>::Err> {
        Pattern::from_str(s)
    }

    pub fn from_raw(p: *mut ffi::kore_pattern) -> Self {
        Self { pattern: p }
    }

    /// Build a fresh composite pattern from a symbol. The symbol is borrowed;
    /// the C++ side keeps a `shared_ptr` to its underlying AST.
    pub fn from_symbol(sym: &Symbol) -> Self {
        let raw = unsafe { ffi::kore_composite_pattern_from_symbol(sym.symbol) };
        Self { pattern: raw }
    }

    /// Build a fresh token (Dv) pattern of the given sort.
    pub fn new_token(value: &str, sort: &Sort) -> Result<Self, NulError> {
        let c_val = CString::new(value)?;
        let raw = unsafe { ffi::kore_pattern_new_token(c_val.as_ptr(), sort.sort) };
        Ok(Self { pattern: raw })
    }

    /// Append `child` as an argument to this composite pattern. The child
    /// is borrowed; the C++ side takes a `shared_ptr` copy of its AST, so
    /// the caller may continue to use or drop `child` afterwards.
    pub fn add_argument(&mut self, child: &Pattern) {
        unsafe { ffi::kore_composite_pattern_add_argument(self.pattern as *mut _, child.pattern) };
    }
}

impl FromStr for Pattern {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let c_str = match CString::new(s) {
            Ok(s) => s.into_raw(),
            Err(e) => return Err(format!("Error generating CString: {}", e)),
        };
        let pattern = unsafe { ffi::kore_pattern_parse(c_str) };
        let _ = unsafe { CString::from_raw(c_str) }; // Free the CString memory
        Ok(Self { pattern })
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = unsafe {
            let c_str = ffi::kore_pattern_dump(self.pattern);
            let result = CStr::from_ptr(c_str)
                .to_str()
                .expect("Failed to convert kllvm::Pattern to &str")
                .to_string();
            libc::free(c_str as *mut c_void);
            result
        };
        write!(f, "{}", result)
    }
}

impl From<Block> for Pattern {
    fn from(subject: Block) -> Self {
        let result = unsafe { ffi::kore_pattern_from_block(subject.block) };
        Self { pattern: result }
    }
}

impl Drop for Pattern {
    fn drop(&mut self) {
        unsafe { ffi::kore_pattern_free(self.pattern) };
    }
}
