//! prv-core: Core types and utilities for PRV
//!
//! This crate provides the foundational data structures for code block
//! extraction and fingerprinting.

pub mod code_block;
pub use code_block::{extract_code_blocks, CodeBlock};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
