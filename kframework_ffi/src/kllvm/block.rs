use super::ffi;
use super::Pattern;
use std::ffi::{c_void, CStr};
use std::fmt;

/// A safe wrapper around a foreign pointer to kllvm's interned representation.
/// kllvm's garbage collector manages the allocation/freeing of this pointer.
pub struct Block {
    pub(crate) block: *mut ffi::block,
}

impl Block {
    pub fn new(pattern: &Pattern) -> Self {
        let result = unsafe { ffi::kore_pattern_construct(pattern.pattern) };
        Self { block: result }
    }

    /// Execute the semantics for a given number of steps over the term.
    ///
    /// Pass `-1` to `steps` to execute until no more rules apply.
    ///
    /// Note that the current pointer will likely be cleaned up by kllvm's garbage
    /// collection during execution and will no longer be a valid pointer, so
    /// the Block will be updated to point at the resulting term.
    pub fn take_steps(&mut self, steps: i64) {
        self.block = unsafe { ffi::take_steps(steps, self.block) };
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let result = unsafe {
            let c_str = ffi::kore_block_dump(self.block);
            let result = CStr::from_ptr(c_str)
                .to_str()
                .expect("Failed to convert kllvm::Block to &str")
                .to_string();
            libc::free(c_str as *mut c_void);
            result
        };
        write!(f, "{}", result)
    }
}

impl From<Pattern> for Block {
    fn from(pattern: Pattern) -> Block {
        Self::new(&pattern)
    }
}

impl From<&Pattern> for Block {
    fn from(pattern: &Pattern) -> Block {
        Self::new(pattern)
    }
}
