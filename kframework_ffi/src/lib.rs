//! # kframework_ffi
//!
//! This crate provides the interface for foreign functions in the K framework.
//!
//! ## Notable Interfaces
//!
//! - [`kllvm::Pattern`]: kllvm's kore AST.
//! - [`kllvm::Block`]: kllvm's interned kore representation for execution.
pub mod kllvm;
