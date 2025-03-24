//! # kllvm
//!
//! A safe interface for the K framework's llvm backend execution engine.
//!
//! This module declares foreign functions which are expected to be linked in
//! from an external interpreter library built by the kframework.
//!
//! ## Example
//!
//! ```no_run
//! use kframework_ffi::kllvm;
//!
//! const KORE_STRING: &str = "Lbl'-LT-'generatedTop'-GT-'{}( ...";
//!
//! kllvm::init();
//!
//! let pattern: kllvm::Pattern = KORE_STRING.parse().expect("Parsing failed!");
//! let mut block: kllvm::Block = pattern.into();
//!
//! block.take_steps(-1);
//!
//! let result: kllvm::Pattern = block.into();
//!
//! println!("{result}");
//!
//! kllvm::free_all_memory();
//! ```
mod block;
mod ffi;
mod pattern;

pub fn init() {
    unsafe {
        ffi::kllvm_init();
    }
}
pub fn free_all_memory() {
    unsafe {
        ffi::kllvm_free_all_memory();
    }
}

pub use self::block::Block;
pub use self::pattern::Pattern;
