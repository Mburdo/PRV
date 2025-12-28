# /execute

Parallel execution orchestrator. Picks up existing beads and runs them with multiple agents working in parallel.

## Usage

```
/execute              # Execute all ready beads
/execute --dry-run    # Show plan without running
/execute --phase 2    # Execute specific phase only
/execute --agents 3   # Limit to 3 parallel agents
```

## What It Does

1. **Discovers** ready beads and computes phases from dependency graph
2. **Computes tracks** — groups beads by non-overlapping files
3. **Spawns workers** — one agent per track, running in parallel
4. **Monitors** — watches for [TRACK COMPLETE] and [BLOCKED] messages
5. **Calibrates** — runs /calibrate at phase boundaries
6. **Continues** — spawns next phase workers after calibration passes

## Prerequisites

- Beads must exist (run planning + decomposition first)
- At least one bead in `ready` status
- Agent Mail available

## See Also

- `/prime` — Start a session (workers run this)
- `/next-bead` — Claim and execute a single bead
- `/calibrate` — Phase boundary checkpoint
- `/decompose-task` — Create beads from a phase doc
