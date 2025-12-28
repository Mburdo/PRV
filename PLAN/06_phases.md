# Implementation Phases — PRV

## Overview

PRV v1.0 is broken into 4 phases, each producing a shippable artifact. Phases build on each other but each delivers standalone value.

```
Phase 0 (Foundation) → Phase 1 (Core) → Phase 2 (Differentiators) → Phase 3 (Polish)
```

---

## Phase 0: Foundation

**Goal:** Establish the build system, CASS integration, and basic data flow.

**Deliverables:**
- Rust project structure with cargo workspace
- CASS SQLite reader library
- Code block extraction from messages
- Basic CLI skeleton (`prv --version`, `prv --help`)

**Requirements Addressed:**
| REQ | Scope |
|-----|-------|
| REQ-003 | Code block extraction (full) |
| REQ-004 | Workspace-repo mapping (full) |

**Exit Criteria:**
- [ ] `cargo build` produces `prv` binary
- [ ] `prv debug cass` shows session count from CASS database
- [ ] Unit tests pass for code block parser
- [ ] Can extract code blocks from sample CASS messages

**Risks Validated:**
- CASS schema compatibility confirmed
- Code block parsing accuracy measured

---

## Phase 1: Core Linking

**Goal:** Implement commit-to-session linking with fingerprinting. This is the core value proposition.

**Deliverables:**
- Fingerprint index builder
- Commit → session matching algorithm
- Link storage in `.prv/links/`
- CLI: `prv link`, `prv query <commit>`

**Requirements Addressed:**
| REQ | Scope |
|-----|-------|
| REQ-002 | Commit-session linking (full) |
| REQ-005 | CLI query (partial: `prv query`) |
| REQ-007 | Git hook linking (full) |
| REQ-008 | Index for fast lookup (full) |

**Exit Criteria:**
- [ ] `prv link --commit HEAD` creates link file
- [ ] `prv query <sha>` returns matching session(s) with confidence
- [ ] Manual audit of 50 commits shows >80% accuracy
- [ ] Index rebuild < 30s for repo with 1000 commits

**Depends On:** Phase 0

**Spikes Required:**
- SPIKE-002: Fingerprinting strategy validation

---

## Phase 2: Differentiators

**Goal:** Implement the features that make PRV unique: Roads Not Taken and Heat Map.

**Deliverables:**
- LLM summarization integration (local + API)
- Enhanced summary generation with alternatives extraction
- Heat map CLI: `prv heatmap <file>`
- LSP server with hover support

**Requirements Addressed:**
| REQ | Scope |
|-----|-------|
| REQ-001 | LSP hover (full) |
| REQ-006 | Session context display (full) |
| REQ-010 | Roads not taken extraction (full) |
| REQ-011 | Heat map (full) |
| REQ-012 | Enhanced summary generation (full) |

**Exit Criteria:**
- [ ] `prv summarize <session>` generates JSON summary < 2KB
- [ ] Summary includes `alternatives[]` when detected
- [ ] `prv heatmap src/main.rs` shows colored output
- [ ] LSP hover returns context in < 500ms (p95)
- [ ] Works in VS Code and Neovim

**Depends On:** Phase 1

**Spikes Required:**
- SPIKE-003: LSP performance baseline
- SPIKE-004: LLM summary extraction
- SPIKE-005: Heat map LSP implementation

---

## Phase 3: Sharing & Polish

**Goal:** Enable team sharing via prv-memory and polish the user experience.

**Deliverables:**
- prv-memory orphan branch sync
- `prv sync push`, `prv sync pull`
- `prv install` for git hooks
- `prv blame <file>:<line>` CLI
- Error messages and edge case handling

**Requirements Addressed:**
| REQ | Scope |
|-----|-------|
| REQ-005 | CLI query (complete: `prv blame`) |
| REQ-009 | Shareable context (full) |

**Exit Criteria:**
- [ ] `prv sync push` pushes to `prv-memory` branch
- [ ] `prv sync pull` fetches and merges summaries
- [ ] Confirmation prompt before push (security)
- [ ] `prv install` sets up post-commit hook
- [ ] All CLI commands have `--help` and `--json`
- [ ] Graceful errors for missing CASS, missing git, etc.

**Depends On:** Phase 2

---

## Phase Summary

| Phase | Focus | Key Deliverable | REQs |
|-------|-------|-----------------|------|
| 0 | Foundation | CASS reader + parser | REQ-003, REQ-004 |
| 1 | Core | Commit-session linking | REQ-002, REQ-007, REQ-008 |
| 2 | Differentiators | LSP + Heat Map + Summaries | REQ-001, REQ-010, REQ-011, REQ-012 |
| 3 | Polish | prv-memory sync + CLI | REQ-005, REQ-009 |

---

## Milestone Definitions

### M0: "It Reads" (Phase 0 Complete)
- PRV can read CASS database
- PRV can extract code blocks
- Basic CLI works

### M1: "It Links" (Phase 1 Complete)
- PRV can link commits to sessions
- Accuracy >80% on AI-generated commits
- Links persist in `.prv/`

### M2: "It Explains" (Phase 2 Complete)
- LSP hover shows context + alternatives
- Heat map visualizes provenance
- Summaries generated with "roads not taken"

### M3: "It Shares" (Phase 3 Complete = v1.0)
- Teams can sync context via git
- Full CLI experience
- Production-ready error handling

---

## v1.1 and Beyond

**v1.1: Evolution**
- REQ-013: Evolution graph
- Cross-session intelligence
- Improved fingerprinting for rebases

**v2.0: Intelligence**
- Provenance-aware AI assistance
- GitHub/GitLab PR integration
- Time travel understanding

---

## Phase Dependencies Graph

```
                    ┌─────────────────┐
                    │    Phase 0      │
                    │   Foundation    │
                    │ (CASS + Parser) │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │    Phase 1      │
                    │  Core Linking   │
                    │ (Fingerprint)   │
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
              ▼              ▼              ▼
        ┌──────────┐  ┌──────────┐  ┌──────────┐
        │ SPIKE-03 │  │ SPIKE-04 │  │ SPIKE-05 │
        │LSP Perf  │  │LLM Extract│  │Heat Map │
        └────┬─────┘  └────┬─────┘  └────┬─────┘
              │              │              │
              └──────────────┼──────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │    Phase 2      │
                    │ Differentiators │
                    │(LSP+Heat+Roads) │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │    Phase 3      │
                    │  Sharing/Polish │
                    │ (prv-memory)    │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │     v1.0        │
                    │    Release      │
                    └─────────────────┘
```
