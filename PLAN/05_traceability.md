# Traceability Matrix — PRV

## Scope
- **Project:** PRV (Context Tracing)
- **Date:** 2024-12-27
- **Rigor Tier:** 2

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

## Milestone Mapping

### v1.0 (MVP)
- REQ-001: LSP Hover
- REQ-002: Commit-Session Linking
- REQ-003: Code Block Extraction
- REQ-004: Workspace Mapping
- REQ-005: CLI Query
- REQ-010: Roads Not Taken
- REQ-011: Heat Map
- REQ-012: Enhanced Summaries

### v1.1
- REQ-013: Evolution Graph
- Improved fingerprinting accuracy

### v2.0
- Provenance-aware AI assistance
- GitHub/GitLab integration
