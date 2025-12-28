//! prv-cass: CASS database integration for PRV
//!
//! This crate provides read-only access to the CASS SQLite database
//! for session data used in commit-to-session linking.

pub mod models;
pub mod db;
pub mod workspace;

pub use models::{Workspace, Conversation, Message};
pub use db::CassDb;
pub use workspace::find_workspace_for_repo;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
