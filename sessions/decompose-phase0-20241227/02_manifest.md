# Phase 0 Content Manifest

## Components

### M1: Cargo Workspace Setup
- [ ] Root `Cargo.toml` with workspace members
- [ ] `.gitignore` for Rust
- [ ] `crates/` directory structure

### M2: prv-core Crate
- [ ] `crates/prv-core/Cargo.toml`
- [ ] `crates/prv-core/src/lib.rs` - exports
- [ ] `crates/prv-core/src/code_block.rs` - CodeBlock struct and parser

### M3: Code Block Parser
- [ ] `CodeBlock` struct: content, language, source_line
- [ ] `extract_code_blocks(text: &str) -> Vec<CodeBlock>`
- [ ] Parse triple-backtick with language tag
- [ ] Parse 4-space indented blocks
- [ ] Parse unified diff `+` lines
- [ ] Handle malformed/nested blocks gracefully

### M4: prv-cass Crate
- [ ] `crates/prv-cass/Cargo.toml`
- [ ] `crates/prv-cass/src/lib.rs` - exports
- [ ] `crates/prv-cass/src/db.rs` - CassDb connection
- [ ] `crates/prv-cass/src/models.rs` - Workspace, Conversation, Message structs
- [ ] `crates/prv-cass/src/workspace.rs` - workspace-repo mapping

### M5: CASS Database Connection
- [ ] `CassDb::open()` - find and open CASS SQLite
- [ ] `CassDb::workspaces()` - list all workspaces
- [ ] `CassDb::conversations(workspace_id)` - list sessions
- [ ] `CassDb::messages(conversation_id)` - list messages
- [ ] Handle missing database gracefully

### M6: Workspace-Repo Mapping
- [ ] `find_workspace_for_repo(repo_path: &Path) -> Option<Workspace>`
- [ ] Handle trailing slashes, symlinks
- [ ] Case-insensitive on macOS
- [ ] Handle nested git repos

### M7: prv Binary Crate
- [ ] `crates/prv/Cargo.toml`
- [ ] `crates/prv/src/main.rs` - CLI entry point
- [ ] clap command structure

### M8: CLI Commands
- [ ] `prv --version` - show version
- [ ] `prv --help` - show help
- [ ] `prv debug cass` - show CASS status and session count

### M9: Tests
- [ ] `tests/code_block_test.rs` - unit tests for parser
- [ ] Test: triple-backtick extraction
- [ ] Test: language tag extraction
- [ ] Test: 4-space indent extraction
- [ ] Test: diff format extraction
- [ ] Test: malformed block handling
- [ ] Test: nested block handling

---

## Verification Checklist

- [ ] `cargo build` succeeds
- [ ] `cargo test` passes
- [ ] `prv --version` outputs version
- [ ] `prv debug cass` connects to CASS and shows count
- [ ] Code blocks extracted correctly from sample messages
