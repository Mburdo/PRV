# Phase 0 Beads Created

## Epic
| ID | Title | Status |
|----|-------|--------|
| PRV-u0f | Phase 0: Foundation | open |

## Task Beads

| ID | Title | Dependencies | Status |
|----|-------|--------------|--------|
| PRV-u0f.1 | Cargo workspace setup | - | open |
| PRV-u0f.2 | prv-core CodeBlock struct | .1 | open |
| PRV-u0f.3 | Code block parser | .2 | open |
| PRV-u0f.4 | Code block parser tests | .3 | open |
| PRV-u0f.5 | prv-cass models | .1 | open |
| PRV-u0f.6 | CASS database connection | .5 | open |
| PRV-u0f.7 | Workspace-repo mapping | .6 | open |
| PRV-u0f.8 | CLI binary with clap | .1 | open |
| PRV-u0f.9 | prv debug cass command | .3, .7, .8 | open |

## Dependency Graph

```
PRV-u0f.1 (Cargo workspace)
    │
    ├──────────────────┬──────────────────┐
    ▼                  ▼                  ▼
PRV-u0f.2          PRV-u0f.5          PRV-u0f.8
(CodeBlock)        (CASS models)      (CLI binary)
    │                  │                  │
    ▼                  ▼                  │
PRV-u0f.3          PRV-u0f.6             │
(Parser)           (CASS db)             │
    │                  │                  │
    ▼                  ▼                  │
PRV-u0f.4          PRV-u0f.7             │
(Tests)            (Workspace map)       │
    │                  │                  │
    └──────────────────┴──────────────────┘
                       │
                       ▼
                   PRV-u0f.9
               (prv debug cass)
```

## Execution Order

**Parallel Track A:** .1 → .2 → .3 → .4
**Parallel Track B:** .1 → .5 → .6 → .7
**Parallel Track C:** .1 → .8
**Integration:** .9 (after A, B, C complete)

## Validation

- `bv --robot-plan`: 2 actionable, 8 blocked ✓
- `bv --robot-alerts`: 0 critical, 0 warning, 1 info ✓
- No cycles detected ✓

## Ready for Execution

Start with PRV-u0f.1 (Cargo workspace setup) - it unblocks 3 downstream tasks.
