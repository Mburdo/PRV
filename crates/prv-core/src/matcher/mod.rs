//! Commit-to-session matching algorithms
//!
//! Multi-step matching pipeline:
//! - Step 0: Single candidate = high confidence
//! - Step 1: File path hints narrow candidates
//! - Step 2: Line hash matching for precision

pub mod gates;

pub use gates::{candidate_sessions, Conversation, ConversationStore, Workspace};
