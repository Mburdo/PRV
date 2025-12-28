---
name: cass-memory
description: Cross-agent learning with cm (cass-memory). Use before starting non-trivial tasks, when looking for patterns from past work, when the user mentions "cm", "memory", "learned rules", or "what do we know about".
---

# cass-memory (cm)

Cross-agent learning system. Extracts patterns, rules, and anti-patterns from past sessions.

## When This Applies

| Signal | Action |
|--------|--------|
| Starting non-trivial task | `cm context` |
| "What do we know about X?" | `cm context` |
| Looking for patterns | `cm context` |
| Anti-patterns to avoid | `cm context` |
| Health check | `cm doctor` |

---

## Primary Command

Before starting any non-trivial task:

```bash
cm context "your task description" --json
```

This returns:
- **Relevant rules** from the playbook
- **Historical context** from past sessions
- **Anti-patterns** to avoid
- **Suggested searches** for deeper investigation

---

## Example Usage

```bash
# Before implementing authentication
cm context "implement OAuth login" --json

# Before database work
cm context "add new database migration" --json

# Before refactoring
cm context "refactor the payment module" --json
```

---

## What You Get Back

```json
{
  "rules": ["Always use parameterized queries", "..."],
  "context": ["Previous auth implementation used JWT...", "..."],
  "anti_patterns": ["Don't store tokens in localStorage", "..."],
  "suggested_searches": ["cass search 'OAuth implementation' --robot"]
}
```

---

## Health Check

```bash
cm doctor
```

---

## How It Works

The system learns automatically from your sessions:
1. Sessions are indexed by CASS
2. `cm` extracts patterns, rules, and anti-patterns
3. `cm context` retrieves relevant learned knowledge

**You don't need to:**
- Manually run reflection/learning steps
- Manually add rules to the playbook
- Worry about the learning pipeline

---

## When to Use cm vs CASS

| Need | Use |
|------|-----|
| Learned patterns and rules | `cm context` |
| Raw session content | `cass search` |
| Specific session details | `cass view` / `cass expand` |

---

## Quick Reference

```bash
cm context "task description" --json   # Get context before task
cm doctor                              # Health check
```

---

## Best Practices

| Practice | Why |
|----------|-----|
| Query before non-trivial work | Get patterns + anti-patterns |
| Be specific in task description | Better relevance |
| Check suggested searches | May reveal useful history |

---

## See Also

- `cass-search/` — Raw session search with CASS
- `project-memory/` — Session context retrieval skill
