//! Workspace gate and time index window for session filtering
//!
//! Filters candidate sessions that could have produced a commit.
//! Hard gate: workspace must match. Time window only bounds indexing.

use anyhow::Result;
use std::path::Path;

/// Conversation data needed for matching (subset of prv_cass::Conversation)
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: i64,
    pub workspace_id: i64,
    pub started_at: i64,       // Unix ms
    pub ended_at: Option<i64>, // Unix ms
}

/// Workspace data (subset of prv_cass::Workspace)
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: i64,
    pub path: String,
}

/// Trait for database operations, enabling test mocks
pub trait ConversationStore {
    fn find_workspace_for_path(&self, repo_path: &Path) -> Result<Option<Workspace>>;
    fn conversations_for_workspace(&self, workspace_id: i64) -> Result<Vec<Conversation>>;
}

/// Index window: sessions within this range of commit time are candidates
const INDEX_WINDOW_MS: i64 = 7 * 24 * 60 * 60 * 1000; // 7 days in ms

/// Filter sessions that could have produced this commit.
///
/// Hard gate: workspace must match.
/// Time window only bounds indexing (7 days).
pub fn candidate_sessions<S: ConversationStore>(
    store: &S,
    repo_path: &Path,
    commit_time_ms: i64,
) -> Result<Vec<Conversation>> {
    let workspace = store.find_workspace_for_path(repo_path)?;

    let Some(ws) = workspace else {
        return Ok(vec![]); // No CASS workspace for this repo
    };

    let conversations = store
        .conversations_for_workspace(ws.id)?
        .into_iter()
        .filter(|c| {
            let session_end = c.ended_at.unwrap_or(c.started_at);
            let start_ok = commit_time_ms + INDEX_WINDOW_MS >= c.started_at;
            let end_ok = commit_time_ms - INDEX_WINDOW_MS <= session_end;
            start_ok && end_ok
        })
        .collect();

    Ok(conversations)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock store for testing
    struct MockStore {
        workspace: Option<Workspace>,
        conversations: Vec<Conversation>,
    }

    impl ConversationStore for MockStore {
        fn find_workspace_for_path(&self, _repo_path: &Path) -> Result<Option<Workspace>> {
            Ok(self.workspace.clone())
        }

        fn conversations_for_workspace(&self, _workspace_id: i64) -> Result<Vec<Conversation>> {
            Ok(self.conversations.clone())
        }
    }

    fn mock_conversation(id: i64, started_at: i64, ended_at: Option<i64>) -> Conversation {
        Conversation {
            id,
            workspace_id: 1,
            started_at,
            ended_at,
        }
    }

    const DAY_MS: i64 = 24 * 60 * 60 * 1000;
    const HOUR_MS: i64 = 60 * 60 * 1000;

    #[test]
    fn test_no_workspace_returns_empty() {
        let store = MockStore {
            workspace: None,
            conversations: vec![mock_conversation(1, 1000, Some(2000))],
        };

        let result = candidate_sessions(&store, Path::new("/some/repo"), 1500).unwrap();
        assert!(result.is_empty(), "No workspace should return empty vec");
    }

    #[test]
    fn test_outside_index_window_filtered() {
        let commit_time = 100 * DAY_MS; // Day 100
        let store = MockStore {
            workspace: Some(Workspace {
                id: 1,
                path: "/repo".into(),
            }),
            conversations: vec![
                // Session ended 10 days before commit (outside 7-day window)
                mock_conversation(1, 80 * DAY_MS, Some(90 * DAY_MS)),
            ],
        };

        let result = candidate_sessions(&store, Path::new("/repo"), commit_time).unwrap();
        assert!(
            result.is_empty(),
            "Session outside window should be filtered"
        );
    }

    #[test]
    fn test_session_in_window_included() {
        let commit_time = 100 * DAY_MS; // Day 100
        let store = MockStore {
            workspace: Some(Workspace {
                id: 1,
                path: "/repo".into(),
            }),
            conversations: vec![
                // Session ended 1 hour before commit (within window)
                mock_conversation(1, 100 * DAY_MS - 2 * HOUR_MS, Some(100 * DAY_MS - HOUR_MS)),
            ],
        };

        let result = candidate_sessions(&store, Path::new("/repo"), commit_time).unwrap();
        assert_eq!(result.len(), 1, "Session in window should be included");
        assert_eq!(result[0].id, 1);
    }

    #[test]
    fn test_session_spanning_commit_time_included() {
        let commit_time = 100 * DAY_MS;
        let store = MockStore {
            workspace: Some(Workspace {
                id: 1,
                path: "/repo".into(),
            }),
            conversations: vec![
                // Session started before and ended after commit
                mock_conversation(1, 100 * DAY_MS - HOUR_MS, Some(100 * DAY_MS + HOUR_MS)),
            ],
        };

        let result = candidate_sessions(&store, Path::new("/repo"), commit_time).unwrap();
        assert_eq!(
            result.len(),
            1,
            "Session spanning commit should be included"
        );
    }

    #[test]
    fn test_ongoing_session_uses_started_at() {
        let commit_time = 100 * DAY_MS;
        let store = MockStore {
            workspace: Some(Workspace {
                id: 1,
                path: "/repo".into(),
            }),
            conversations: vec![
                // Ongoing session (no ended_at), started within window
                mock_conversation(1, 100 * DAY_MS - HOUR_MS, None),
            ],
        };

        let result = candidate_sessions(&store, Path::new("/repo"), commit_time).unwrap();
        assert_eq!(
            result.len(),
            1,
            "Ongoing session should use started_at as end"
        );
    }

    #[test]
    fn test_multiple_sessions_filtered_correctly() {
        let commit_time = 100 * DAY_MS;
        let store = MockStore {
            workspace: Some(Workspace {
                id: 1,
                path: "/repo".into(),
            }),
            conversations: vec![
                // In window
                mock_conversation(1, 99 * DAY_MS, Some(99 * DAY_MS + HOUR_MS)),
                // Out of window (too old)
                mock_conversation(2, 80 * DAY_MS, Some(81 * DAY_MS)),
                // In window
                mock_conversation(3, 100 * DAY_MS - HOUR_MS, Some(100 * DAY_MS)),
                // Out of window (too new)
                mock_conversation(4, 110 * DAY_MS, Some(111 * DAY_MS)),
            ],
        };

        let result = candidate_sessions(&store, Path::new("/repo"), commit_time).unwrap();
        assert_eq!(result.len(), 2, "Should have 2 sessions in window");

        let ids: Vec<i64> = result.iter().map(|c| c.id).collect();
        assert!(ids.contains(&1), "Session 1 should be included");
        assert!(ids.contains(&3), "Session 3 should be included");
    }
}
