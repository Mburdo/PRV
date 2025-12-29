# North Star Card — PRV

## The Goal

Context tracing for code. Trace any line back to its origin — the session that created it, the reasoning behind it, **the alternatives that were rejected**, and everything else created alongside it.

**Tagline:** Git tells you what changed. PRV tells you why, what else was considered, and how understanding evolved.

## Who It's For

- **Primary:** Code reviewers who need to understand AI-generated changes
- **Secondary:** Developers doing archaeology on unfamiliar code, teams needing provenance/audit trails

## The Problem

Agents write code at incredible velocity. The reasoning disappears. No one knows where code came from or why. Worse: no one knows what alternatives were considered and rejected.

## What Makes PRV Different

| Others Do | PRV Does |
|-----------|----------|
| Track who wrote code | Track **why** and **what was rejected** |
| Show commit history | Show **decision history** |
| Metadata only | **Structured summaries** with alternatives |
| Per-file view | **Heat map** showing provenance coverage |

### Differentiating Features

1. **Roads Not Taken** — Extract rejected alternatives from sessions, not just what was chosen
2. **Provenance Heat Map** — Visual overlay showing which code has context vs. blind spots
3. **Enhanced Summaries** — AI-generated summaries shared via prv-memory (not raw transcripts)
4. **Evolution Graph** — Chain sessions over time to show how code understanding evolved (v1.1)

## Build Profile

**Startup MVP** — OSS project, ship fast, iterate on feedback

## Rigor Tier

**Tier 2 (Balanced)** — Unit + integration tests, ADRs for decisions

## Success Metrics (Ranked)

1. Hover over any line → see origin context with alternatives in <500ms
2. **Working tree visibility:** provisional provenance appears within seconds of save for AI-generated code
3. 80%+ of AI-generated commits correctly traced to sessions (confirmed provenance)
4. Zero workflow change — invisible capture via CASS
5. Editor-agnostic — LSP works in VS Code, Neovim, Zed, etc.
6. Heat map shows provenance coverage for any file (confirmed + provisional)
7. Shared summaries include rejected alternatives (not just chosen approach)

## Non-Goals (v1)

- GitHub/GitLab PR integration (future)
- Secret detection/redaction
- Standalone mode (CASS required)
- Windows support
- Perfect attribution (heuristic acceptable)
- Full transcript sharing (summaries only)
- Per-keystroke capture (provisional state is updated on save, not every edit)
- Provenance-aware AI assistance (v2)

## Constraints

- **Language:** Rust
- **Dependency:** CASS (session data via SQLite)
- **Primary interface:** LSP server (hover in any editor)
- **Secondary interface:** CLI (`prv blame`, `prv query`, `prv heatmap`)
- **Transport:** Git-native, local-first
- **Sharing:** Enhanced summaries via prv-memory (not raw transcripts)

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    AI Tools (logging)                       │
│  Claude Code │ Cursor │ Codex │ Aider │ ChatGPT │ ...      │
└─────────────────────────┬───────────────────────────────────┘
                          │ (automatic)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                         CASS                                │
│  Discovery → Normalization → SQLite                         │
└─────────────────────────┬───────────────────────────────────┘
                          │ (PRV reads SQLite)
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                         PRV                                 │
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Ingest    │→ │ Fingerprint │→ │       Link          │ │
│  │ (from CASS) │  │ (hunk hash) │  │ (session → commit)  │ │
│  └─────────────┘  └─────────────┘  └──────────┬──────────┘ │
│                                               │            │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────▼──────────┐ │
│  │  Summarize  │← │   Extract   │← │    PRV Index        │ │
│  │  (LLM call) │  │ Alternatives│  │  (links + hashes)   │ │
│  └──────┬──────┘  └─────────────┘  └─────────────────────┘ │
│         │                                                   │
│  ┌──────▼────────────────────────────────────────────────┐ │
│  │                    Surface                            │ │
│  │  LSP: hover with context + alternatives               │ │
│  │  CLI: prv blame, prv query, prv heatmap               │ │
│  │  Sync: prv-memory with enhanced summaries             │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Stop/Ask Rules

Agents should pause and ask if:
- Implementation deviates from existing ADRs
- Security implications arise (transcript exposure, permissions)
- Matching algorithm changes would affect accuracy significantly
- New external dependencies are being considered
- LLM provider/model choices for summarization

## Open Questions

See `PLAN/01_requirements.md` for consolidated list. Summary:

**Resolved:**
- [x] CASS SQLite schema structure — SPIKE-001 complete
- [x] Context sharing model — Enhanced summaries (ADR-009)
- [x] Time prior for matching — ECDF/decay (no hard cutoff)
- [x] Heat map thresholds — >80%/20-80%/<20% (AC-011.3)
- [x] Summary schema — Defined in AC-010.6
- [x] Fingerprinting algorithm — 3-step pipeline (gates → step0 → step1 → step2)
- [x] CASS snippets empty — Extract from message content with regex

**Open (will be resolved by spikes):**
- [ ] LLM for summarization — SPIKE-004
- [ ] License choice — MIT vs Apache 2.0 (decision needed)

## Key Decisions (ADRs)

| ADR | Decision |
|-----|----------|
| ADR-001 | Hunk fingerprinting for commit↔session matching |
| ADR-002 | Orphan branch transport (prv-memory) |
| ADR-003 | Pipes default, PTY opt-in (deprecated - using CASS) |
| ADR-004 | Queue-then-push security model |
| ADR-005 | Indexed lookup for O(1) candidate matching |
| ADR-006 | CASS integration via SQLite (not wrapping) |
| ADR-007 | Rust as implementation language |
| ADR-008 | LSP server as primary interface |
| ADR-009 | Enhanced summaries for cross-machine sharing (not raw transcripts) |
| ADR-010 | Provenance heat map visualization |

## Roadmap

### v1.0 (MVP)
- Core linking (session → commit)
- Provisional working-tree overlay for AI-generated changes
- LSP hover with context
- CLI commands (blame, query)
- Enhanced summaries with alternatives
- Provenance heat map
- prv-memory sync

### v1.1
- Evolution graph (session chains over time)
- Cross-session intelligence
- Improved fingerprinting accuracy

### v2.0
- Provenance-aware AI (AI uses context when helping)
- GitHub/GitLab integration
- Time travel understanding
