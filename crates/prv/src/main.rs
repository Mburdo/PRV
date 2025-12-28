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
                println!("\nSample extraction ({} blocks from recent message):", blocks.len());
                if let Some(block) = blocks.first() {
                    let preview: String = block.content.lines().take(3).collect::<Vec<_>>().join("\n");
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
}
