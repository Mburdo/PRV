//! Step 0 matching: single candidate = high confidence
//!
//! When exactly one session passes the workspace gate and time window,
//! return it with high confidence (0.9). No further disambiguation needed.

use super::Conversation;

/// Result of a successful match
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub conversation: Conversation,
    pub confidence: f64,
    pub step: u8,
}

/// Step 0: If exactly one candidate, return with high confidence.
///
/// This is the simplest matching case - when the gates narrow down
/// to a single session, we can be confident it produced the commit.
pub fn match_step0(candidates: &[Conversation]) -> Option<MatchResult> {
    if candidates.len() == 1 {
        Some(MatchResult {
            conversation: candidates[0].clone(),
            confidence: 0.9,
            step: 0,
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_conversation(id: i64) -> Conversation {
        Conversation {
            id,
            workspace_id: 1,
            started_at: 1000,
            ended_at: Some(2000),
        }
    }

    #[test]
    fn test_single_candidate_high_confidence() {
        let candidates = vec![mock_conversation(42)];
        let result = match_step0(&candidates);

        assert!(result.is_some(), "Single candidate should match");
        let r = result.unwrap();
        assert_eq!(r.confidence, 0.9, "Confidence should be 0.9");
        assert_eq!(r.step, 0, "Step should be 0");
        assert_eq!(r.conversation.id, 42, "Should return the candidate");
    }

    #[test]
    fn test_no_candidates_returns_none() {
        let candidates: Vec<Conversation> = vec![];
        let result = match_step0(&candidates);

        assert!(result.is_none(), "No candidates should return None");
    }

    #[test]
    fn test_multiple_candidates_returns_none() {
        let candidates = vec![mock_conversation(1), mock_conversation(2)];
        let result = match_step0(&candidates);

        assert!(
            result.is_none(),
            "Multiple candidates should return None (escalate to Step 1)"
        );
    }

    #[test]
    fn test_result_contains_correct_conversation() {
        let candidates = vec![mock_conversation(99)];
        let result = match_step0(&candidates).unwrap();

        assert_eq!(result.conversation.id, 99);
        assert_eq!(result.conversation.workspace_id, 1);
    }
}
