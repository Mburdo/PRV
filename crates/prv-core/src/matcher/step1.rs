//! Step 1 matching: file path hints
//!
//! When Step 0 returns None (multiple candidates), check for file path
//! overlap between commit files and files mentioned in sessions.

use std::collections::HashSet;

use super::step0::MatchResult;
use super::Conversation;

/// Trait for conversations that can report mentioned files
pub trait FileMentions {
    fn file_mentions(&self) -> Vec<String>;
}

/// Extended conversation with file mentions for Step 1 matching
#[derive(Debug, Clone)]
pub struct ConversationWithFiles {
    pub conversation: Conversation,
    pub mentioned_files: Vec<String>,
}

impl FileMentions for ConversationWithFiles {
    fn file_mentions(&self) -> Vec<String> {
        self.mentioned_files.clone()
    }
}

/// Step 1: Disambiguate by file path mentions in session.
///
/// Returns the first candidate with file path overlap, with confidence
/// bonus based on number of overlapping files (capped at +0.15).
pub fn match_step1<C: FileMentions + Clone>(
    candidates: &[(C, Conversation)],
    commit_files: &[String],
) -> Option<MatchResult> {
    if commit_files.is_empty() {
        return None;
    }

    let commit_set: HashSet<&String> = commit_files.iter().collect();

    for (mentions, conv) in candidates {
        let mentioned = mentions.file_mentions();
        let overlap = mentioned.iter().filter(|f| commit_set.contains(f)).count();

        if overlap > 0 {
            // Confidence bonus: +0.05 per overlap, capped at +0.15
            let bonus = (overlap.min(3) as f64) * 0.05;
            return Some(MatchResult {
                conversation: conv.clone(),
                confidence: 0.85 + bonus,
                step: 1,
            });
        }
    }

    None
}

/// Simplified version that works with ConversationWithFiles directly
pub fn match_step1_simple(
    candidates: &[ConversationWithFiles],
    commit_files: &[String],
) -> Option<MatchResult> {
    let pairs: Vec<_> = candidates
        .iter()
        .map(|c| (c.clone(), c.conversation.clone()))
        .collect();
    match_step1(&pairs, commit_files)
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

    fn mock_with_files(id: i64, files: Vec<&str>) -> ConversationWithFiles {
        ConversationWithFiles {
            conversation: mock_conversation(id),
            mentioned_files: files.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_file_overlap_matches() {
        let candidates = vec![mock_with_files(1, vec!["src/main.rs"])];
        let commit_files = vec!["src/main.rs".to_string()];

        let result = match_step1_simple(&candidates, &commit_files);

        assert!(result.is_some(), "File overlap should match");
        let r = result.unwrap();
        assert!(r.confidence >= 0.85, "Confidence should be at least 0.85");
        assert_eq!(r.step, 1, "Step should be 1");
    }

    #[test]
    fn test_no_overlap_returns_none() {
        let candidates = vec![mock_with_files(1, vec!["other.rs"])];
        let commit_files = vec!["src/main.rs".to_string()];

        let result = match_step1_simple(&candidates, &commit_files);

        assert!(result.is_none(), "No overlap should return None");
    }

    #[test]
    fn test_multiple_overlaps_higher_confidence() {
        let candidates = vec![mock_with_files(1, vec!["a.rs", "b.rs", "c.rs"])];
        let commit_files = vec!["a.rs".to_string(), "b.rs".to_string()];

        let result = match_step1_simple(&candidates, &commit_files).unwrap();

        assert!(
            result.confidence > 0.85,
            "Multiple overlaps should increase confidence"
        );
        assert_eq!(result.confidence, 0.95, "2 overlaps = 0.85 + 0.10");
    }

    #[test]
    fn test_confidence_capped_at_three_overlaps() {
        let candidates = vec![mock_with_files(
            1,
            vec!["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"],
        )];
        let commit_files = vec![
            "a.rs".to_string(),
            "b.rs".to_string(),
            "c.rs".to_string(),
            "d.rs".to_string(),
        ];

        let result = match_step1_simple(&candidates, &commit_files).unwrap();

        assert_eq!(
            result.confidence, 1.0,
            "Confidence capped at 0.85 + 0.15 = 1.0"
        );
    }

    #[test]
    fn test_first_matching_candidate_wins() {
        let candidates = vec![
            mock_with_files(1, vec!["unrelated.rs"]),
            mock_with_files(2, vec!["target.rs"]),
            mock_with_files(3, vec!["target.rs", "another.rs"]),
        ];
        let commit_files = vec!["target.rs".to_string()];

        let result = match_step1_simple(&candidates, &commit_files).unwrap();

        assert_eq!(
            result.conversation.id, 2,
            "First matching candidate should win"
        );
    }

    #[test]
    fn test_empty_commit_files_returns_none() {
        let candidates = vec![mock_with_files(1, vec!["a.rs"])];
        let commit_files: Vec<String> = vec![];

        let result = match_step1_simple(&candidates, &commit_files);

        assert!(result.is_none(), "Empty commit files should return None");
    }

    #[test]
    fn test_empty_candidates_returns_none() {
        let candidates: Vec<ConversationWithFiles> = vec![];
        let commit_files = vec!["a.rs".to_string()];

        let result = match_step1_simple(&candidates, &commit_files);

        assert!(result.is_none(), "Empty candidates should return None");
    }
}
