//! CASS database connection
//!
//! Provides read-only access to the CASS SQLite database.

use crate::models::{Conversation, Message, Snippet, Workspace};
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
    pub fn recent_message(&self) -> Result<Option<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, conversation_id, role, content, created_at
             FROM messages
             WHERE role = 'assistant' AND length(content) > 100
             ORDER BY created_at DESC
             LIMIT 1",
        )?;

        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(Message {
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

    /// Get all conversations for a workspace.
    ///
    /// Returns conversations ordered by started_at descending (most recent first).
    pub fn conversations_for_workspace(&self, workspace_id: i64) -> Result<Vec<Conversation>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, workspace_id, started_at, ended_at, title, source_path
             FROM conversations
             WHERE workspace_id = ?
             ORDER BY started_at DESC",
        )?;

        let rows = stmt.query_map([workspace_id], |row| {
            Ok(Conversation {
                id: row.get(0)?,
                workspace_id: row.get(1)?,
                started_at: row.get(2)?,
                ended_at: row.get(3)?,
                title: row.get(4)?,
                source_path: row.get(5)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get all messages for a conversation.
    ///
    /// Returns messages ordered by idx (message order within conversation).
    pub fn messages_for_conversation(&self, conversation_id: i64) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, conversation_id, role, content, created_at
             FROM messages
             WHERE conversation_id = ?
             ORDER BY idx ASC",
        )?;

        let rows = stmt.query_map([conversation_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    /// Get all code snippets for a conversation.
    ///
    /// Joins through messages table to get all snippets belonging to
    /// messages in the specified conversation.
    pub fn snippets_for_conversation(&self, conversation_id: i64) -> Result<Vec<Snippet>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.message_id, s.file_path, s.start_line, s.end_line,
                    s.language, s.snippet_text
             FROM snippets s
             JOIN messages m ON s.message_id = m.id
             WHERE m.conversation_id = ?
             ORDER BY m.idx ASC, s.id ASC",
        )?;

        let rows = stmt.query_map([conversation_id], |row| {
            Ok(Snippet {
                id: row.get(0)?,
                message_id: row.get(1)?,
                file_path: row.get(2)?,
                start_line: row.get(3)?,
                end_line: row.get(4)?,
                language: row.get(5)?,
                snippet_text: row.get(6)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
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

    // Integration tests (require CASS):
    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_session_count() {
        let db = CassDb::open().unwrap();
        let count = db.session_count().unwrap();
        assert!(count >= 0);
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_conversations_for_workspace() {
        let db = CassDb::open().unwrap();
        let workspaces = db.workspaces().unwrap();

        if let Some(ws) = workspaces.first() {
            let convs = db.conversations_for_workspace(ws.id).unwrap();
            // Just verify query works; may be empty
            for conv in &convs {
                assert_eq!(conv.workspace_id, Some(ws.id));
            }
        }
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_messages_for_conversation() {
        let db = CassDb::open().unwrap();
        let workspaces = db.workspaces().unwrap();

        if let Some(ws) = workspaces.first() {
            let convs = db.conversations_for_workspace(ws.id).unwrap();
            if let Some(conv) = convs.first() {
                let msgs = db.messages_for_conversation(conv.id).unwrap();
                // Verify all messages belong to this conversation
                for msg in &msgs {
                    assert_eq!(msg.conversation_id, conv.id);
                }
            }
        }
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored
    fn test_snippets_for_conversation() {
        let db = CassDb::open().unwrap();
        let workspaces = db.workspaces().unwrap();

        if let Some(ws) = workspaces.first() {
            let convs = db.conversations_for_workspace(ws.id).unwrap();
            if let Some(conv) = convs.first() {
                // Query works even if no snippets
                let snippets = db.snippets_for_conversation(conv.id).unwrap();
                // Snippets may be empty, just verify query succeeds
                let _ = snippets;
            }
        }
    }
}
