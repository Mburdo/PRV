# Risks & Spikes — PRV

## Top Risks

| Risk | Why It Matters | Likelihood | Impact | Resolution |
|:-----|:---------------|:-----------|:-------|:-----------|
| CASS schema changes | PRV depends on CASS SQLite structure; breaking changes break PRV | M | H | Spike + version pinning |
| Fingerprint accuracy | Heuristic matching may miss or false-positive on edge cases | M | M | Spike + accept imperfection |
| LSP performance | Hover must return in <500ms; git blame + CASS lookup chain | L | H | Benchmark early |
| Rebase/squash handling | Git history rewrites orphan existing links | M | M | Accept or re-link strategy |
| CASS not installed | PRV requires CASS; cold start UX | L | M | Clear error + install guide |
| LLM summary quality | "Roads not taken" extraction may be inconsistent or miss alternatives | M | H | SPIKE-004 + prompt iteration |
| LLM availability/cost | Local LLM may be too slow; API costs may accumulate | M | M | Support both, default local |
| Heat map LSP support | Gutter annotations may not work across all editors | L | M | SPIKE-005 + CLI fallback |
| Summary schema evolution | Schema changes break existing summaries | L | M | Version field + migration |
| Alternative extraction reliability | Not all sessions discuss alternatives explicitly | H | M | Accept partial extraction, enrich UI

---

## Spikes

### SPIKE-001: CASS SQLite Schema Investigation

**Status:** Active

**Goal:** Understand the CASS SQLite schema to design PRV's ingest layer.

**Questions to answer:**
1. What tables exist? What are the key columns?
2. How are sessions/conversations structured?
3. How are messages linked to sessions?
4. Is there file/path information we can use for fingerprinting?
5. Is there timestamp data for correlation with commits?
6. What's the schema version situation? Is it stable?

**Timebox:** 1 hour

**Steps:**
1. Locate CASS SQLite database (`~/.cass/` or similar)
2. Dump schema with `.schema`
3. Sample data from key tables
4. Document structure and relevant fields
5. Identify any gaps for PRV's needs

**Output:**
- Schema documentation
- Assessment of what's available vs. what PRV needs
- Any concerns or blockers

**Decision it unlocks:**
- ADR-006 confirmation (can we actually read what we need?)
- Requirements for ingest layer

---

### SPIKE-002: Fingerprinting Strategy (Future)

**Status:** Pending (after SPIKE-001)

**Goal:** Validate hunk fingerprinting approach with real data.

**Questions to answer:**
1. How do we extract code snippets from CASS sessions?
2. What hashing strategy gives best match accuracy?
3. How do we handle partial matches?
4. What about formatting-only changes?

**Timebox:** 2 hours

**Output:** Prototype fingerprinting code + accuracy assessment

---

### SPIKE-003: LSP Performance Baseline (Future)

**Status:** Pending

**Goal:** Validate that the git blame → CASS lookup → response chain can hit <500ms.

**Timebox:** 1 hour

**Output:** Benchmark numbers, bottleneck identification

---

### SPIKE-004: LLM Summary Extraction with Alternatives

**Status:** Pending (after SPIKE-002)

**Goal:** Validate LLM-based extraction of enhanced summaries including "roads not taken."

**Questions to answer:**
1. Can we reliably extract rejected alternatives from session transcripts?
2. What prompt structure yields best summary quality?
3. Local model (ollama/llama) vs API (Claude) - quality vs. cost tradeoffs?
4. What summary schema captures the right information?
5. Can we stay under 2KB per summary?

**Timebox:** 3 hours

**Steps:**
1. Sample 10 real sessions from CASS with varying complexity
2. Test prompt variations for alternative extraction
3. Compare local (llama3) vs API (Claude Haiku) quality
4. Draft summary JSON schema
5. Measure output sizes

**Output:**
- Recommended model/approach
- Prompt template
- Summary schema v1
- Quality assessment (manual review)

**Decision it unlocks:**
- ADR-009 validation (does this actually work?)
- REQ-010 feasibility
- Cost model for API usage

---

### SPIKE-005: Heat Map LSP Implementation (Future)

**Status:** Pending

**Goal:** Validate LSP mechanisms for per-line provenance visualization.

**Questions to answer:**
1. Does `textDocument/documentColor` work for gutter annotations?
2. Can CodeLens provide per-line provenance indicators?
3. Which approach works across VS Code, Neovim, Zed?
4. Performance impact of per-line annotations?

**Timebox:** 2 hours

**Output:**
- Recommended LSP mechanism
- Editor compatibility matrix
- Prototype implementation

---

## Spike Outcomes Log

| Spike | Date | Outcome | Decision Made |
|-------|------|---------|---------------|
| SPIKE-001 | 2024-12-27 | Completed - schema documented | Proceed with CASS integration |

---

## SPIKE-001 Results: CASS SQLite Schema

### Database Location
```
/Users/burdo/Library/Application Support/com.coding-agent-search.coding-agent-search/agent_search.db
```

### Schema Overview

```
agents          → AI tools (codex, claude_code, cursor, etc.)
workspaces      → Project directories (maps to git repos!)
conversations   → Sessions with timestamps, workspace, source file
messages        → Individual messages with role, content, timestamps
snippets        → Code snippets (EXISTS BUT EMPTY - not populated)
fts_messages    → Full-text search index
```

### Key Tables for PRV

**conversations**
| Column | Type | Use for PRV |
|--------|------|-------------|
| id | INTEGER | Link target |
| workspace_id | INTEGER | Maps to git repo path |
| started_at | INTEGER | Correlation with commit time |
| ended_at | INTEGER | Session duration |
| source_path | TEXT | Original session file |
| title | TEXT | Display in hover |

**messages**
| Column | Type | Use for PRV |
|--------|------|-------------|
| conversation_id | INTEGER | Links to session |
| role | TEXT | user/agent |
| content | TEXT | Contains code blocks (``` delimited) |
| created_at | INTEGER | Fine-grained timestamps |

**workspaces**
| Column | Type | Use for PRV |
|--------|------|-------------|
| path | TEXT | **CRUCIAL**: Maps directly to git repo path |

### Key Findings

1. **Workspace = Git Repo**: `workspaces.path` contains absolute paths like `/Users/burdo/Desktop/Lucidity Backend`. This maps directly to git repository roots.

2. **Timestamps in milliseconds**: All `_at` fields are Unix epoch milliseconds. Can correlate with `git log --format=%at`.

3. **Snippets table is EMPTY**: Despite schema existing, no data. PRV must extract code from message content (parse ``` blocks).

4. **Code in messages**: Agent messages contain code blocks with triple backticks. PRV will need to parse these.

5. **545 conversations, 55,775 messages**: Substantial data to work with.

### Correlation Strategy

```
Git commit
    │
    ├── timestamp (commit time)
    ├── repo path (working directory)
    └── diff content (code changes)
         │
         ▼
CASS lookup:
    1. Match workspace.path to git repo
    2. Find conversations where started_at <= commit_time <= ended_at + buffer
    3. Parse code blocks from agent messages
    4. Fingerprint match against commit diff
    5. Return matching conversation(s)
```

### Concerns

1. **Empty snippets table**: Extra work to parse code blocks from content
2. **Timestamp granularity**: Sessions may span multiple commits
3. **No file-level tracking**: CASS doesn't track which files were modified in a session (we infer from content)

### Decision

**Proceed with CASS integration.** The schema provides:
- ✅ Workspace → repo mapping
- ✅ Timestamps for correlation
- ✅ Full message content with code
- ✅ Agent identification

PRV will need to:
- Parse code blocks from message content
- Implement time-window + fingerprint matching
- Handle the case where snippets table is empty
