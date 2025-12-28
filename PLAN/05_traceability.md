# Traceability Matrix — PRV

## Scope
- **Project:** PRV (Context Tracing)
- **Date:** 2024-12-27
- **Rigor Tier:** 2

---

## North Star Success Metrics → Requirements

| NS Success Metric | Primary REQ | Supporting REQs | Phase |
|-------------------|-------------|-----------------|-------|
| 1. Hover → context with alternatives in <500ms | REQ-001 | REQ-010 | 2 |
| 2. 80%+ commits traced to sessions | REQ-002 | REQ-003, REQ-004 | 1 |
| 3. Zero workflow change (invisible via CASS) | REQ-004 | REQ-007 | 0-1 |
| 4. Editor-agnostic LSP | REQ-001 | - | 2 |
| 5. Heat map shows provenance coverage | REQ-011 | REQ-008 | 2 |
| 6. Summaries include rejected alternatives | REQ-010 | REQ-012 | 2 |

---

## Coverage Matrix

### Core Linking (P0)

| ID | Requirement | ADRs | Spikes | Tests | Status |
|----|-------------|------|--------|-------|--------|
| REQ-001 | LSP Hover Context | ADR-008 | SPIKE-003 | - | planned |
| REQ-002 | Commit-Session Linking | ADR-001, ADR-005, ADR-006 | SPIKE-001, SPIKE-002 | - | planned |
| REQ-003 | Code Block Extraction | ADR-006 | SPIKE-001 | - | planned |
| REQ-004 | Workspace-Repo Mapping | ADR-006 | SPIKE-001 | - | planned |

### Differentiating Features (P0)

| ID | Requirement | ADRs | Spikes | Tests | Status |
|----|-------------|------|--------|-------|--------|
| REQ-010 | Roads Not Taken Extraction | ADR-009 | SPIKE-004 | - | planned |
| REQ-011 | Provenance Heat Map | ADR-010 | SPIKE-005 | - | planned |

### CLI & Interface (P1)

| ID | Requirement | ADRs | Spikes | Tests | Status |
|----|-------------|------|--------|-------|--------|
| REQ-005 | CLI Query Interface | ADR-008 | - | - | planned |
| REQ-006 | Session Context Display | ADR-009 | - | - | planned |
| REQ-007 | Git Hook Linking | ADR-004 | - | - | planned |
| REQ-012 | Enhanced Summary Generation | ADR-009 | SPIKE-004 | - | planned |

### Advanced Features (P2)

| ID | Requirement | ADRs | Spikes | Tests | Status |
|----|-------------|------|--------|-------|--------|
| REQ-008 | Index for Fast Lookup | ADR-005 | - | - | planned |
| REQ-009 | Shareable Context (prv-memory) | ADR-002, ADR-004 | - | - | planned |
| REQ-013 | Evolution Graph | ADR-009 | - | - | planned |

---

## ADR → Requirement Mapping

| ADR | Title | Requirements |
|-----|-------|--------------|
| ADR-001 | Hunk Fingerprinting | REQ-002 |
| ADR-002 | Orphan Branch Transport | REQ-009 |
| ADR-003 | (Deprecated) | - |
| ADR-004 | Queue-then-Push Security | REQ-007, REQ-009 |
| ADR-005 | Indexed Lookup | REQ-002, REQ-008 |
| ADR-006 | CASS Integration | REQ-002, REQ-003, REQ-004 |
| ADR-007 | Rust Language | (all) |
| ADR-008 | LSP Primary Interface | REQ-001, REQ-005 |
| ADR-009 | Enhanced Summaries | REQ-006, REQ-010, REQ-012, REQ-013 |
| ADR-010 | Heat Map Visualization | REQ-011 |

---

## Spike → Requirement Mapping

| Spike | Goal | Requirements Validated |
|-------|------|------------------------|
| SPIKE-001 | CASS Schema | REQ-002, REQ-003, REQ-004 |
| SPIKE-002 | Fingerprinting | REQ-002 |
| SPIKE-003 | LSP Performance | REQ-001 |
| SPIKE-004 | LLM Summary Extraction | REQ-010, REQ-012 |
| SPIKE-005 | Heat Map LSP | REQ-011 |

---

## Gap Check

| Pattern | Symptom | Status |
|---------|---------|--------|
| SPIKE-001 | CASS schema validation | **Complete** |
| SPIKE-002 | Fingerprinting validation | Pending |
| SPIKE-003 | LSP performance validation | Pending |
| SPIKE-004 | LLM extraction validation | Pending |
| SPIKE-005 | Heat map LSP validation | Pending |

---

## Phase → Requirements Mapping

See `PLAN/06_phases.md` for full phase details.

| Phase | Milestone | Requirements |
|-------|-----------|--------------|
| 0 | "It Reads" | REQ-003, REQ-004 |
| 1 | "It Links" | REQ-002, REQ-007, REQ-008 |
| 2 | "It Explains" | REQ-001, REQ-006, REQ-010, REQ-011, REQ-012 |
| 3 | "It Shares" (v1.0) | REQ-005, REQ-009 |

### v1.1
- REQ-013: Evolution Graph
- Improved fingerprinting accuracy

### v2.0
- Provenance-aware AI assistance
- GitHub/GitLab integration
