//! Code block types for PRV
//!
//! Represents extracted code blocks from markdown and AI sessions.

use regex::Regex;

/// A code block extracted from text
#[derive(Debug, Clone, PartialEq)]
pub struct CodeBlock {
    /// The actual code content
    pub content: String,
    /// The programming language (e.g., "rust", "python")
    pub language: Option<String>,
    /// The line number where this block started in the source
    pub source_line: usize,
}

impl CodeBlock {
    /// Create a new code block
    pub fn new(content: String, language: Option<String>, source_line: usize) -> Self {
        Self {
            content,
            language,
            source_line,
        }
    }
}

/// Extract all code blocks from markdown-style text
///
/// Supports:
/// - Triple backtick blocks (```lang\ncode\n```)
/// - 4-space indented blocks (consecutive lines starting with 4 spaces)
/// - Diff format (+ lines from unified diff)
///
/// # Arguments
/// * `text` - The markdown text to parse
///
/// # Returns
/// A vector of CodeBlock structs
pub fn extract_code_blocks(text: &str) -> Vec<CodeBlock> {
    let mut blocks = Vec::new();

    // 1. Triple backtick blocks: ```lang\ncode\n```
    let backtick_re = Regex::new(r"```(\w*)\n([\s\S]*?)```").unwrap();

    for cap in backtick_re.captures_iter(text) {
        let match_start = cap.get(0).unwrap().start();
        let source_line = text[..match_start].matches('\n').count() + 1;

        let lang = cap
            .get(1)
            .map(|m| m.as_str().to_string())
            .filter(|s| !s.is_empty());
        let content = cap
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();

        blocks.push(CodeBlock::new(content, lang, source_line));
    }

    // 2. 4-space indented blocks (consecutive lines)
    let lines: Vec<&str> = text.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("    ") && !lines[i].trim().is_empty() {
            let start_line = i + 1; // 1-indexed
            let mut block_lines = Vec::new();

            while i < lines.len() && (lines[i].starts_with("    ") || lines[i].trim().is_empty()) {
                // Strip the 4-space prefix for indented lines
                if lines[i].starts_with("    ") {
                    block_lines.push(&lines[i][4..]);
                } else {
                    block_lines.push(lines[i]);
                }
                i += 1;
            }

            // Trim trailing empty lines
            while block_lines
                .last()
                .map(|s| s.trim().is_empty())
                .unwrap_or(false)
            {
                block_lines.pop();
            }

            if !block_lines.is_empty() {
                let content = block_lines.join("\n") + "\n";
                blocks.push(CodeBlock::new(content, None, start_line));
            }
        } else {
            i += 1;
        }
    }

    // 3. Diff format (+ lines from unified diff)
    let diff_re = Regex::new(r"(?m)^@@[^@]+@@\n((?:[+\- ].*\n?)+)").unwrap();

    for cap in diff_re.captures_iter(text) {
        let match_start = cap.get(0).unwrap().start();
        let source_line = text[..match_start].matches('\n').count() + 1;

        if let Some(diff_content) = cap.get(1) {
            // Extract only the + lines (added code)
            let added_lines: Vec<&str> = diff_content
                .as_str()
                .lines()
                .filter(|line| line.starts_with('+'))
                .map(|line| &line[1..]) // Strip the + prefix
                .collect();

            if !added_lines.is_empty() {
                let content = added_lines.join("\n") + "\n";
                blocks.push(CodeBlock::new(
                    content,
                    Some("diff".to_string()),
                    source_line,
                ));
            }
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================
    // CodeBlock struct tests
    // =========================================

    #[test]
    fn test_code_block_new() {
        let block = CodeBlock::new("fn main() {}".to_string(), Some("rust".to_string()), 10);
        assert_eq!(block.content, "fn main() {}");
        assert_eq!(block.language, Some("rust".to_string()));
        assert_eq!(block.source_line, 10);
    }

    #[test]
    fn test_code_block_no_language() {
        let block = CodeBlock::new("code".to_string(), None, 1);
        assert!(block.language.is_none());
    }

    #[test]
    fn test_code_block_equality() {
        let a = CodeBlock::new("x".to_string(), None, 1);
        let b = CodeBlock::new("x".to_string(), None, 1);
        assert_eq!(a, b);
    }

    // =========================================
    // Triple backtick tests
    // =========================================

    #[test]
    fn test_triple_backtick_basic() {
        let text = "```\nfn main() {}\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, "fn main() {}\n");
        assert!(blocks[0].language.is_none());
    }

    #[test]
    fn test_triple_backtick_with_language() {
        let text = "```rust\nlet x = 1;\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks[0].language, Some("rust".to_string()));
        assert_eq!(blocks[0].content, "let x = 1;\n");
    }

    #[test]
    fn test_multiple_blocks() {
        let text = "```python\nprint(1)\n```\ntext\n```js\nconsole.log(1)\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].language, Some("python".to_string()));
        assert_eq!(blocks[1].language, Some("js".to_string()));
    }

    #[test]
    fn test_nested_blocks() {
        // When someone pastes markdown containing code blocks inside a code block
        let text = "````\n```rust\nlet x = 1;\n```\n````";
        let blocks = extract_code_blocks(text);
        // Should handle gracefully - exact behavior depends on regex matching
        assert!(blocks.len() >= 1);
    }

    // =========================================
    // Edge cases and error handling
    // =========================================

    #[test]
    fn test_empty_input() {
        let blocks = extract_code_blocks("");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_no_code_blocks() {
        let blocks = extract_code_blocks("just plain text");
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_malformed_unclosed() {
        let text = "```rust\nlet x = 1;";
        let blocks = extract_code_blocks(text);
        // Should not crash, returns 0 (unclosed block not matched)
        assert!(blocks.is_empty());
    }

    // =========================================
    // 4-space indented code blocks
    // =========================================

    #[test]
    fn test_4space_indent() {
        let text =
            "Some text:\n\n    fn main() {\n        println!(\"hello\");\n    }\n\nMore text.";
        let blocks = extract_code_blocks(text);
        // Should find the indented block
        assert!(blocks.iter().any(|b| b.content.contains("fn main()")));
    }

    #[test]
    fn test_4space_indent_multiline() {
        let text = "    line1\n    line2\n    line3";
        let blocks = extract_code_blocks(text);
        assert!(!blocks.is_empty());
        let block = blocks.iter().find(|b| b.language.is_none()).unwrap();
        assert!(block.content.contains("line1"));
        assert!(block.content.contains("line2"));
        assert!(block.content.contains("line3"));
    }

    // =========================================
    // Diff format tests
    // =========================================

    #[test]
    fn test_diff_format() {
        let text = r#"@@ -1,3 +1,4 @@
 context line
+added line 1
+added line 2
-removed line
"#;
        let blocks = extract_code_blocks(text);
        // Should find the diff block with + lines
        let diff_block = blocks
            .iter()
            .find(|b| b.language == Some("diff".to_string()));
        assert!(diff_block.is_some());
        let content = &diff_block.unwrap().content;
        assert!(content.contains("added line 1"));
        assert!(content.contains("added line 2"));
        assert!(!content.contains("removed line")); // - lines excluded
    }

    #[test]
    fn test_diff_format_empty_additions() {
        let text = r#"@@ -1,3 +1,2 @@
 context line
-removed line
"#;
        let blocks = extract_code_blocks(text);
        // No + lines, so no diff block created
        let diff_blocks: Vec<_> = blocks
            .iter()
            .filter(|b| b.language == Some("diff".to_string()))
            .collect();
        assert!(diff_blocks.is_empty());
    }

    // =========================================
    // Source line tracking
    // =========================================

    #[test]
    fn test_source_line_tracking() {
        let text = "line 1\nline 2\n```rust\ncode\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 1);
        // Block starts on line 3 (after "line 1" and "line 2")
        assert_eq!(blocks[0].source_line, 3);
    }

    #[test]
    fn test_multiple_blocks_line_tracking() {
        let text = "```\nfirst\n```\n\n```\nsecond\n```";
        let blocks = extract_code_blocks(text);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].source_line, 1);
        assert_eq!(blocks[1].source_line, 5);
    }
}
