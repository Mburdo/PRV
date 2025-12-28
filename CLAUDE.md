# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PRV is a Git-native "memory layer" for AI-assisted development, implemented in **Rust**. It traces any line of code back to its origin — the AI session that created it, the reasoning behind it, and alternatives that were rejected.

**Core concept:** Git tells you *what* changed; PRV tells you *why*, what else was considered, and how understanding evolved.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    AI Tools (logging)                       │
│  Claude Code │ Cursor │ Codex │ Aider │ ChatGPT │ ...      │
└─────────────────────────┬───────────────────────────────────┘
                          │ (automatic via CASS)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                         CASS                                │
│  Discovery → Normalization → SQLite                         │
└─────────────────────────┬───────────────────────────────────┘
                          │ (PRV reads SQLite)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                         PRV                                 │
│  Ingest → Fingerprint → Link → Summarize → Surface         │
│                                                             │
│  Primary: LSP server (hover in any editor)                  │
│  Secondary: CLI (prv blame, prv query, prv heatmap)         │
│  Transport: prv-memory orphan branch                        │
└─────────────────────────────────────────────────────────────┘
```

PRV reads session data from CASS SQLite (no capture code in PRV itself). It links commits to sessions using hunk fingerprinting, generates AI-enhanced summaries with rejected alternatives, and surfaces context via LSP hover or CLI.

## Key Concepts

- **Draft/Link/Index** — Terminology from earlier design; now superseded by CASS integration
- **prv-memory** — Orphan branch for sharing enhanced summaries (not raw transcripts)
- **Hunk fingerprinting** — Hash normalized code hunks to match commits ↔ sessions
- **Roads Not Taken** — Extract rejected alternatives from sessions (differentiating feature)
- **Heat Map** — Visual overlay showing provenance coverage per file

## Build & Test Commands

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test --verbose
cargo test <test_name>        # Single test

# Run
cargo run -- <args>           # e.g., cargo run -- blame src/main.rs:42

# Lint
cargo clippy
cargo fmt --check
```

## Key Decisions (ADRs)

| ADR | Decision |
|-----|----------|
| ADR-006 | Read from CASS SQLite, not own capture |
| ADR-007 | Rust for performance (<500ms hover) |
| ADR-008 | LSP server as primary interface |
| ADR-009 | Enhanced AI summaries for sharing (not raw transcripts) |
| ADR-010 | Heat map visualization for provenance coverage |

See `PLAN/03_decisions.md` for full ADR details.

## Storage Layout (planned)

```
.prv/
├── links/           # Link JSON files (commit → session)
├── summaries/       # Enhanced summaries with alternatives
├── index/           # Fingerprint hash → session lookup
└── pending/         # Queued summaries awaiting explicit push
```

## Performance Requirements

- LSP hover: <500ms for 95th percentile
- Commit linking: 80%+ accuracy on AI-generated commits
- Summaries: <2KB per session

## What NOT To Do

- Do NOT auto-share anything (queue-then-push model)
- Do NOT block git operations (hooks must soft-fail)
- Do NOT modify CASS data (read-only integration)
- Do NOT include raw transcripts in prv-memory (summaries only)

## Reference Documentation

- `PLAN/00_north_star.md` — Product vision and differentiators
- `PLAN/01_requirements.md` — Requirements with acceptance criteria
- `PLAN/03_decisions.md` — Architecture Decision Records
- `README.md` — User-facing product description
- `AGENTS.md` — Workflow instructions, tooling (bd, bv, cass, etc.)
