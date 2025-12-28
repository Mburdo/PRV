# Phase 0 Understanding

## Goal
Establish the build system, CASS integration, and basic data flow.

## Components

### 1. Rust Project Structure
- Cargo workspace with multiple crates
- `prv` binary crate (CLI)
- `prv-core` library crate (shared logic)
- `prv-cass` library crate (CASS integration)

### 2. CASS SQLite Reader
- Connect to CASS database at known path
- Read workspaces table (path mapping)
- Read conversations table (sessions)
- Read messages table (content with code blocks)

### 3. Code Block Extraction
- Parse triple-backtick blocks from message content
- Handle language tags (```rust, ```python, etc.)
- Handle 4-space indented blocks
- Extract from unified diff format
- Return structured code blocks with metadata

### 4. CLI Skeleton
- `prv --version` shows version
- `prv --help` shows help
- `prv debug cass` shows CASS connection status and session count

## Files to Create

```
prv/
├── Cargo.toml              (workspace)
├── Cargo.lock
├── crates/
│   ├── prv/                (binary)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── main.rs
│   ├── prv-core/           (library)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── code_block.rs
│   └── prv-cass/           (library)
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── db.rs
│           ├── models.rs
│           └── workspace.rs
└── tests/
    └── code_block_test.rs
```

## Dependencies (External)

- `rusqlite` - SQLite access
- `clap` - CLI argument parsing
- `anyhow` - Error handling
- `regex` - Code block parsing

## Dependencies (Internal)

- prv-core depends on nothing
- prv-cass depends on prv-core
- prv (binary) depends on prv-core, prv-cass

## Exit Criteria

1. `cargo build` produces `prv` binary
2. `prv debug cass` shows session count from CASS
3. Unit tests pass for code block parser
4. Can extract code blocks from sample CASS messages

## Ambiguities

None - Phase 0 is well-defined.
