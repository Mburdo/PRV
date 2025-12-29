//! prv-core: Core types and utilities for PRV
//!
//! This crate provides the foundational data structures for code block
//! extraction and fingerprinting.

pub mod code_block;
pub mod link;
pub mod link_storage;
pub mod matcher;

pub use code_block::{extract_code_blocks, CodeBlock};
pub use link::Link;
pub use link_storage::LinkStorage;
pub use matcher::{candidate_sessions, Conversation, ConversationStore, Workspace};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
