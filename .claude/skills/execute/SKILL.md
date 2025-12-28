---
name: execute
description: Parallel execution orchestrator. Picks up existing beads, computes parallel tracks, spawns worker agents, coordinates via Agent Mail, runs calibration at phase boundaries.
---

# Execute — Parallel Execution Orchestrator

Picks up existing beads and executes them with multiple parallel agents.

## When This Applies

| Signal | Action |
|--------|--------|
| Beads exist, ready to execute | Run /execute |
| User says "execute" or "run" | Run /execute |
| User says "start working" | Run /execute |
| Resume after planning | Run /execute |

---

## Prerequisites

Before running /execute:
1. Beads must exist in `.beads/`
2. At least one bead must be in `ready` status
3. Agent Mail MCP must be available

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      EXECUTE ORCHESTRATOR                        │
│  You are here. You:                                              │
│  - Discover ready beads                                          │
│  - Compute parallel tracks                                       │
│  - Spawn worker agents                                           │
│  - Monitor via Agent Mail                                        │
│  - Run calibration at phase boundaries                           │
└─────────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  Worker Agent   │  │  Worker Agent   │  │  Worker Agent   │
│  Track A        │  │  Track B        │  │  Track C        │
│  (Task tool)    │  │  (Task tool)    │  │  (Task tool)    │
└─────────────────┘  └─────────────────┘  └─────────────────┘
         │                    │                    │
         └────────────────────┼────────────────────┘
                              │
                    Agent Mail Coordination
              (File reservations, [CLAIMED], [CLOSED])
```

---

## Execution Flow

### Step 1: Setup

```markdown
1. Register as orchestrator:
   - ensure_project(human_key=<project_path>)
   - register_agent(name="Coordinator", program="claude-code", model=<model>)

2. Create session tracking:
   - Initialize TodoWrite with execution stages
   - Note start time
```

### Step 2: Discovery

```bash
# Get all ready beads
bd ready --json

# Get graph analysis and execution plan
bv --robot-triage
bv --robot-plan
```

Parse the output to understand:
- Which beads are ready now
- What the dependency structure is
- Which beads must complete before others can start

### Step 3: Phase Identification

Beads naturally form phases based on dependencies:

```
Phase 1: Beads with no blockers (can start immediately)
Phase 2: Beads blocked by Phase 1 beads
Phase 3: Beads blocked by Phase 2 beads
...
```

For each phase, identify:
- Which beads belong to it
- When it can start (all blockers from previous phase done)

### Step 4: Track Computation

Within each phase, group beads into parallel tracks:

**Goal:** Beads in different tracks have NO overlapping files, so agents can work simultaneously without conflicts.

**Method:**
1. For each bead, extract file scope from:
   - `edit_locus` field if present
   - File paths mentioned in description
   - Parent directory patterns
2. Group beads whose file scopes don't overlap
3. Each group = one track

**Example:**
```
Phase 1 has 6 beads:
  bd-101, bd-102 → src/auth/**         → Track A
  bd-103, bd-104 → src/api/**          → Track B
  bd-105         → src/models/**       → Track C
  bd-106, bd-107 → tests/**            → Track D

4 tracks can run in parallel.
```

### Step 5: Spawn Workers

For each track, spawn a worker agent using the Task tool:

```python
Task(
    description=f"Worker: Track {track.name}",
    prompt=WORKER_PROMPT.format(
        track_name=track.name,
        bead_ids=track.beads,
        file_scope=track.files,
        project_path=project_path
    ),
    subagent_type="general-purpose",
    run_in_background=True
)
```

**Worker Prompt Template:**

```markdown
You are a worker agent executing beads for parallel execution.

## Your Assignment
- **Track:** {track_name}
- **Beads:** {bead_ids}
- **File Scope:** {file_scope}
- **Project:** {project_path}

## Instructions

### 1. Startup
Run `/prime` to register with Agent Mail and get oriented.

### 2. Execute Each Bead

For each bead in your assignment, in order:

1. **Claim it:**
   ```
   bd update {bead_id} --status in_progress --assignee {your_name}
   ```
   Reserve files via Agent Mail. Send [CLAIMED] announcement.

2. **Implement (TDD-first):**
   - Read bead description for test requirements
   - Write tests first
   - Implement to pass tests
   - Run `ubs --staged` before commit
   - Commit with bead ID in message

3. **Close it:**
   ```
   bd close {bead_id} --reason "Completed: <summary>"
   ```
   Release reservations. Send [CLOSED] announcement.

4. **Move to next bead** in your track.

### 3. Handle Failures

If a bead fails after 3 attempts:
1. STOP trying
2. Send [BLOCKED] message:
   ```
   send_message(
       to=["Coordinator"],
       subject="[BLOCKED] {your_name} - {bead_id}",
       body_md="Failed after 3 attempts.\n\nError: {details}",
       importance="urgent"
   )
   ```
3. Continue to next bead in your track (if any)

### 4. Track Complete

When ALL beads in your track are closed (or blocked):
```
send_message(
    to=["Coordinator"],
    subject="[TRACK COMPLETE] {track_name}",
    body_md="Beads completed: {list}\nBeads blocked: {list}",
    importance="high"
)
```

Then STOP and wait. Do not pick up other work.

## Rules
1. ONLY work on beads in YOUR track
2. ONLY touch files in YOUR scope
3. TDD always — tests before implementation
4. Security gate — ubs --staged before every commit
5. 3-try limit — after 3 failures, send [BLOCKED], move on
6. Announce everything — [CLAIMED], [CLOSED], [BLOCKED], [TRACK COMPLETE]
```

### Step 6: Monitor Progress

The orchestrator monitors workers:

```python
while phase_in_progress:
    # Check Agent Mail inbox
    messages = fetch_inbox("Coordinator", limit=20, since_ts=last_check)

    for msg in messages:
        if "[TRACK COMPLETE]" in msg.subject:
            mark_track_done(extract_track(msg))
            acknowledge_message(msg.id)

        elif "[BLOCKED]" in msg.subject:
            record_blocked(msg)
            acknowledge_message(msg.id)

    # Check background task status
    for task_id in worker_tasks:
        status = TaskOutput(task_id, block=False)
        if status.completed:
            handle_worker_complete(task_id)

    # Update TodoWrite with current state
    update_progress_display()

    # Check if phase is complete
    if all_tracks_done():
        break

    # Wait before next check (avoid spinning)
    # Continue with other orchestrator work or brief pause
```

### Step 7: Phase Boundary

When all tracks in current phase complete:

```python
# 1. Run calibration
run_calibrate()  # This runs /calibrate skill

# 2. Check result
if calibration_passed:
    send_message(
        to=[ALL_WORKERS],
        subject="[PHASE COMPLETE] Phase {n}",
        body_md="Calibration passed. Proceeding to Phase {n+1}."
    )

    # Compute tracks for next phase
    next_phase_beads = get_phase_beads(n + 1)
    if next_phase_beads:
        tracks = compute_tracks(next_phase_beads)
        spawn_workers(tracks)
    else:
        # No more phases
        execution_complete()

else:
    send_message(
        to=[ALL_WORKERS],
        subject="[CALIBRATION FAILED] Phase {n}",
        body_md="Issues found. Fix beads created. Stand by."
    )

    # Create fix beads from calibration output
    create_fix_beads(calibration_issues)

    # Add fix beads to current phase and re-run
    tracks = compute_tracks(fix_beads)
    spawn_workers(tracks)
```

### Step 8: Completion

When all phases complete:

```python
send_message(
    to=[ALL_AGENTS],
    subject="[EXECUTION COMPLETE]",
    body_md="All beads closed. Execution finished."
)

# Present summary to user
print("""
## Execution Complete

**Phases:** {n} completed
**Beads:** {total} closed, {blocked} blocked
**Time:** {duration}

### Next Steps
- Run /release for final checklist
- Review any blocked beads
""")
```

---

## Options

| Flag | Default | Description |
|------|---------|-------------|
| `--agents N` | Auto | Maximum parallel agents (default: one per track) |
| `--phase N` | All | Execute only specific phase |
| `--dry-run` | false | Show tracks and plan without spawning |
| `--sequential` | false | Run one agent at a time (for debugging) |

---

## Agent Mail Protocol

| Subject | Sender | Recipients | When |
|---------|--------|------------|------|
| `[CLAIMED] bd-XXX - Title` | Worker | All agents | Starting a bead |
| `[CLOSED] bd-XXX - Title` | Worker | All agents | Finished a bead |
| `[BLOCKED] Agent - bd-XXX` | Worker | Coordinator | Stuck after 3 tries |
| `[TRACK COMPLETE] Track A` | Worker | Coordinator | All beads in track done |
| `[PHASE COMPLETE] Phase N` | Coordinator | All agents | All tracks done, calibration passed |
| `[CALIBRATION FAILED] Phase N` | Coordinator | All agents | Issues found |
| `[EXECUTION COMPLETE]` | Coordinator | All agents | All phases done |

---

## Error Handling

### Worker Gets Blocked

1. Worker sends `[BLOCKED]` message with error details
2. Worker continues to next bead in track (if any)
3. Orchestrator records blocked bead
4. At phase end, orchestrator can:
   - Create spike bead to investigate
   - Reassign to different agent
   - Escalate to user

### Calibration Fails

1. Calibration identifies issues
2. Orchestrator creates fix beads
3. Fix beads added to current phase
4. New workers spawned for fix beads
5. Re-run calibration when fixes complete

### Worker Crashes

1. Detect via TaskOutput showing unexpected completion
2. Check which beads were in progress
3. Release orphaned file reservations
4. Reassign incomplete beads to new worker

### File Conflict Detected

1. Worker's file reservation fails (conflict response)
2. Worker reports conflict to orchestrator
3. Orchestrator reassigns bead to different track or waits

---

## Progress Display

During execution, maintain a live view:

```markdown
## Execution Progress

**Phase:** 2 of 4
**Status:** Running
**Elapsed:** 12m 34s

### Active Tracks

| Track | Agent | Beads | Progress | Current |
|-------|-------|-------|----------|---------|
| A: Auth | BlueLake | bd-101, bd-102 | 2/2 ✓ | Done |
| B: API | GreenCastle | bd-103, bd-104 | 1/2 | bd-104 |
| C: Models | RedStone | bd-105 | 0/1 | bd-105 |
| D: Tests | PurpleBear | bd-106, bd-107 | 1/2 | bd-107 |

### Completed This Phase
- bd-101: JWT token generation ✓
- bd-102: Token validation ✓
- bd-103: User endpoints ✓
- bd-106: Auth test suite ✓

### Blocked
(none)

### Next
Waiting for Tracks B, C, D...
Then: Calibration → Phase 3
```

---

## Example Session

```bash
# User runs execute
> /execute

## Discovery

Found 24 beads across 4 phases:
- Phase 1: 8 beads (4 tracks)
- Phase 2: 10 beads (5 tracks)
- Phase 3: 4 beads (2 tracks)
- Phase 4: 2 beads (1 track)

## Phase 1: Starting

Spawning 4 workers...
- Track A (Auth): BlueLake → bd-101, bd-102
- Track B (API): GreenCastle → bd-103, bd-104, bd-105
- Track C (Models): RedStone → bd-106
- Track D (Tests): PurpleBear → bd-107, bd-108

Monitoring...

[10:05] BlueLake: [CLAIMED] bd-101
[10:06] GreenCastle: [CLAIMED] bd-103
[10:06] RedStone: [CLAIMED] bd-106
[10:07] PurpleBear: [CLAIMED] bd-107
[10:12] BlueLake: [CLOSED] bd-101
[10:14] BlueLake: [CLAIMED] bd-102
...
[10:45] PurpleBear: [TRACK COMPLETE] Track D

## Phase 1: Calibration

All tracks complete. Running /calibrate...

✓ Coverage: All REQs traced
✓ Drift: None detected
✓ Security: Clean

[PHASE COMPLETE] Phase 1

## Phase 2: Starting

Spawning 5 workers...
...
```

---

## See Also

- `/prime` — Worker startup
- `/next-bead` — Worker task loop (used by workers internally)
- `/calibrate` — Phase boundary check
- `/release` — Post-execution checklist
- `docs/guides/TUTORIAL.md` — Complete tool guide
