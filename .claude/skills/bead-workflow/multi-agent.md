# Multi-Agent Coordination

Extra protocols when multiple agents are active.

---

## Detecting Multi-Agent

```python
# Check who's active
ReadMcpResourceTool(
    server="mcp-agent-mail",
    uri="resource://agents/{project_path}"
)
```

If other agents are active, follow all protocols strictly.

---

## File Reservations

### Before Editing

```python
file_reservation_paths(
    project_key=PROJECT_PATH,
    agent_name=YOUR_NAME,
    paths=["src/module/**"],
    ttl_seconds=3600,
    exclusive=True,
    reason="{bead-id}"
)
```

### Check for Conflicts

If reservation fails, another agent has those files. Options:
1. Pick different task
2. Coordinate via message
3. Wait for TTL expiry

### Extend If Needed

```python
renew_file_reservations(
    project_key=PROJECT_PATH,
    agent_name=YOUR_NAME,
    extend_seconds=1800
)
```

### Release When Done

```python
release_file_reservations(
    project_key=PROJECT_PATH,
    agent_name=YOUR_NAME
)
```

---

## Inbox Protocol

### Check Before Claiming

```python
fetch_inbox(
    project_key=PROJECT_PATH,
    agent_name=YOUR_NAME,
    limit=10
)
```

Look for:
- Recent [CLAIMED] messages (avoid that task)
- Requests from other agents
- Urgent coordination messages

### Check Periodically

While working, check inbox every ~30 minutes for:
- API changes from other agents
- Coordination requests
- Blockers discovered

---

## Announcements

### [CLAIMED] — When Starting

```python
send_message(
    project_key=PROJECT_PATH,
    sender_name=YOUR_NAME,
    to=[ALL_AGENTS],
    subject="[CLAIMED] {id} - {title}",
    body_md="...",
    thread_id="{id}"
)
```

### [CLOSED] — When Finishing

```python
send_message(
    project_key=PROJECT_PATH,
    sender_name=YOUR_NAME,
    to=[ALL_AGENTS],
    subject="[CLOSED] {id} - {title}",
    body_md="...",
    thread_id="{id}"
)
```

### Mid-Work Updates

If you make changes that affect others:

```python
send_message(
    ...
    subject="[UPDATE] Changed API in {module}",
    body_md="Updated method signature. If you're using X, update to Y.",
    importance="high"
)
```

---

## Conflict Resolution

| Conflict | Resolution |
|----------|------------|
| Both claimed same task | First [CLAIMED] wins, other picks new task |
| Both editing same file | Reservation holder wins, other waits or coordinates |
| Dependency disagreement | Discuss in thread, escalate to user if stuck |

---

## Thread IDs

Use bead ID as thread ID for all related messages:

```
Bead:           bd-123
Thread ID:      bd-123
Subject prefix: [bd-123]
Reservation:    reason="bd-123"
Commit:         Closes bd-123
```

This links everything together for easy tracking.
