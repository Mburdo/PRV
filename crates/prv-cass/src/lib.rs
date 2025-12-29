//! prv-cass: CASS database integration for PRV
//!
//! This crate provides read-only access to the CASS SQLite database
//! for session data used in commit-to-session linking.

pub mod db;
pub mod models;
pub mod workspace;

pub use db::CassDb;
pub use models::{Conversation, Message, Snippet, Workspace};
pub use workspace::find_workspace_for_repo;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
