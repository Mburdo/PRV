//! CASS database models
//!
//! These structs map to the CASS SQLite schema tables for workspaces,
//! conversations, messages, and snippets. All access is read-only.

/// A workspace tracked by CASS, representing a project directory.
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: i64,
    pub path: String,
}

/// A conversation (AI session) within a workspace.
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: i64,
    pub workspace_id: Option<i64>, // Can be NULL in CASS schema
    pub started_at: Option<i64>,   // Unix ms, can be NULL
    pub ended_at: Option<i64>,     // Unix ms
    pub title: Option<String>,
    pub source_path: Option<String>,
}

/// A message within a conversation.
#[derive(Debug, Clone)]
pub struct Message {
    pub id: i64,
    pub conversation_id: i64,
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub created_at: Option<i64>, // Unix ms, can be NULL
}

/// A code snippet extracted from a message by CASS.
#[derive(Debug, Clone)]
pub struct Snippet {
    pub id: i64,
    pub message_id: i64,
    pub file_path: Option<String>,
    pub start_line: Option<i32>,
    pub end_line: Option<i32>,
    pub language: Option<String>,
    pub snippet_text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_creation() {
        let ws = Workspace {
            id: 1,
            path: "/home/user/project".to_string(),
        };
        assert_eq!(ws.id, 1);
    }

    #[test]
    fn test_conversation_optional_fields() {
        let conv = Conversation {
            id: 1,
            workspace_id: Some(1),
            started_at: Some(1000),
            ended_at: None,
            title: None,
            source_path: None,
        };
        assert!(conv.ended_at.is_none());
        assert_eq!(conv.workspace_id, Some(1));
    }

    #[test]
    fn test_conversation_null_workspace() {
        let conv = Conversation {
            id: 1,
            workspace_id: None,
            started_at: Some(1000),
            ended_at: None,
            title: None,
            source_path: None,
        };
        assert!(conv.workspace_id.is_none());
    }

    #[test]
    fn test_message_roles() {
        let msg = Message {
            id: 1,
            conversation_id: 1,
            role: "assistant".to_string(),
            content: "Hello".to_string(),
            created_at: Some(1000),
        };
        assert_eq!(msg.role, "assistant");
    }

    #[test]
    fn test_snippet_creation() {
        let snippet = Snippet {
            id: 1,
            message_id: 1,
            file_path: Some("src/main.rs".to_string()),
            start_line: Some(1),
            end_line: Some(10),
            language: Some("rust".to_string()),
            snippet_text: Some("fn main() {}".to_string()),
        };
        assert_eq!(snippet.file_path, Some("src/main.rs".to_string()));
    }
}
