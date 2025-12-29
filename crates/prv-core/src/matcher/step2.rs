//! Step 2 matching: line hash overlap
//!
//! When Steps 0 and 1 return None, compare normalized line hashes
//! between commit diff and session code blocks. Best overlap wins.

use std::collections::HashSet;

use fxhash::hash64;

use super::step0::MatchResult;
use super::Conversation;

/// Trait for conversations that can provide code block lines
pub trait CodeBlockLines {
    fn code_block_lines(&self) -> Vec<String>;
}

/// Extended conversation with code block content for Step 2 matching
#[derive(Debug, Clone)]
pub struct ConversationWithCode {
    pub conversation: Conversation,
    pub code_lines: Vec<String>,
}

impl CodeBlockLines for ConversationWithCode {
    fn code_block_lines(&self) -> Vec<String> {
        self.code_lines.clone()
    }
}

/// Normalize a line for hashing: collapse whitespace
fn hash_normalized(line: &str) -> u64 {
    let normalized: String = line.split_whitespace().collect::<Vec<_>>().join(" ");
    hash64(&normalized)
}

/// Step 2: Disambiguate by normalized line hash overlap.
///
/// Compares commit diff lines against session code blocks.
/// Returns best match if overlap ratio > 50%.
/// Confidence = 0.8 + (overlap_ratio * 0.2), so max is 1.0.
pub fn match_step2(
    candidates: &[ConversationWithCode],
    diff_lines: &[String],
) -> Option<MatchResult> {
    if diff_lines.is_empty() || candidates.is_empty() {
        return None;
    }

    let diff_hashes: HashSet<u64> = diff_lines.iter().map(|l| hash_normalized(l)).collect();

    let mut best: Option<(Conversation, f64)> = None;

    for candidate in candidates {
        let session_hashes: HashSet<u64> = candidate
            .code_block_lines()
            .iter()
            .map(|l| hash_normalized(l))
            .collect();

        let intersection = diff_hashes.intersection(&session_hashes).count();
        let overlap_ratio = intersection as f64 / diff_hashes.len().max(1) as f64;

        if overlap_ratio > 0.5 {
            let confidence = 0.8 + (overlap_ratio * 0.2);
            if best.as_ref().is_none_or(|(_, c)| confidence > *c) {
                best = Some((candidate.conversation.clone(), confidence));
            }
        }
    }

    best.map(|(conversation, confidence)| MatchResult {
        conversation,
        confidence,
        step: 2,
    })
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

    fn mock_with_code(id: i64, lines: Vec<&str>) -> ConversationWithCode {
        ConversationWithCode {
            conversation: mock_conversation(id),
            code_lines: lines.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_hash_normalized_strips_whitespace() {
        assert_eq!(
            hash_normalized("  let x = 1;  "),
            hash_normalized("let x = 1;")
        );
    }

    #[test]
    fn test_hash_normalized_collapses_internal_whitespace() {
        assert_eq!(
            hash_normalized("let   x   =   1;"),
            hash_normalized("let x = 1;")
        );
    }

    #[test]
    fn test_high_overlap_matches() {
        // Session has 4 of 5 diff lines (80% overlap)
        let candidates = vec![mock_with_code(
            1,
            vec!["line1", "line2", "line3", "line4", "other"],
        )];
        let diff_lines: Vec<String> = vec!["line1", "line2", "line3", "line4", "line5"]
            .into_iter()
            .map(String::from)
            .collect();

        let result = match_step2(&candidates, &diff_lines);

        assert!(result.is_some(), "80% overlap should match");
        let r = result.unwrap();
        assert!(
            r.confidence > 0.9,
            "High overlap should have high confidence"
        );
        assert_eq!(r.step, 2);
    }

    #[test]
    fn test_low_overlap_no_match() {
        // Session has 1 of 5 diff lines (20% overlap)
        let candidates = vec![mock_with_code(1, vec!["line1", "other1", "other2"])];
        let diff_lines: Vec<String> = vec!["line1", "line2", "line3", "line4", "line5"]
            .into_iter()
            .map(String::from)
            .collect();

        let result = match_step2(&candidates, &diff_lines);

        assert!(result.is_none(), "20% overlap should not match");
    }

    #[test]
    fn test_exactly_50_percent_no_match() {
        // Session has 2 of 4 diff lines (exactly 50% overlap)
        let candidates = vec![mock_with_code(1, vec!["line1", "line2"])];
        let diff_lines: Vec<String> = vec!["line1", "line2", "line3", "line4"]
            .into_iter()
            .map(String::from)
            .collect();

        let result = match_step2(&candidates, &diff_lines);

        assert!(result.is_none(), "Exactly 50% should not match (need >50%)");
    }

    #[test]
    fn test_best_match_selected() {
        // Multiple candidates with different overlaps
        let candidates = vec![
            mock_with_code(1, vec!["line1", "line2"]), // 40% overlap
            mock_with_code(2, vec!["line1", "line2", "line3", "line4"]), // 80% overlap
            mock_with_code(3, vec!["line1", "line2", "line3"]), // 60% overlap
        ];
        let diff_lines: Vec<String> = vec!["line1", "line2", "line3", "line4", "line5"]
            .into_iter()
            .map(String::from)
            .collect();

        let result = match_step2(&candidates, &diff_lines).unwrap();

        assert_eq!(result.conversation.id, 2, "Highest overlap should win");
    }

    #[test]
    fn test_empty_diff_returns_none() {
        let candidates = vec![mock_with_code(1, vec!["line1"])];
        let diff_lines: Vec<String> = vec![];

        let result = match_step2(&candidates, &diff_lines);

        assert!(result.is_none(), "Empty diff should return None");
    }

    #[test]
    fn test_empty_candidates_returns_none() {
        let candidates: Vec<ConversationWithCode> = vec![];
        let diff_lines: Vec<String> = vec!["line1".to_string()];

        let result = match_step2(&candidates, &diff_lines);

        assert!(result.is_none(), "Empty candidates should return None");
    }

    #[test]
    fn test_confidence_calculation() {
        // 100% overlap should give confidence = 0.8 + 0.2 = 1.0
        let candidates = vec![mock_with_code(1, vec!["a", "b", "c"])];
        let diff_lines: Vec<String> = vec!["a", "b", "c"].into_iter().map(String::from).collect();

        let result = match_step2(&candidates, &diff_lines).unwrap();

        assert!(
            (result.confidence - 1.0).abs() < 0.001,
            "100% overlap should give confidence 1.0"
        );
    }

    #[test]
    fn test_whitespace_normalization_in_matching() {
        // Lines with different leading/trailing/internal whitespace should match
        let candidates = vec![mock_with_code(
            1,
            vec!["  let x = 1;  ", "   let  y  =  2;   "],
        )];
        let diff_lines: Vec<String> = vec!["let x = 1;".to_string(), "let y = 2;".to_string()];

        let result = match_step2(&candidates, &diff_lines);

        assert!(result.is_some(), "Whitespace-normalized lines should match");
    }
}
