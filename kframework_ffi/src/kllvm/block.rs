use super::ffi;
use super::Pattern;

/// A safe wrapper around a foreign pointer to kllvm's interned representation.
/// kllvm's garbage collector manages the allocation/freeing of this pointer.
pub struct Block {
    pub(crate) block: *const ffi::block,
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
