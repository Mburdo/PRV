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
