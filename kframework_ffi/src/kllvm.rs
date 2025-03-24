use std::ffi::{c_char, c_void};

#[repr(C)]
pub struct block {
    _private: [u8; 0], // Unused field. Makes rust happy about FFI safety.
}

#[repr(C)]
pub struct kore_pattern {
    _private: [u8; 0], // Unused field. Makes rust happy about FFI safety.
}

#[allow(dead_code)]
unsafe extern "C" {
    pub fn free(ptr: *const c_void) -> c_void;

    pub fn kllvm_init() -> c_void;
    pub fn kllvm_free_all_memory() -> c_void;

    pub fn take_steps(steps: i64, subject: *const block) -> *const block;

    pub fn kore_pattern_parse(data: *const c_char) -> *const kore_pattern;
    pub fn kore_pattern_dump(pattern: *const kore_pattern) -> *const c_char;
    pub fn kore_pattern_free(pattern: *const kore_pattern) -> c_void;

    pub fn kore_pattern_construct(pattern: *const kore_pattern) -> *const block;
    pub fn kore_pattern_from_block(subject: *const block) -> *const kore_pattern;
}
