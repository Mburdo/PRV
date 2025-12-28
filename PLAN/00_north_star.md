# North Star Card — PRV

## The Goal

Context tracing for code. Trace any line back to its origin — the session that created it, the reasoning behind it, and everything else created alongside it.

## Who It's For

- **Primary:** Code reviewers who need to understand AI-generated changes
- **Secondary:** Developers doing archaeology on unfamiliar code, teams needing provenance/audit trails

## The Problem

Agents write code at incredible velocity. The reasoning disappears. No one knows where code came from or why.

## Build Profile

**Startup MVP** — OSS project, ship fast, iterate on feedback

## Rigor Tier

**Tier 2 (Balanced)** — Unit + integration tests, ADRs for decisions

## Success Metrics (Ranked)

1. Hover over any line → see origin context in <500ms
2. 80%+ of AI-generated commits correctly traced to sessions
3. Zero workflow change — invisible capture via CASS
4. Editor-agnostic — LSP works in VS Code, Neovim, Zed, etc.

## Non-Goals (v1)

- GitHub/GitLab PR integration
- Secret detection/redaction
- Standalone mode (CASS required)
- Windows support
- Perfect attribution (heuristic acceptable)

## Constraints

- **Language:** Rust
- **Dependency:** CASS (session data via SQLite)
- **Primary interface:** LSP server (hover in any editor)
- **Secondary interface:** CLI (`prv blame`, `prv query`)
- **Transport:** Git-native, local-first

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
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │   Ingest    │→ │ Fingerprint │→ │       Link          │ │
│  │ (from CASS) │  │ (hunk hash) │  │ (session → commit)  │ │
│  └─────────────┘  └─────────────┘  └──────────┬──────────┘ │
│                                               │            │
│  ┌────────────────────────────────────────────▼──────────┐ │
│  │                    Surface                            │ │
│  │  LSP: hover in any editor                             │ │
│  │  CLI: prv blame, prv query                            │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Stop/Ask Rules

Agents should pause and ask if:
- Implementation deviates from existing ADRs
- Security implications arise (transcript exposure, permissions)
- Matching algorithm changes would affect accuracy significantly
- New external dependencies are being considered

## Open Questions

- [ ] CASS SQLite schema structure — need to investigate
- [ ] Fingerprinting edge cases — rebases, squashes, cherry-picks
- [ ] License choice — MIT vs Apache 2.0
- [ ] prv-memory sync — still needed? Or simplify for v1?

## Key Decisions (ADRs)

| ADR | Decision |
|-----|----------|
| ADR-001 | Hunk fingerprinting for commit↔session matching |
| ADR-002 | Orphan branch transport (prv-memory) |
| ADR-003 | Pipes default, PTY opt-in (from original design) |
| ADR-004 | Queue-then-push security model |
| ADR-005 | Indexed lookup for O(1) candidate matching |
| ADR-006 | CASS integration via SQLite (not wrapping) |
| ADR-007 | Rust as implementation language |
| ADR-008 | LSP server as primary interface |
