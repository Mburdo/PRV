---
name: project-memory
description: Retrieve relevant context from past sessions before starting implementation. Use when beginning work on a task, when the user describes what to build, when about to write significant code, or when stuck on a problem that may have been solved before.
---

# Project Memory

Get relevant history, rules, and anti-patterns from past sessions before implementing.

---

## When to Retrieve Context

| Signal | Action |
|--------|--------|
| Starting work on a bead/task | Retrieve before coding |
| User describes feature to build | Retrieve matching patterns |
| About to write significant code | Check for existing solutions |
| Stuck on a problem | Search for prior solutions |
| Refactoring existing code | Check why it was built that way |

**Default: Retrieve context before any non-trivial implementation.**

---

## Protocol

```
1. DISTILLED CONTEXT
   cm context "{task}" --json
        ↓
2. REVIEW RESULTS
   Rules, anti-patterns, suggested searches
        ↓
3. DEEP DIVE (if needed)
   cass search "{specific}" --robot
        ↓
4. APPLY
   Follow rules, avoid anti-patterns, reference prior work
```

---

## Step 1: Distilled Context

Always start here:

```bash
cm context "{task_description}" --json
```

**Returns:**
- **Rules** — Distilled patterns from past sessions
- **Anti-patterns** — What NOT to do (and why)
- **Suggested searches** — Specific CASS queries for more detail
- **Historical context** — Related past work

---

## Step 2: Review and Extract

From `cm context` output, extract:

| Category | What to Note |
|----------|--------------|
| **Must-follow rules** | "Always use X", "Never do Y" |
| **Anti-patterns** | Past failures to avoid |
| **Relevant sessions** | Sessions to dig into |
| **Unknowns** | Gaps that need grounding |

---

## Step 3: Deep Dive (Optional)

If `cm context` suggests searches or you need more detail:

```bash
# Specific pattern
cass search "{pattern}" --robot --fields minimal --limit 5

# View a session
cass view /path/to/session.jsonl --json

# Expand around a match
cass expand /path -n {line} -C 3 --json
```

---

## Step 4: Apply Context

Before implementing:

- [ ] Rules noted and will be followed
- [ ] Anti-patterns noted and will be avoided
- [ ] Prior solutions reviewed for reuse
- [ ] Gaps identified for grounding

---

## Query Patterns

### Task-based
```bash
cm context "implement user authentication" --json
cm context "fix pagination bug" --json
cm context "add API endpoint for X" --json
```

### Pattern-based
```bash
cm context "error handling in API routes" --json
cm context "database transaction patterns" --json
cm context "form validation" --json
```

### Problem-based
```bash
cm context "timeout errors in background jobs" --json
cm context "memory leak in long-running process" --json
```

---

## Output Interpretation

### Rules (high signal)
```json
{
  "rules": [
    {"pattern": "auth", "rule": "Always use bcrypt cost >= 12"}
  ]
}
```
→ **Follow these.** They're distilled from successful sessions.

### Anti-patterns (high signal)
```json
{
  "antipatterns": [
    "Don't use localStorage for tokens - session abc failed"
  ]
}
```
→ **Avoid these.** They caused failures before.

### Suggested searches (medium signal)
```json
{
  "suggested_searches": [
    "cass search 'jwt refresh' --robot"
  ]
}
```
→ **Run these** if you need more detail on a specific aspect.

---

## Integration with Workflow

### At session start
```bash
# After claiming a bead
cm context "{bead_title}" --json
```

### Before implementation
```bash
# Before writing significant code
cm context "{what_you're_about_to_build}" --json
```

### When stuck
```bash
# When hitting a wall
cm context "{problem_description}" --json
cass search "{error_message}" --robot
```

---

## See Also

- `deep-dive.md` — CASS search patterns for detailed history
