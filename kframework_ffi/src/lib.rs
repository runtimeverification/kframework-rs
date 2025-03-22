use std::ffi::{CStr, CString};
mod kllvm;

pub struct KllvmPattern {
    pattern: *const kllvm::kore_pattern
}

pub struct KllvmBlock {
    block: *const kllvm::block
}

pub fn kllvm_init() { unsafe { kllvm::kllvm_init(); } }
pub fn kllvm_free_all_memory() { unsafe { kllvm::kllvm_free_all_memory(); } }

pub fn take_steps(steps: i64, subject: KllvmBlock) -> KllvmBlock {
    let result = unsafe { kllvm::take_steps(steps, subject.block) };
    KllvmBlock{ block: result }
}

pub fn kore_pattern_parse(data: &str) -> KllvmPattern {
    let c_str = CString::new(data).expect("CString::new failed").into_raw();
    let pattern = unsafe { kllvm::kore_pattern_parse(c_str) };
    let _ = unsafe { CString::from_raw(c_str) }; // Free the CString memory
    KllvmPattern { pattern: pattern }
}

pub fn kore_pattern_dump(pattern: &KllvmPattern) -> String {
    unsafe {
        let c_str = kllvm::kore_pattern_dump(pattern.pattern);
        CStr::from_ptr(c_str).to_str().expect("Failed to convert KllvmPattern to &str").to_string()
    }
}

pub fn kore_pattern_construct(pattern: &KllvmPattern) -> KllvmBlock {
    let block = unsafe { kllvm::kore_pattern_construct(pattern.pattern) };
    KllvmBlock { block: block }
}

pub fn kore_pattern_from_block(subject: &KllvmBlock) -> KllvmPattern {
    let pattern = unsafe { kllvm::kore_pattern_from_block(subject.block) };
    KllvmPattern { pattern: pattern }
}

impl From<KllvmPattern> for KllvmBlock {
    fn from(pattern: KllvmPattern) -> KllvmBlock {
        kore_pattern_construct(&pattern)
    }
}

impl From<KllvmBlock> for KllvmPattern {
    fn from(block: KllvmBlock) -> KllvmPattern {
        kore_pattern_from_block(&block)
    }
}