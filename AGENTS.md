# AGENTS.md

## Project Overview

- **Name**: PRV (Git-native memory layer for AI-assisted development)
- **Language**: Go
- **Key Paths**:
  - `cmd/prv/` - CLI entry point
  - `internal/` - Core packages (gitutil, capture, diff, index, recorder, linker, syncer, installer)
  - `plan_legacy/` - Existing design documentation (reference only)
  - `PLAN/` - Active planning documents

---

## Workflow Baseline (Evidence-Backed)

If this project is using the Knowledge & Vibes workflow, align to:
- `docs/workflow/IDEATION_TO_PRODUCTION.md` (complete pipeline reference)
- `docs/workflow/PROTOCOLS.md` (protocol cards)
- `/calibrate` between phases (hard stop; evidence-grounded plan control)

If those files do not exist in this repo, ask the user where the canonical workflow docs live.

---

## RULE 1: ABSOLUTE (DO NOT EVER VIOLATE THIS)

You may NOT delete any file or directory unless the user explicitly gives the exact command **in this session**.

- This includes files you just created (tests, tmp files, scripts, etc.)
- You do not get to decide that something is "safe" to remove
- If you think something should be removed, stop and ask

Treat "never delete files without permission" as a hard invariant.

---

## Irreversible Git & Filesystem Actions

Absolutely forbidden unless the user gives **exact command and explicit approval** in the same message:

- `git reset --hard`
- `git clean -fd`
- `rm -rf`
- Any command that can delete or overwrite code/data

Rules:

1. If you are not 100% sure what a command will delete, do not run it. Ask first.
2. Prefer safe tools: `git status`, `git diff`, `git stash`, copying to backups.
3. After approval, restate the command verbatim, list what it will affect, wait for confirmation.

---

## Code Editing Discipline

- Do **not** run scripts that bulk-modify code (codemods, one-off scripts, giant sed/regex refactors)
- Large mechanical changes: break into smaller, explicit edits and review diffs
- Subtle/complex changes: edit by hand, file-by-file, with careful reasoning

---

## Configuration Reference

<!-- CUSTOMIZE: Update paths based on what you've configured -->

Your configuration is split across these locations:

| Location | Purpose | When to Use |
|----------|---------|-------------|
| `CLAUDE.md` | Project context, architecture | Understanding the codebase |
| `AGENTS.md` | This file - workflow instructions | Session startup, tool usage |
| `.claude/rules/` | Constraints and conventions | Auto-loaded, always follow |
| `.claude/skills/` | Detailed guides and capabilities | Reference when relevant |
| `.claude/commands/` | Slash commands | Invoke with `/command-name` |

### Rules (Auto-Loaded)

Rules in `.claude/rules/*.md` are automatically enforced:
- `safety.md`: File deletion, destructive commands
- `beads.md`: Issue tracking with bd
- `calibration-protocol.md`: Agent coordination and calibration

### Skills (On-Demand)

Skills in `.claude/skills/*/SKILL.md` provide detailed guidance:
- `bd-triage/`: Beads system triage and prioritization
- `calibration/`: Multi-agent calibration protocol
- `calibration-roundtable/`: Roundtable discussions
- `cass-memory/`: Cross-agent search and memory
- `codemaps-protocol/`: Documentation system
- `discovery/`: Requirements gathering and discovery
- `grounding-protocol/`: Context and search strategy
- `ideation/`: Idea generation and exploration
- `implementation-ticket/`: Ticket implementation workflow
- `next-bead/`: Find and claim next work
- `north-star/`: North Star Card creation
- `prime/`: Agent startup checklist
- `refactoring-protocol/`: Safe refactoring practices
- `risk-assessment/`: Risk and spike analysis
- `ubs/`: Bug scanner usage

### Commands (User-Triggered)

Slash commands in `.claude/commands/*.md`:
- `/prime [task_focus]`: New agent startup checklist
- `/next-bead [focus_area]`: Find/verify/claim next work safely
- `/ground [question-or-task]`: Decide Warp-Grep vs Exa vs CASS/cm
- `/decompose-task [phase]`: Turn a phase into beads and sub-beads
- `/calibrate [phase_or_milestone]`: Pause and realign plan via roundtable
- `/discovery`: Run interactive requirements discovery
- `/north-star`: Generate North Star Card from context

---

## Issue Tracking with bd (Beads)

All issue tracking goes through **bd**. No other TODO systems.

Key invariants:

- `.beads/` is authoritative state and **must always be committed** with code changes
- Do not edit `.beads/*.jsonl` directly; only via `bd`

### Core Commands

```bash
# Finding work
bd ready --json                    # Unblocked tasks ready for work
bd blocked                         # Tasks waiting on dependencies
bd list                            # All tasks

# Task lifecycle
bd create "Title" -t bug -p 1 --estimate 30   # Create with type, priority, estimate
bd update bd-42 --status in_progress --assignee YOUR_AGENT_NAME  # Claim task
bd close bd-42 --reason "Completed"           # Complete task

# Viewing
bd show bd-42                      # Full task details
bd info                            # Project summary
```

### Dependencies

```bash
bd dep add bd-child bd-blocker --type blocks        # Child blocked by blocker
bd dep add bd-a bd-b --type related                 # Related tasks
bd dep add bd-child bd-parent --type parent-child   # Hierarchy
bd dep add bd-new bd-old --type discovered-from     # Found during work
bd dep tree bd-42                                   # Visualize dependencies
bd dep cycles                                       # Find circular deps
```

### Advanced Features

```bash
bd doctor                          # Health check
bd doctor --fix                    # Auto-fix issues
bd compact --analyze --json        # Analyze for compaction
bd --readonly list                 # Safe read-only mode
```

**Types**: `bug`, `feature`, `task`, `epic`, `chore`
**Priority**: `0` (critical) to `4` (backlog)
**Child beads**: `bd-a1b2.1`, `bd-a1b2.3.1` (hierarchical)

### Agent Workflow

1. **Check inbox** for recent `[CLAIMED]` messages from other agents
2. `bd ready --json` to find unblocked work
3. `bv --robot-triage` (or `bv --robot-next`) to pick the best next task deterministically
4. **Claim PARENT bead AND ALL SUB-BEADS together:**
   ```bash
   bd update <id> --status in_progress --assignee YOUR_AGENT_NAME
   bd update <id>.1 --status in_progress --assignee YOUR_AGENT_NAME
   bd update <id>.2 --status in_progress --assignee YOUR_AGENT_NAME
   # ... repeat for ALL sub-beads
   ```
   **WHY:** If you only claim the parent, other agents see sub-beads as "ready" → CONFLICT
5. **Reserve file paths** via `file_reservation_paths()`
6. **Send `[CLAIMED]` message** to all agents
7. Implement + test
8. Discovered work: `bd create "..." && bd dep add <new> <current> --type discovered-from`
9. **Close ALL sub-beads first**, then parent: `bd close <id> --reason "..."`
10. **Send `[CLOSED]` message** to all agents
11. **Release file reservations**
12. Commit `.beads/` in the same commit as code changes: `git add -A && git commit`

**Note:** Beads auto-exports to JSONL after each command. Just include `.beads/` in your normal git commits.

Never:
- Use markdown TODO lists
- Use other trackers
- Duplicate tracking
- Claim only the parent bead (always claim ALL sub-beads)
- Skip `[CLAIMED]`/`[CLOSED]` announcements

---

## Using bv as an AI Sidecar

`bv` is a terminal UI + analysis layer for `.beads/`. It precomputes graph metrics.

**Always use robot flags. Never run bare `bv`.**

```bash
# Session kickoff (default)
bv --robot-next                    # Single best next task (fast)
bv --robot-triage                  # Full triage bundle (blockers, quick wins, commands)

# Multi-agent task partitioning
bv --robot-triage --robot-triage-by-track  # Split work by parallel tracks
bv --robot-plan                            # Track details + what each task unblocks

# Risk & hygiene (use before starting “big” work)
bv --robot-alerts                  # Stale issues, blocking cascades, drift warnings
bv --robot-suggest                 # Duplicates, missing deps, cycle breaks, label fixes

# Debugging regressions / “what changed?”
bv --robot-diff --diff-since HEAD~5
bv --as-of HEAD~10 --robot-triage

# Handoff / audit trail (when needed)
bv --robot-history --bead-history bd-123

# Keep bv current
bv --check-update && bv --update --yes
```

**Graph metrics explained**:
- **PageRank**: Foundational blockers (tasks that enable many others)
- **Betweenness**: Bottlenecks (must pass through these)
- **Critical path**: Longest dependency chain

---

## CASS: Cross-Agent Search

`cass` indexes prior agent conversations so we can reuse solved problems.

**Always use `--robot` or `--json`. Never run bare `cass`.**

### Search

```bash
cass search "query" --robot --limit 5              # Basic search
cass search "query" --robot --fields minimal       # Lean output (path, line, agent)
cass search "query" --robot --fields summary       # Add title, score
cass search "query" --robot --max-tokens 2000      # Token budget
cass search "auth*" --robot                        # Wildcard prefix
```

### View & Expand

```bash
cass view /path/to/session.jsonl --json            # View full session
cass expand /path -n 42 -C 3 --json                # Line 42 with 3 lines context
```

### Timeline & Export

```bash
cass timeline --today --json                       # Today's sessions
cass timeline --since 7d --json                    # Last week
cass export /path/session.jsonl --format markdown  # Export to markdown
```

### Indexing

```bash
cass index --full                  # Rebuild index (if search returns nothing)
```

**Output formats**: `--robot-format jsonl` (streaming), `--robot-format compact` (minimal)

---

## cass-memory: Cross-Agent Learning

Before starting any non-trivial task:

```bash
cm context "your task description" --json
```

This returns:
- **Relevant rules** from the playbook
- **Historical context** from past sessions
- **Anti-patterns** to avoid
- **Suggested searches** for deeper investigation

```bash
cm doctor                          # Health check
```

You do NOT need to:
- Manually run reflection/learning steps
- Manually add rules to the playbook
- Worry about the learning pipeline

The system learns from your sessions automatically.

---

## UBS: Bug Scanner

### Pre-Commit (Required)

```bash
ubs --staged                       # Scan staged changes
ubs --staged --fail-on-warning     # Strict mode (exit 1 on any issue)
```

### Scanning Options

```bash
ubs .                              # Scan current directory
ubs path/to/file.ts                # Scan specific file
ubs --diff                         # Scan working tree changes vs HEAD
ubs -v .                           # Verbose with code examples
```

### Profiles & Filters

```bash
ubs --profile=strict .             # Fail on warnings
ubs --profile=loose .              # Skip nits (prototyping)
ubs --only=python .                # Single language
ubs --only=typescript,javascript . # Multiple languages
```

**Languages**: javascript, typescript, python, c, c++, rust, go, java, ruby

### Output Formats

```bash
ubs . --format=json                # JSON
ubs . --format=jsonl               # Line-delimited JSON
ubs . --format=sarif               # GitHub Code Scanning
```

### CI Integration

```bash
ubs --ci                           # CI mode
ubs --comparison baseline.json .   # Regression detection
```

**Suppress false positives**: `// ubs:ignore`

**Health check**: `ubs doctor --fix`

---

## Multi-Agent Coordination Rules

**These rules are MANDATORY to prevent conflicts in multi-agent environments.**

### Rule 1: Claim ALL Sub-Beads Together

When you claim a parent bead, you MUST claim ALL its sub-beads immediately:

```bash
bd update bd-123 --status in_progress --assignee YOUR_AGENT_NAME
bd update bd-123.1 --status in_progress --assignee YOUR_AGENT_NAME
bd update bd-123.2 --status in_progress --assignee YOUR_AGENT_NAME
bd update bd-123.3 --status in_progress --assignee YOUR_AGENT_NAME
```

**WHY:** If you only claim the parent, another agent sees sub-beads as "ready" and starts working on them → CONFLICT.

### Rule 2: File Reservations for ALL Bead Paths

When you claim a bead, IMMEDIATELY reserve ALL file paths it will touch:

```python
file_reservation_paths(
    project_key, agent_name,
    paths=["src/module/**", "tests/test_module.py"],
    ttl_seconds=3600,
    exclusive=True,
    reason="bd-123: Brief description"
)
```

### Rule 3: Announce When You START

Send a `[CLAIMED]` message when you claim a bead:

```python
send_message(
    project_key, sender_name,
    to=["all"],
    subject="[CLAIMED] bd-123 - Feature Title",
    body_md="Starting work on **bd-123** (plus sub-beads .1, .2, .3).\n\nFile reservations: `src/module/**`",
    thread_id="bd-123"
)
```

### Rule 4: Announce When You FINISH

Send a `[CLOSED]` message when you complete a bead:

```python
send_message(
    project_key, sender_name,
    to=["all"],
    subject="[CLOSED] bd-123 - Feature Title",
    body_md="Completed **bd-123**.\n\nFiles created: ...\nTests: X passing\n\nReleasing file reservations.",
    thread_id="bd-123"
)
```

### Rule 5: Check Inbox BEFORE Claiming

Before running `bd ready`, check your inbox for recent `[CLAIMED]` messages.

---

## Bead Claiming Checklist

```
□ 1. Check inbox for recent [CLAIMED] messages
□ 2. Run `bd ready --json` to find unblocked work
□ 3. Run `bv --robot-next` (or `bv --robot-triage`) to pick the best next task
□ 4. Check current file reservations (avoid conflicts)
□ 5. Claim PARENT bead: `bd update <id> --status in_progress --assignee YOUR_NAME`
□ 6. Claim ALL SUB-BEADS: `bd update <id.1> --status in_progress --assignee YOUR_NAME` (repeat for all)
□ 7. Reserve ALL file paths you will touch (exclusive when appropriate)
□ 8. Send `[CLAIMED]` message (use `thread_id="<id>"`, list reserved paths)
□ 9. Work on the bead (keep updates in-thread)
```

## Bead Finish Checklist

```
□ 1. Run tests / builds relevant to your change
□ 2. Run `ubs --staged` (fix issues in files you touched; rerun until clean)
□ 3. Commit your work (include `.beads/issues.jsonl`): `git add -A && git commit`
□ 4. Close ALL sub-beads first: `bd close <id>.1 ...` (repeat for all)
□ 5. Close the parent bead: `bd close <id> --reason "Completed: ..."`
□ 6. Release file reservations
□ 7. Send `[CLOSED]` message in the same thread (what changed, tests run, reservations released)
□ 8. `git push`
```

**Note:** Beads auto-exports to JSONL. Share via normal git commits (include `.beads/issues.jsonl` with code changes).

---

## MCP Agent Mail: Multi-Agent Coordination

Agent Mail is available as an MCP server for coordinating multiple agents.

### Registration (Required First)

```python
ensure_project(human_key="/abs/path/to/project")
register_agent(project_key="/abs/path/to/project", program="claude-code", model="opus-4.5")
# Returns auto-generated name like "GreenCastle"
```

### File Reservations

```python
# Reserve before editing
file_reservation_paths(
    project_key, agent_name,
    paths=["src/**/*.ts"],
    ttl_seconds=3600,
    exclusive=True,
    reason="bd-123"
)

# Extend if needed
renew_file_reservations(project_key, agent_name, extend_seconds=1800)

# Release when done
release_file_reservations(project_key, agent_name)
```

### Messaging

```python
# Send
send_message(
    project_key, sender_name,
    to=["OtherAgent"],
    subject="[bd-123] Starting auth refactor",
    body_md="Working on login module...",
    thread_id="bd-123",
    importance="normal"  # low, normal, high, urgent
)

# Reply
reply_message(project_key, message_id, sender_name, body_md="Done!")

# Check inbox
fetch_inbox(project_key, agent_name, limit=10)
fetch_inbox(project_key, agent_name, urgent_only=True)

# Acknowledge
acknowledge_message(project_key, agent_name, message_id)
```

### Search & Discovery

```python
# Full-text search
search_messages(project_key, query="authentication", limit=20)

# Thread summary
summarize_thread(project_key, thread_id="bd-123")

# Who is this agent?
whois(project_key, agent_name="BlueLake")
```

### Build Coordination

```python
# Acquire build slot (prevents concurrent builds)
acquire_build_slot(project_key, agent_name, slot="main", exclusive=True)
release_build_slot(project_key, agent_name, slot="main")
```

### Quick Start Macros

```python
# Start session with reservation in one call
macro_start_session(
    human_key="/abs/path",
    program="claude-code",
    model="opus-4.5",
    file_reservation_paths=["src/**"],
    inbox_limit=10
)
```

Common pitfalls:
- "from_agent not registered" → call `register_agent` first
- `FILE_RESERVATION_CONFLICT` → wait for expiry or coordinate with holder

---

## Warp-Grep: Parallel Code Search

Warp-Grep runs 8 parallel searches per turn. It's an MCP tool, not a CLI command.

**When to use**:
- "How does X work?" discovery
- Data flow across multiple files
- Cross-cutting concerns

**When NOT to use**:
- You know the function name (use `rg` or Grep tool)
- You know the exact file (just open it)
- Simple existence check

The tool activates automatically for natural language code questions.

---

## Exa MCP: AI Web & Code Search

Exa provides real-time web search and code context retrieval.

**When to use**:
- Current documentation (APIs change after training cutoff)
- Code examples from GitHub/StackOverflow
- Latest library versions or deprecation notices

**Available tools**:
```
web_search_exa        # Real-time web search
get_code_context_exa  # Search GitHub, docs, StackOverflow
deep_search_exa       # Deep research with query expansion
crawling              # Extract content from specific URLs
```

**When NOT to use**:
- Information likely in codebase (use CASS or Warp-Grep)
- Historical context (use cass-memory)
- Task information (use Beads)

---

## Session Workflow

### Session Naming (Claude Code 2.0.64+)

Name sessions immediately after Agent Mail registration for traceability:

```
{project}-{AgentMailName}-{YYYYMMDD-HHMMSS}
```

Examples:
- `myapp-GreenCastle-20251210-143022`
- `backend-BlueLake-20251210-091500`

After `register_agent` or `macro_start_session` returns your agent name:

```bash
/rename myapp-GreenCastle-20251210-143022
```

To resume later:

```bash
claude --resume myapp-GreenCastle-20251210-143022
# Or use /resume in REPL
```

### Start

```bash
bd ready --json                    # Find work
bd update <id> --status in_progress --assignee YOUR_AGENT_NAME
cm context "task description" --json
```

### During Work

```bash
# If stuck
cass search "similar problem" --robot
bv --robot-triage
# or: bv --robot-next

# If blocked on your current bead
bv --robot-blocker-chain <id>

# Before a risky refactor (understand coupling/impact)
bv --robot-impact path/to/file1,path/to/file2
bv --robot-file-relations path/to/core_file

# If multi-agent
fetch_inbox(project_key, agent_name)
```

### End

```bash
ubs --staged                       # Scan for bugs
bd close <id> --reason "Completed: ..."
git add -A && git commit && git push   # .beads/ included automatically
release_file_reservations(...)     # If multi-agent
```

**Note:** Commit/push your changes normally (include `.beads/issues.jsonl` with code changes).

---

## PRV-Specific Guidelines

### Build & Test Commands

```bash
# Module init (one-time)
go mod init github.com/Mburdo/prv

# Build
go build ./...

# Test
go test ./...
go test -v ./internal/...      # Verbose tests
go test -cover ./...           # With coverage
go test -race ./...            # Race detection

# Run
go run ./cmd/prv ...

# Lint (install: go install github.com/golangci/golangci-lint/cmd/golangci-lint@latest)
golangci-lint run
```

### Coding Conventions

- **Package structure**: Follow `cmd/` + `internal/` layout (see `plan_legacy/05_PART1_DESIGN_DECISIONS.md`)
- **Testing**: TDD required - write tests first (see `plan_legacy/06_PART2_TDD_PROTOCOL.md`)
- **Error handling**: Return errors, don't panic; wrap with context using `fmt.Errorf("...: %w", err)`
- **File permissions**: Drafts/Links use `0600`, hooks use `0755`
- **Immutability**: Draft and Link files are append-only, never edited after creation

### What NOT To Do

- Do NOT auto-share transcripts (security risk - may contain secrets)
- Do NOT block git operations (hooks must soft-fail with exit 0)
- Do NOT edit `.prv/drafts/` or `.prv/links/` files after creation
- Do NOT guess hunk fingerprinting logic - consult the ADRs in plan_legacy/
- Do NOT implement PTY capture for Windows (pipes only - ADR-003)

### Key Concepts

- **Draft**: Immutable record of one AI tool session
- **Link**: Immutable mapping from commit SHA → Draft IDs
- **Index**: Fast lookup map from hunk hash → Draft IDs (rebuildable)
- **prv-memory**: Orphan branch for sharing Drafts/Links via Git

### Reference Documents

The `plan_legacy/` directory contains detailed design docs:
- `00_INDEX.md` - Document index
- `01_PART1_OVERVIEW.md` - Architecture overview
- `02-04_PART1_PACKAGES_*.md` - Package specifications
- `05_PART1_DESIGN_DECISIONS.md` - ADRs (decisions with rationale)
- `06-14_PART2_*` - Sprint planning and TDD specs

---

## Contribution Policy

This is a personal project. External contributions not expected.
