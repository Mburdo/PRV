use anyhow::Result;
use clap::{Parser, Subcommand};

#[cfg(test)]
use clap::CommandFactory;

#[derive(Parser)]
#[command(name = "prv")]
#[command(version, about = "Context tracing for code")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Link a commit to its originating AI session
    Link {
        /// Commit SHA, branch name, or HEAD
        #[arg(long, default_value = "HEAD")]
        commit: String,
    },

    /// Query linked session for a commit
    Query {
        /// Commit SHA, branch name, or HEAD
        #[arg(default_value = "HEAD")]
        commit: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Debug utilities
    Debug {
        #[command(subcommand)]
        debug_cmd: DebugCommands,
    },
}

#[derive(Subcommand)]
enum DebugCommands {
    /// Show CASS database status
    Cass,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Link { commit }) => {
            link_commit(&commit)?;
        }
        Some(Commands::Query { commit, json }) => {
            query_commit(&commit, json)?;
        }
        Some(Commands::Debug { debug_cmd }) => match debug_cmd {
            DebugCommands::Cass => {
                debug_cass()?;
            }
        },
        None => {
            println!("prv: context tracing for code");
            println!("Run with --help for usage");
        }
    }

    Ok(())
}

fn link_commit(commit_ref: &str) -> Result<()> {
    use prv_cass::{find_workspace_for_repo, CassDb};
    use prv_core::matcher::{
        candidate_sessions, match_step0, match_step1_simple, match_step2, Conversation,
        ConversationStore, ConversationWithCode, ConversationWithFiles, Workspace,
    };
    use prv_core::{extract_code_blocks, Link, LinkIndex, LinkStorage};

    // Open the git repository
    let repo = git2::Repository::open_from_env()
        .map_err(|e| anyhow::anyhow!("Not in a git repository: {}", e))?;

    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("Repository has no working directory (bare repo?)"))?;

    // Resolve the commit reference to a full SHA
    let commit_sha = resolve_commit(&repo, commit_ref)?;
    let short_sha = &commit_sha[..7.min(commit_sha.len())];

    // Check if already linked
    let storage = LinkStorage::new(workdir);
    if storage.exists(&commit_sha) {
        println!("Commit {} is already linked.", short_sha);
        println!("Run `prv query {}` to see the link.", short_sha);
        return Ok(());
    }

    // Open CASS database
    let db = CassDb::open()
        .map_err(|e| anyhow::anyhow!("CASS not available: {}. Is CASS installed?", e))?;

    // Get commit info
    let commit = repo
        .find_commit(git2::Oid::from_str(&commit_sha)?)
        .map_err(|e| anyhow::anyhow!("Cannot find commit: {}", e))?;
    let commit_time_ms = commit.time().seconds() * 1000;

    // Create a store wrapper for the matcher
    struct CassStore<'a> {
        db: &'a CassDb,
    }

    impl ConversationStore for CassStore<'_> {
        fn find_workspace_for_path(
            &self,
            repo_path: &std::path::Path,
        ) -> Result<Option<Workspace>> {
            match find_workspace_for_repo(self.db, repo_path)? {
                Some(ws) => Ok(Some(Workspace {
                    id: ws.id,
                    path: ws.path,
                })),
                None => Ok(None),
            }
        }

        fn conversations_for_workspace(&self, workspace_id: i64) -> Result<Vec<Conversation>> {
            let cass_convs = self.db.conversations_for_workspace(workspace_id)?;

            // Convert prv_cass::Conversation to prv_core::matcher::Conversation
            // Filter out conversations without workspace_id or started_at
            Ok(cass_convs
                .into_iter()
                .filter_map(|c| {
                    let ws_id = c.workspace_id?;
                    let started = c.started_at?;
                    Some(Conversation {
                        id: c.id,
                        workspace_id: ws_id,
                        started_at: started,
                        ended_at: c.ended_at,
                    })
                })
                .collect())
        }
    }

    let store = CassStore { db: &db };

    // Get candidate sessions within time window
    let candidates = candidate_sessions(&store, workdir, commit_time_ms)?;

    if candidates.is_empty() {
        println!("No matching session found for {}.", short_sha);
        println!("Possible reasons:");
        println!("  - No CASS workspace matches this repository");
        println!("  - No sessions within 7-day window of commit time");
        return Ok(());
    }

    // Get commit files for step1 matching
    let commit_files = get_commit_files(&repo, &commit)?;

    // Get diff lines for step2 matching
    let diff_lines = get_commit_diff_lines(&repo, &commit).unwrap_or_default();

    // Run matching pipeline: Step 0 → Step 1 → Step 2
    let result = match_step0(&candidates)
        .or_else(|| {
            // Step 1: file path hints from message content
            if commit_files.is_empty() {
                return None;
            }

            // Extract file paths mentioned in messages for each candidate
            let candidates_with_files: Vec<ConversationWithFiles> = candidates
                .iter()
                .filter_map(|c| {
                    let messages = db.messages_for_conversation(c.id).ok()?;
                    let mentioned_files: Vec<String> = messages
                        .iter()
                        .flat_map(|m| extract_file_paths(&m.content))
                        .collect();
                    Some(ConversationWithFiles {
                        conversation: c.clone(),
                        mentioned_files,
                    })
                })
                .collect();

            match_step1_simple(&candidates_with_files, &commit_files)
        })
        .or_else(|| {
            // Step 2: line hash matching from code blocks in messages
            if diff_lines.is_empty() {
                return None;
            }

            // Extract code blocks from messages for each candidate
            let candidates_with_code: Vec<ConversationWithCode> = candidates
                .iter()
                .filter_map(|c| {
                    let messages = db.messages_for_conversation(c.id).ok()?;
                    let code_lines: Vec<String> = messages
                        .iter()
                        .filter(|m| m.role == "assistant") // Only assistant messages have code
                        .flat_map(|m| {
                            extract_code_blocks(&m.content)
                                .into_iter()
                                .flat_map(|block| {
                                    block.content.lines().map(String::from).collect::<Vec<_>>()
                                })
                        })
                        .collect();
                    Some(ConversationWithCode {
                        conversation: c.clone(),
                        code_lines,
                    })
                })
                .collect();

            match_step2(&candidates_with_code, &diff_lines)
        });

    match result {
        Some(m) => {
            let link = Link::new(&commit_sha, m.conversation.id, m.confidence, m.step);
            storage.save(&link)?;

            // Update index
            let mut index = LinkIndex::load(workdir)?;
            index.insert(&link);
            index.save(workdir)?;

            println!(
                "Linked {} → session {} ({:.0}% confidence, step {})",
                short_sha,
                m.conversation.id,
                m.confidence * 100.0,
                m.step
            );
        }
        None => {
            println!("No matching session found for {}.", short_sha);
            println!(
                "Found {} candidate(s) but no confident match.",
                candidates.len()
            );
        }
    }

    Ok(())
}

/// Extract file paths mentioned in text (e.g., "src/main.rs", "crates/prv/Cargo.toml")
fn extract_file_paths(text: &str) -> Vec<String> {
    use regex::Regex;
    use std::collections::HashSet;

    // Match common file path patterns
    let re = Regex::new(r#"(?:^|[\s`"'(])([a-zA-Z0-9_\-./]+\.[a-zA-Z0-9]+)(?:[\s`"'):]|$)"#)
        .expect("Invalid regex");

    let mut paths: HashSet<String> = HashSet::new();

    for cap in re.captures_iter(text) {
        if let Some(m) = cap.get(1) {
            let path = m.as_str();
            // Filter for likely file paths
            if path.contains('/')
                || path.ends_with(".rs")
                || path.ends_with(".toml")
                || path.ends_with(".json")
                || path.ends_with(".md")
                || path.ends_with(".ts")
                || path.ends_with(".js")
                || path.ends_with(".py")
            {
                paths.insert(path.to_string());
            }
        }
    }

    paths.into_iter().collect()
}

/// Get list of files changed in a commit
fn get_commit_files(repo: &git2::Repository, commit: &git2::Commit) -> Result<Vec<String>> {
    let tree = commit.tree()?;
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

    let mut files = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            if let Some(path) = delta.new_file().path() {
                if let Some(s) = path.to_str() {
                    files.push(s.to_string());
                }
            }
            true
        },
        None,
        None,
        None,
    )?;

    Ok(files)
}

fn get_commit_diff_lines(repo: &git2::Repository, commit: &git2::Commit) -> Result<Vec<String>> {
    let tree = commit.tree()?;
    let parent_tree = commit.parent(0).ok().and_then(|p| p.tree().ok());

    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;

    let mut lines = Vec::new();
    diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
        if line.origin() == '+' || line.origin() == '-' {
            if let Ok(content) = std::str::from_utf8(line.content()) {
                lines.push(content.trim_end().to_string());
            }
        }
        true
    })?;

    Ok(lines)
}

fn query_commit(commit_ref: &str, json_output: bool) -> Result<()> {
    use prv_core::LinkStorage;

    // Open the git repository
    let repo = git2::Repository::open_from_env()
        .map_err(|e| anyhow::anyhow!("Not in a git repository: {}", e))?;

    let workdir = repo
        .workdir()
        .ok_or_else(|| anyhow::anyhow!("Repository has no working directory (bare repo?)"))?;

    // Resolve the commit reference to a full SHA
    let commit_sha = resolve_commit(&repo, commit_ref)?;

    // Load link from storage
    let storage = LinkStorage::new(workdir);

    match storage.load(&commit_sha)? {
        Some(link) => {
            if json_output {
                println!("{}", serde_json::to_string_pretty(&link)?);
            } else {
                println!("Commit:     {}", &commit_sha[..7.min(commit_sha.len())]);
                println!("Session:    {}", link.session_id);
                println!("Confidence: {:.0}%", link.confidence * 100.0);
                println!("Match step: {}", link.match_step);
                println!(
                    "Linked at:  {}",
                    link.created_at.format("%Y-%m-%d %H:%M:%S UTC")
                );
            }
        }
        None => {
            let short_sha = &commit_sha[..7.min(commit_sha.len())];
            println!("No link found for {}.", short_sha);
            println!("Run `prv link --commit {}` to create one.", short_sha);
        }
    }

    Ok(())
}

fn resolve_commit(repo: &git2::Repository, commit_ref: &str) -> Result<String> {
    // Try to resolve as a reference (HEAD, branch name, etc.)
    let obj = repo
        .revparse_single(commit_ref)
        .map_err(|e| anyhow::anyhow!("Cannot resolve '{}': {}", commit_ref, e))?;

    let commit = obj
        .peel_to_commit()
        .map_err(|e| anyhow::anyhow!("'{}' is not a commit: {}", commit_ref, e))?;

    Ok(commit.id().to_string())
}

fn debug_cass() -> Result<()> {
    use prv_cass::CassDb;
    use prv_core::extract_code_blocks;

    match CassDb::open() {
        Ok(db) => {
            let count = db.session_count()?;
            let workspaces = db.workspaces()?;

            println!("CASS Database Status");
            println!("====================");
            println!("Location: ~/Library/Application Support/com.coding-agent-search.coding-agent-search/agent_search.db");
            println!("Status: Connected");
            println!("Workspaces: {}", workspaces.len());
            println!("Sessions: {}", count);

            // Sample code block extraction from recent message
            if let Ok(Some(msg)) = db.recent_message() {
                let blocks = extract_code_blocks(&msg.content);
                println!(
                    "\nSample extraction ({} blocks from recent message):",
                    blocks.len()
                );
                if let Some(block) = blocks.first() {
                    let preview: String =
                        block.content.lines().take(3).collect::<Vec<_>>().join("\n");
                    println!("  Language: {:?}", block.language);
                    println!("  Preview:\n    {}", preview.replace('\n', "\n    "));
                }
            } else {
                println!("\nNo recent messages found for sample extraction.");
            }
        }
        Err(e) => {
            eprintln!("CASS not available: {}", e);
            eprintln!("\nIs CASS installed? See: https://github.com/mburdo/cass");
            std::process::exit(1);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parses() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_query_parses_with_defaults() {
        let cli = Cli::parse_from(["prv", "query"]);
        match cli.command {
            Some(Commands::Query { commit, json }) => {
                assert_eq!(commit, "HEAD");
                assert!(!json);
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_query_parses_with_commit() {
        let cli = Cli::parse_from(["prv", "query", "abc123"]);
        match cli.command {
            Some(Commands::Query { commit, json }) => {
                assert_eq!(commit, "abc123");
                assert!(!json);
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_query_parses_with_json_flag() {
        let cli = Cli::parse_from(["prv", "query", "HEAD", "--json"]);
        match cli.command {
            Some(Commands::Query { commit, json }) => {
                assert_eq!(commit, "HEAD");
                assert!(json);
            }
            _ => panic!("Expected Query command"),
        }
    }

    #[test]
    fn test_link_parses_with_defaults() {
        let cli = Cli::parse_from(["prv", "link"]);
        match cli.command {
            Some(Commands::Link { commit }) => {
                assert_eq!(commit, "HEAD");
            }
            _ => panic!("Expected Link command"),
        }
    }

    #[test]
    fn test_link_parses_with_commit() {
        let cli = Cli::parse_from(["prv", "link", "--commit", "abc123"]);
        match cli.command {
            Some(Commands::Link { commit }) => {
                assert_eq!(commit, "abc123");
            }
            _ => panic!("Expected Link command"),
        }
    }

    #[test]
    #[ignore] // Integration test - requires CASS
    fn test_debug_cass_integration() {
        use std::process::Command;

        let output = Command::new("cargo")
            .args(["run", "--", "debug", "cass"])
            .output()
            .expect("Failed to run prv");

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Should either connect or show clear error
        assert!(
            stdout.contains("Connected") || stderr.contains("CASS not available"),
            "stdout: {}\nstderr: {}",
            stdout,
            stderr
        );
    }

    #[test]
    #[ignore] // Integration test - requires git repo
    fn test_query_no_link_shows_helpful_message() {
        use std::process::Command;

        let output = Command::new("cargo")
            .args(["run", "--", "query", "HEAD"])
            .output()
            .expect("Failed to run prv");

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Should show helpful message about running prv link
        assert!(
            stdout.contains("No link found") || stdout.contains("prv link"),
            "stdout: {}",
            stdout
        );
    }
}
