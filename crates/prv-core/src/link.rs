//! Link model for commit-to-session associations.
//!
//! A Link represents the connection between a git commit and the AI session
//! that produced it, including confidence score and matching metadata.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A link between a git commit and an AI session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    /// The git commit SHA this link refers to.
    pub commit_sha: String,
    /// The CASS session ID that produced this commit.
    pub session_id: i64,
    /// Confidence score (0.0 to 1.0) of the match.
    pub confidence: f64,
    /// Which matching step produced this link (0 = single candidate, 1 = file hints, etc.)
    pub match_step: u8,
    /// When this link was created.
    pub created_at: DateTime<Utc>,
}

impl Link {
    /// Create a new Link with the current timestamp.
    pub fn new(commit_sha: &str, session_id: i64, confidence: f64, step: u8) -> Self {
        Self {
            commit_sha: commit_sha.to_string(),
            session_id,
            confidence,
            match_step: step,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_new() {
        let link = Link::new("abc123", 42, 0.9, 0);
        assert_eq!(link.commit_sha, "abc123");
        assert_eq!(link.session_id, 42);
        assert_eq!(link.confidence, 0.9);
        assert_eq!(link.match_step, 0);
    }

    #[test]
    fn test_link_serialization_roundtrip() {
        let link = Link::new("abc123def456", 42, 0.95, 1);
        let json = serde_json::to_string(&link).unwrap();
        let parsed: Link = serde_json::from_str(&json).unwrap();

        assert_eq!(link.commit_sha, parsed.commit_sha);
        assert_eq!(link.session_id, parsed.session_id);
        assert_eq!(link.confidence, parsed.confidence);
        assert_eq!(link.match_step, parsed.match_step);
        assert_eq!(link.created_at, parsed.created_at);
    }

    #[test]
    fn test_link_json_structure() {
        let link = Link::new("abc123", 42, 0.9, 0);
        let json = serde_json::to_string(&link).unwrap();

        // Verify expected fields are present
        assert!(json.contains("\"commit_sha\":\"abc123\""));
        assert!(json.contains("\"session_id\":42"));
        assert!(json.contains("\"confidence\":0.9"));
        assert!(json.contains("\"match_step\":0"));
        assert!(json.contains("\"created_at\":"));
    }

    #[test]
    fn test_link_clone() {
        let link = Link::new("abc123", 42, 0.9, 0);
        let cloned = link.clone();

        assert_eq!(link.commit_sha, cloned.commit_sha);
        assert_eq!(link.session_id, cloned.session_id);
        assert_eq!(link.confidence, cloned.confidence);
        assert_eq!(link.match_step, cloned.match_step);
        assert_eq!(link.created_at, cloned.created_at);
    }

    #[test]
    fn test_link_equality_same_timestamp() {
        // Two links with same data should be equal
        let timestamp = Utc::now();
        let a = Link {
            commit_sha: "abc".to_string(),
            session_id: 1,
            confidence: 0.9,
            match_step: 0,
            created_at: timestamp,
        };
        let b = Link {
            commit_sha: "abc".to_string(),
            session_id: 1,
            confidence: 0.9,
            match_step: 0,
            created_at: timestamp,
        };
        assert_eq!(a, b);
    }

    #[test]
    fn test_link_debug_format() {
        let link = Link::new("abc123", 42, 0.9, 0);
        let debug = format!("{:?}", link);

        assert!(debug.contains("Link"));
        assert!(debug.contains("abc123"));
        assert!(debug.contains("42"));
    }
}
