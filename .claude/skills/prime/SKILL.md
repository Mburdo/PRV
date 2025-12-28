---
name: prime
description: New agent startup checklist for Agent Mail and Beads. Use when starting a new agent session, when the user says "prime" or "startup", or when beginning work on a multi-agent project.
---

# Prime — Orchestrator

Complete startup checklist for new agent sessions.

> **Pattern:** This skill uses the orchestrator-subagent pattern. Each phase runs in a fresh context for optimal multi-agent coordination. See `docs/guides/ORCHESTRATOR_SUBAGENT_PATTERN.md`.

## When This Applies

| Signal | Action |
|--------|--------|
| New session starting | Run full checklist |
| User says "/prime" | Run full checklist |
| User says "startup" or "initialize" | Run full checklist |
| Multi-agent coordination needed | Run full checklist |

---

## Tool Reference

### Agent Mail (MCP)
| Tool | Purpose |
|------|---------|
| `macro_start_session(human_key, program, model)` | **PREFERRED:** All-in-one startup |
| `ensure_project(human_key)` | Initialize project in Agent Mail |
| `register_agent(project_key, program, model)` | Register and get agent name |
| `set_contact_policy(agent_name, policy)` | Set to "open" for coordination |
| `fetch_inbox(agent_name, limit)` | Check for messages |
| `send_message(to, subject, body_md)` | Send greeting/announcements |
| `file_reservation_paths(paths, exclusive)` | Reserve files before editing |

### Bash Commands
| Command | Purpose |
|---------|---------|
| `cm context "task description" --json` | Get patterns + anti-patterns before work |
| `cass search "query" --robot --days 7` | Search session history |
| `bd ready --json` | List available tasks |
| `bv --robot-triage` | Get task recommendations |
| `bd update <id> --status in_progress --assignee NAME` | Claim task |
| `git status` | Check repo state |
| `git log --oneline -5` | Recent commits |

### Message Subjects
| Pattern | When |
|---------|------|
| `Agent Online` | After registration |
| `[CLAIMED] bd-XXX - Title` | When claiming task |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      PRIME ORCHESTRATOR                          │
│  - Creates session: sessions/prime-{timestamp}/                  │
│  - Manages TodoWrite state                                       │
│  - Spawns subagents with minimal context                         │
│  - Passes agent_name between phases                              │
└─────────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│    Identity     │  │     Orient      │  │   Coordinate    │
│  agents/        │  │  agents/        │  │  agents/        │
│  identity.md    │  │  orient.md      │  │  coordinate.md  │
└────────┬────────┘  └────────┬────────┘  └────────┬────────┘
         │                    │                    │
    01_identity.md       02_orientation.md   03_coordination.md
         │                    │                    │
         │      agent_name    │    active_agents   │    blocked_files
         └────────────────────┼────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │    Discover     │ → Recommends task
                    │  agents/        │
                    │  discover.md    │
                    └────────┬────────┘
                             │
                        04_discovery.md
```

## Subagents

| Phase | Agent | Input | Output |
|-------|-------|-------|--------|
| 1 | `agents/identity.md` | project_path | agent_name, registered |
| 2 | `agents/orient.md` | project_path, agent_name | context, active_agents |
| 3 | `agents/coordinate.md` | agent_name, active_agents | inbox, blocked_files |
| 4 | `agents/discover.md` | agent_name, blocked_files | recommended_task |

---

## Execution Flow

### 1. Setup (Orchestrator)

```markdown
1. Create session directory:
   mkdir -p sessions/prime-{timestamp}

2. Initialize TodoWrite with phases:
   - [ ] Phase 1: Identity
   - [ ] Phase 2: Orient
   - [ ] Phase 3: Coordinate
   - [ ] Phase 4: Discover

3. Gather inputs:
   - project_path: Absolute path to project
   - program: "claude-code"
   - model: "opus-4.5"
```

### 2. Phase 1: Identity

**Spawn:** `agents/identity.md`

**Input:**
```json
{
  "project_path": "/abs/path/to/project",
  "session_dir": "sessions/prime-{timestamp}",
  "program": "claude-code",
  "model": "opus-4.5"
}
```

**Output:**
```json
{
  "agent_name": "BlueLake",
  "registered": true
}
```

### 3. Phase 2: Orient

**Spawn:** `agents/orient.md`

**Input:**
```json
{
  "project_path": "/abs/path/to/project",
  "session_dir": "sessions/prime-{timestamp}",
  "agent_name": "BlueLake"
}
```

**Output:**
```json
{
  "active_agents": ["GreenCastle", "RedStone"],
  "recent_sessions_summary": "Auth module, rate limiting",
  "git_branch": "feature/user-auth"
}
```

### 4. Phase 3: Coordinate

**Spawn:** `agents/coordinate.md`

**Input:**
```json
{
  "project_path": "/abs/path/to/project",
  "session_dir": "sessions/prime-{timestamp}",
  "agent_name": "BlueLake",
  "active_agents": ["GreenCastle", "RedStone"]
}
```

**Output:**
```json
{
  "urgent_messages": 0,
  "blocked_files": ["src/api/**", "migrations/**"],
  "greeting_sent": true
}
```

### 5. Phase 4: Discover

**Spawn:** `agents/discover.md`

**Input:**
```json
{
  "project_path": "/abs/path/to/project",
  "session_dir": "sessions/prime-{timestamp}",
  "agent_name": "BlueLake",
  "blocked_files": ["src/api/**", "migrations/**"],
  "requires_response": []
}
```

**Output:**
```json
{
  "recommended_task": {
    "id": "bd-125",
    "title": "JWT validation",
    "files": ["src/auth/**"]
  },
  "can_claim": true
}
```

### 6. Finalize (Orchestrator)

1. Update TodoWrite (all phases complete)
2. Present startup summary to user
3. If task recommended and user approves, proceed to claim

---

## Configuration

Set these for your project:

```
PROJECT_PATH="/abs/path/to/your/project"  # Agent Mail project_key
PROJECT_NAME="Your Project Name"
```

If working across repos, use ONE canonical "coordination root" for Agent Mail + Beads.

---

## Step 1: Announce Identity

```bash
echo -ne "\033]0;[AgentName] | PROJECT_NAME\007"
```

Update after registration with your actual name.

---

## Step 2: Register with Agent Mail

**PREFERRED: Use macro_start_session for simplicity:**

```python
result = macro_start_session(
  human_key="PROJECT_PATH",
  program="claude-code",
  model="opus-4.5",
  file_reservation_paths=["src/**"],  # Optional: reserve files upfront
  inbox_limit=10
)
agent_name = result["agent"]["name"]
# Includes: project, agent_name, file_reservations, inbox
```

**ALTERNATIVE: Manual steps:**

```python
ensure_project(human_key="PROJECT_PATH")

register_agent(
  project_key="PROJECT_PATH",
  program="claude-code",
  model="opus-4.5",
  task_description="Agent - [your focus]"
)
# SAVE YOUR AGENT NAME from response (e.g., "BlueLake")

set_contact_policy(
  project_key="PROJECT_PATH",
  agent_name="YOUR_AGENT_NAME",
  policy="open"
)
```

Update terminal:
```bash
echo -ne "\033]0;[YOUR_AGENT_NAME] | PROJECT_NAME\007"
```

---

## Step 3: Orient

Read AGENTS.md:
```
Read("AGENTS.md")
```

Check recent session history:
```bash
cass search "PROJECT_NAME" --workspace "PROJECT_PATH" --robot --days 7 --limit 5
```

Get learned patterns for context:
```bash
cm context "starting work on PROJECT_NAME" --json
```

This returns relevant rules, anti-patterns, and historical context to inform your work.

---

## Step 4: Coordinate

Check inbox:
```python
fetch_inbox(
  project_key="PROJECT_PATH",
  agent_name="YOUR_AGENT_NAME",
  include_bodies=true,
  limit=20
)
```

Discover other agents:
```python
ReadMcpResourceTool(
  server="mcp-agent-mail",
  uri="resource://agents/PROJECT_PATH"
)
```

Send greeting:
```python
send_message(
  project_key="PROJECT_PATH",
  sender_name="YOUR_AGENT_NAME",
  to=["OtherAgent1", "OtherAgent2"],
  subject="Agent Online",
  body_md="I'm online. Focusing on: [task]. What needs attention?",
  importance="normal"
)
```

---

## Step 5: Discover Work

```bash
git status
git log --oneline -5
bd ready --json
bv --robot-triage
```

For multi-agent, use track grouping:
```bash
bv --robot-triage --robot-triage-by-track
```

---

## Step 6: Claim Task

```bash
bd update <task-id> --status in_progress --assignee YOUR_AGENT_NAME
```

Reserve files:
```python
file_reservation_paths(
  project_key="PROJECT_PATH",
  agent_name="YOUR_AGENT_NAME",
  paths=["app/services/**"],
  ttl_seconds=3600,
  exclusive=true,
  reason="<task-id>: brief description"
)
```

---

## Step 7: Output Summary

```markdown
## Agent Initialized

**Name:** [YOUR_AGENT_NAME]
**Focus:** [task description]
**Repo:** PROJECT_NAME

### Context
- Recent sessions: [summary from CASS]
- Inbox: [X messages, Y requiring response]
- Active agents: [list]

### Task Claimed
- **ID:** [task-id]
- **Title:** [task title]
- **Files reserved:** [list]

### Ready to Work
[State your plan for the task]
```

---

## After Priming

When done or switching tasks:
1. Run tests / builds
2. **Run `ubs --staged` (MANDATORY GATE)**
   - Zero high/critical findings required
   - Medium findings require documented justification
   - This is NOT optional—~40% of LLM code has vulnerabilities
3. Commit (include `.beads/issues.jsonl`) and push
4. Release file reservations
5. Close sub-beads first, then parent
6. Run `/next-bead` for next task

---

## Mandatory Rules (2025 Research)

| Rule | Why | Evidence |
|------|-----|----------|
| **TDD-first** | 45.97% higher success | `research/054-tdd-ai-code-gen.md` |
| **`ubs --staged` mandatory** | ~40% of LLM code has vulnerabilities | `research/052-llm-security-vulnerabilities.md` |
| **Max 3 iterations** | Security degrades with more | `research/053-feedback-loop-security.md` |
| **Escalate after 3 failures** | Don't waste time debugging | Stop, spike, notify operator |

See `bead-workflow/` for full details on these gates.

---

## See Also

- `bead-workflow/` — Claiming and closing beads
- `project-memory/` — Past session context
