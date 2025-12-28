# Requirements — PRV

## Scope

- **Project:** PRV (Context Tracing)
- **Date:** 2024-12-27
- **Rigor Tier:** 2

---

## REQ-001 (P0): Context Lookup via LSP Hover

**What must be true:**
When a user hovers over any line of code in an LSP-compatible editor, PRV returns the origin context for that line within 500ms.

**Acceptance Criteria:**
- AC-001.1: Hover over a line → see which session created it (if known)
- AC-001.2: Response includes: agent name, timestamp, session title
- AC-001.3: Response time < 500ms for 95th percentile
- AC-001.4: If no match found, return graceful "unknown origin" message

**Assumptions:**
- CASS database is indexed and available (HIGH confidence)
- Git blame is available for the file (HIGH confidence)
- LSP hover protocol supports markdown responses (HIGH confidence)

**Dependencies:**
- CASS SQLite database
- Git repository with history

---

## REQ-002 (P0): Commit-to-Session Linking

**What must be true:**
PRV can correlate a git commit to the CASS session(s) that likely produced it.

**Acceptance Criteria:**
- AC-002.1: Given a commit SHA, return candidate session IDs with confidence scores
- AC-002.2: Matching uses time window (session.started_at ≤ commit_time ≤ session.ended_at + buffer)
- AC-002.3: Matching uses workspace path (session workspace = git repo root)
- AC-002.4: Matching uses content fingerprinting (code in session ≈ code in diff)
- AC-002.5: 80%+ accuracy on commits made during AI sessions (measured by manual audit)

**Assumptions:**
- Commits made shortly after sessions end are still matchable (MED confidence)
- Code blocks in messages contain the actual code that was committed (MED confidence)

**Dependencies:**
- Git log access
- CASS conversations + messages tables

---

## REQ-003 (P0): Code Block Extraction

**What must be true:**
PRV extracts code blocks from CASS message content for fingerprinting.

**Acceptance Criteria:**
- AC-003.1: Parse triple-backtick code blocks from message content
- AC-003.2: Handle language tags (```python, ```rust, etc.)
- AC-003.3: Handle nested or malformed blocks gracefully (don't crash)
- AC-003.4: Extract file paths when mentioned near code blocks

**Assumptions:**
- Most code is in triple-backtick blocks (HIGH confidence based on spike)
- Some code may be inline without blocks (MED confidence - accept missing these)

**Dependencies:**
- CASS messages table

---

## REQ-004 (P0): Workspace-to-Repo Mapping

**What must be true:**
PRV maps CASS workspaces to git repository roots.

**Acceptance Criteria:**
- AC-004.1: Given a git repo path, find matching CASS workspace
- AC-004.2: Handle path variations (trailing slashes, symlinks)
- AC-004.3: Cache workspace mappings for performance

**Assumptions:**
- workspace.path in CASS matches actual filesystem paths (HIGH confidence - verified in spike)

**Dependencies:**
- CASS workspaces table

---

## REQ-005 (P1): CLI Query Interface

**What must be true:**
Users can query context from the command line.

**Acceptance Criteria:**
- AC-005.1: `prv blame <file>:<line>` returns origin context
- AC-005.2: `prv query <commit>` returns linked sessions
- AC-005.3: Output is human-readable by default, JSON with `--json` flag
- AC-005.4: Commands work without LSP server running

**Assumptions:**
- CLI is secondary interface; LSP is primary (per North Star)

**Dependencies:**
- Same linking logic as LSP

---

## REQ-006 (P1): Session Context Display

**What must be true:**
When PRV returns a match, it includes useful context about the session.

**Acceptance Criteria:**
- AC-006.1: Display agent name (claude_code, cursor, codex, etc.)
- AC-006.2: Display session timestamp (human-readable)
- AC-006.3: Display session title or first prompt
- AC-006.4: Display other files touched in same session (if detectable)
- AC-006.5: Provide link/path to view full transcript

**Assumptions:**
- Session title is meaningful (MED confidence - some are auto-generated)

**Dependencies:**
- CASS conversations table

---

## REQ-007 (P1): Git Hook for Automatic Linking

**What must be true:**
PRV can run as a post-commit hook to link commits to sessions automatically.

**Acceptance Criteria:**
- AC-007.1: `prv link --commit HEAD` creates a link record
- AC-007.2: Hook is non-blocking (times out gracefully, never blocks commit)
- AC-007.3: Links stored in `.prv/links/<commit>.json`
- AC-007.4: `prv install` sets up the hook

**Assumptions:**
- Users will run `prv install` to enable automatic linking (MED confidence)

**Dependencies:**
- Git hooks infrastructure

---

## REQ-008 (P2): Index for Fast Lookup

**What must be true:**
PRV maintains an index for O(1) lookup from line → session.

**Acceptance Criteria:**
- AC-008.1: Index maps (file, line_range) → session candidates
- AC-008.2: Index is rebuilt on `prv index` command
- AC-008.3: Index is invalidated when CASS data changes
- AC-008.4: Lookup without index falls back to slower path

**Assumptions:**
- Index provides meaningful speedup for large repos (MED confidence - needs benchmarking)

**Dependencies:**
- Link records from REQ-007

---

## REQ-009 (P2): Shareable Context (prv-memory)

**What must be true:**
Teams can share PRV context via git without polluting main history.

**Acceptance Criteria:**
- AC-009.1: `prv sync push` pushes links to orphan branch `prv-memory`
- AC-009.2: `prv sync pull` fetches links from remote
- AC-009.3: Explicit confirmation before pushing (transcripts may contain secrets)
- AC-009.4: Works with standard git remotes

**Assumptions:**
- Teams want to share context (MED confidence - may be v2)

**Dependencies:**
- Git orphan branch operations

---

## REQ-010 (P0): Rejected Alternatives Extraction (Roads Not Taken)

**What must be true:**
PRV extracts and displays alternatives that were considered but rejected during a session.

**Acceptance Criteria:**
- AC-010.1: Parse session content for rejected approaches (e.g., "instead of X, let's do Y")
- AC-010.2: LLM summarization explicitly extracts "roads not taken"
- AC-010.3: Hover context includes rejected alternatives when available
- AC-010.4: Summary schema has dedicated `alternatives` field

**Assumptions:**
- Sessions contain discussion of alternatives (MED confidence - varies by user)
- LLM can reliably identify rejected vs. chosen approaches (MED confidence)

**Dependencies:**
- LLM integration for summarization
- Enhanced summary schema (ADR-009)

---

## REQ-011 (P0): Provenance Heat Map

**What must be true:**
Users can see a visual overview of which code has known provenance vs. unknown origin.

**Acceptance Criteria:**
- AC-011.1: `prv heatmap <file>` shows per-line provenance status
- AC-011.2: LSP provides color annotations or CodeLens for in-editor display
- AC-011.3: Green = traced, Yellow = partial, Gray/Red = unknown
- AC-011.4: File-level coverage percentage available

**Assumptions:**
- LSP editors support `textDocument/documentColor` or CodeLens (HIGH confidence)
- Visual overlay doesn't interfere with normal editing (HIGH confidence)

**Dependencies:**
- Link index from REQ-008
- LSP server from REQ-001

---

## REQ-012 (P1): Enhanced Summary Generation

**What must be true:**
PRV generates structured, shareable summaries from session transcripts.

**Acceptance Criteria:**
- AC-012.1: LLM processes session to extract summary, reasoning, alternatives, decisions
- AC-012.2: Summary schema is well-defined and versioned
- AC-012.3: Summaries stored in `.prv/summaries/<session_id>.json`
- AC-012.4: `prv summarize <session_id>` generates summary on demand
- AC-012.5: Summaries < 2KB per session (efficient for sync)

**Assumptions:**
- Local LLM (ollama) or API available (MED confidence)
- Summary quality sufficient for useful context (MED confidence - needs validation)

**Dependencies:**
- CASS session access from REQ-002
- prv-memory sync from REQ-009

---

## REQ-013 (P2): Evolution Graph

**What must be true:**
PRV can chain sessions over time to show how understanding of code evolved.

**Acceptance Criteria:**
- AC-013.1: Link sessions that touch same code areas
- AC-013.2: `prv evolution <file>` shows timeline of sessions affecting that file
- AC-013.3: Visualization shows decision evolution (chose A, then changed to B)
- AC-013.4: Export as timeline/graph for documentation

**Assumptions:**
- Sessions touching same code can be reliably linked (MED confidence)
- Evolution patterns are meaningful to users (LOW confidence - needs validation)

**Dependencies:**
- Multi-session linking
- Enhanced summaries from REQ-012

---

## Open Questions

- [x] Should PRV store its own copy of session data, or always read from CASS? → Read from CASS (ADR-006)
- [ ] How to handle sessions that span multiple commits?
- [ ] What's the right "buffer" time after session end for matching?
- [x] Should hover show transcript excerpt, or just link to full? → Show enhanced summary (ADR-009)
- [ ] Which LLM for summarization? Local (ollama) vs API?
- [ ] Summary schema design
