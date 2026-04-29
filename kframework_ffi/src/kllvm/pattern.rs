use libc;
use super::ffi;
use super::Block;
use std::ffi::{c_void, CStr, CString};
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
