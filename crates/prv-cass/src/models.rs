//! CASS database models
//!
//! These structs map to the CASS SQLite schema tables for workspaces,
//! conversations, and messages.

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
    pub workspace_id: i64,
    pub started_at: i64, // Unix ms
    pub ended_at: Option<i64>,
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
    pub created_at: i64, // Unix ms
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
            workspace_id: 1,
            started_at: 1000,
            ended_at: None,
            title: None,
            source_path: None,
        };
        assert!(conv.ended_at.is_none());
    }

    #[test]
    fn test_message_roles() {
        let msg = Message {
            id: 1,
            conversation_id: 1,
            role: "assistant".to_string(),
            content: "Hello".to_string(),
            created_at: 1000,
        };
        assert_eq!(msg.role, "assistant");
    }
}
