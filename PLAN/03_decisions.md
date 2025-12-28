# Decisions — PRV

---

## ADR-001: Hunk Fingerprinting for Commit↔Session Matching

**Status:** accepted
**Date:** 2024-12-27

### Context
PRV needs to correlate git commits with CASS sessions. Time windows alone are insufficient since multiple sessions may overlap with a commit, and developers may commit hours after a session ends.

### Options

| Option | Summary | Pros | Cons |
|:-------|:--------|:-----|:-----|
| A | Time window only | Simple | Low accuracy, many false positives |
| B | Full diff matching | High accuracy | Expensive, brittle to formatting |
| C | Hunk fingerprinting | Good accuracy, fast | Some edge cases (rebases) |

### Decision
Option C: Hunk fingerprinting

### Rationale
Hash normalized code hunks from commit diffs and match against code blocks extracted from session messages. Handles formatting variations while maintaining good accuracy. Aligns with <500ms performance requirement.

### Consequences
- Need code block parser for CASS messages
- Need normalization strategy (whitespace, comments)
- Need index for O(1) hash lookups

### Reversal Triggers
- Accuracy falls below 70% in real-world testing
- Performance exceeds 500ms despite optimizations

---

## ADR-002: Orphan Branch Transport (prv-memory)

**Status:** accepted
**Date:** 2024-12-27

### Context
PRV context needs to be shareable across team members without polluting main git history or exposing raw transcripts.

### Options

| Option | Summary | Pros | Cons |
|:-------|:--------|:-----|:-----|
| A | Separate git repo | Clean separation | Extra repo management |
| B | `.prv/` in main branch | Simple | Pollutes history, bloat |
| C | Orphan branch | Git-native, invisible to main | Slightly complex |

### Decision
Option C: Orphan branch (`prv-memory`)

### Rationale
Git-native transport that works with existing remotes. Invisible to normal git operations. Teams can choose to push/pull context independently.

### Consequences
- Need orphan branch management commands
- Users must explicitly sync (`prv sync push/pull`)
- Works with standard git remotes

### Reversal Triggers
- Teams consistently confused by orphan branches
- Major git hosting platforms break orphan support

---

## ADR-003: Pipes Default, PTY Opt-in (DEPRECATED)

**Status:** superseded by ADR-006
**Date:** 2024-12-27

### Context
Original design considered wrapping AI tools to capture sessions. This approach was superseded by CASS integration.

### Decision
Deprecated. See ADR-006 for current approach.

---

## ADR-004: Queue-then-Push Security Model

**Status:** accepted
**Date:** 2024-12-27

### Context
Enhanced summaries shared via prv-memory could contain sensitive information. Need explicit user control over what gets shared.

### Options

| Option | Summary | Pros | Cons |
|:-------|:--------|:-----|:-----|
| A | Auto-push everything | Frictionless | Security risk |
| B | Never share | Safe | No team value |
| C | Queue locally, explicit push | Safe, flexible | Extra step |

### Decision
Option C: Queue-then-push

### Rationale
Summaries queue locally in `.prv/pending/`. User explicitly reviews and pushes with `prv sync push`. Balances team utility with security.

### Consequences
- Users must run `prv sync push` to share
- Pending queue visible via `prv status`
- No accidental secret exposure

### Reversal Triggers
- Users consistently forget to push
- Friction kills adoption

---

## ADR-005: Indexed Lookup for O(1) Candidate Matching

**Status:** accepted
**Date:** 2024-12-27

### Context
LSP hover must return in <500ms. Linear scan through all sessions is too slow for large histories.

### Options

| Option | Summary | Pros | Cons |
|:-------|:--------|:-----|:-----|
| A | Linear scan | Simple | O(n) per lookup |
| B | Indexed by hash | O(1) lookup | Index maintenance |
| C | Hybrid (recent linear, old indexed) | Balanced | Complex |

### Decision
Option B: Full index

### Rationale
Pre-compute fingerprint hashes during ingest. Store in SQLite index. Lookup is hash comparison, not content scanning.

### Consequences
- `prv index` command to rebuild
- Index invalidation on CASS changes
- Fallback to slower path if index stale

### Reversal Triggers
- Index size becomes problematic (>1GB)
- Rebuild time exceeds tolerance

---

## ADR-006: CASS Integration via SQLite (Not Wrapping)

**Status:** accepted
**Date:** 2024-12-27

### Context
PRV needs session data from AI tools. Originally considered wrapping each tool; CASS already aggregates this data.

### Options

| Option | Summary | Pros | Cons |
|:-------|:--------|:-----|:-----|
| A | Wrap each AI tool | Full control | N integrations, invasive |
| B | Build own capture | Full control | Duplicate CASS effort |
| C | Read CASS SQLite | Zero capture work, invisible | CASS dependency |

### Decision
Option C: CASS integration via SQLite

### Rationale
CASS already captures sessions from Claude Code, Cursor, Codex, etc. PRV reads this data directly. Zero workflow change for users. Invisible capture.

### Consequences
- CASS is required dependency
- PRV follows CASS schema (version pin if needed)
- No session capture code in PRV

### Reversal Triggers
- CASS deprecated or unavailable
- Schema changes break PRV repeatedly

---

## ADR-007: Rust as Implementation Language

**Status:** accepted
**Date:** 2024-12-27

### Context
PRV is a developer tool requiring fast performance (<500ms hover), reliable operation, and good ecosystem support.

### Options

| Option | Summary | Pros | Cons |
|:-------|:--------|:-----|:-----|
| A | Go | Fast compilation, simple | GC pauses, less ecosystem for LSP |
| B | Rust | Fast, no GC, strong LSP ecosystem | Steeper learning curve |
| C | TypeScript | Fast dev, large ecosystem | Slower runtime |

### Decision
Option B: Rust

### Rationale
Performance-critical tool with LSP requirements. Rust ecosystem has excellent LSP libraries (tower-lsp). No GC pauses for consistent latency. Single binary distribution.

### Consequences
- Rust toolchain required for development
- Steeper contribution barrier
- Excellent runtime performance

### Reversal Triggers
- Team unable to maintain Rust code
- Critical library unavailable in Rust

---

## ADR-008: LSP Server as Primary Interface

**Status:** accepted
**Date:** 2024-12-27

### Context
PRV context should be accessible in any editor without workflow change. Need editor-agnostic solution.

### Options

| Option | Summary | Pros | Cons |
|:-------|:--------|:-----|:-----|
| A | VS Code extension only | Rich UI | Locks out other editors |
| B | CLI only | Universal | Workflow friction |
| C | LSP server | Works in any LSP editor | Limited to hover/completion |

### Decision
Option C: LSP server with CLI secondary

### Rationale
LSP hover works in VS Code, Neovim, Zed, Helix, Emacs, etc. Zero workflow change — hover over a line to see context. CLI supplements for queries and automation.

### Consequences
- Must implement LSP server (tower-lsp in Rust)
- Hover is primary surface
- CLI for `prv blame`, `prv query`, `prv heatmap`

### Reversal Triggers
- LSP hover insufficient for rich context display
- Major editors drop LSP support

---

## ADR-009: Enhanced Summaries for Cross-Machine Sharing

**Status:** accepted
**Date:** 2024-12-27

### Context
Context needs to be shareable across team members on different machines. Raw transcripts are too large, contain sensitive info, and don't extract key insights. Pure metadata is insufficient.

### Options

| Option | Summary | Pros | Cons |
|:-------|:--------|:-----|:-----|
| A | Metadata only | Small, safe | Missing context |
| B | Full transcripts | Complete | Privacy risk, large |
| C | AI-generated summaries | Right balance | LLM dependency, cost |

### Decision
Option C: AI-generated enhanced summaries

### Rationale
LLM processes session transcripts locally to extract:
- **What was done** (structured summary)
- **Why it was done** (reasoning, constraints)
- **What was rejected** (alternatives considered, roads not taken)
- **Key decisions** (choices made during session)

Summaries are small, shareable, and preserve the most valuable context. "Roads Not Taken" is a key differentiator — knowing what was rejected is as valuable as knowing what was chosen.

### Consequences
- LLM integration required (local or API)
- Summary schema needs design
- Cost per session (minimal for local models)
- Summaries stored in prv-memory branch

### Reversal Triggers
- Summary quality too low to be useful
- Users prefer full transcripts despite size
- LLM costs become prohibitive

---

## ADR-010: Provenance Heat Map Visualization

**Status:** accepted
**Date:** 2024-12-27

### Context
Code reviewers and developers need to quickly see which parts of a codebase have provenance (known origin) vs. blind spots (unknown origin). Line-by-line hover is too slow for orientation.

### Options

| Option | Summary | Pros | Cons |
|:-------|:--------|:-----|:-----|
| A | Per-file coverage score | Simple | No visual orientation |
| B | Heat map overlay in editor | Immediate visual | Editor integration work |
| C | CLI-generated HTML report | Works everywhere | Not inline |

### Decision
Option B: Heat map overlay via LSP with Option C as fallback

### Rationale
Visual overlay in editor gutter shows green (traced) vs. red/gray (unknown) for each line. Immediate orientation for reviewers. LSP `textDocument/documentColor` or custom CodeLens for implementation.

CLI `prv heatmap <file>` generates terminal-colored or HTML output for environments without LSP.

### Consequences
- LSP must support color annotations or CodeLens
- Need per-line provenance scoring
- CSS/styling for HTML report

### Reversal Triggers
- LSP color support too limited across editors
- Users prefer simple coverage percentage
