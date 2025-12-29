//! CASS database connection
//!
//! Provides read-only access to the CASS SQLite database.

use crate::models::Workspace;
use anyhow::{Context, Result};
use rusqlite::Connection;

/// CASS database connection wrapper.
pub struct CassDb {
    conn: Connection,
}

impl CassDb {
    /// Open CASS database at default location.
    pub fn open() -> Result<Self> {
        let path = Self::default_path()?;
        let conn = Connection::open(&path)
            .with_context(|| format!("Failed to open CASS db at {:?}", path))?;
        Ok(Self { conn })
    }

    /// Get the default CASS database path.
    fn default_path() -> Result<std::path::PathBuf> {
        let support = dirs::data_dir().context("Could not find Application Support directory")?;
        Ok(support
            .join("com.coding-agent-search.coding-agent-search")
            .join("agent_search.db"))
    }

    /// Count the number of conversations in the database.
    pub fn session_count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM conversations", [], |row| row.get(0))?;
        Ok(count)
    }

    /// Get all workspaces from the database.
    pub fn workspaces(&self) -> Result<Vec<Workspace>> {
        let mut stmt = self.conn.prepare("SELECT id, path FROM workspaces")?;
        let rows = stmt.query_map([], |row| {
            Ok(Workspace {
                id: row.get(0)?,
                path: row.get(1)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get a recent message from the database (for sampling).
    pub fn recent_message(&self) -> Result<Option<crate::models::Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, conversation_id, role, content, created_at
             FROM messages
             WHERE role = 'assistant' AND length(content) > 100
             ORDER BY created_at DESC
             LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(crate::models::Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_path_exists() {
        // Just verify path construction doesn't panic
        let result = CassDb::default_path();
        assert!(result.is_ok());
    }

    #[test]
    fn test_open_missing_db() {
        // If CASS not installed, should get clear error
        let result = CassDb::open();
        // Either succeeds or fails with context
        match result {
            Ok(_) => (), // CASS installed
            Err(e) => {
                assert!(e.to_string().contains("CASS") || e.to_string().contains("agent_search"))
            }
        }
    }

    // Integration test (requires CASS):
    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_session_count() {
        let db = CassDb::open().unwrap();
        let count = db.session_count().unwrap();
        assert!(count >= 0);
    }
}
