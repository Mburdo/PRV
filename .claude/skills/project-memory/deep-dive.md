# Deep Dive with CASS

When `cm context` isn't enough, dig into specific sessions.

---

## CASS Search Patterns

### Basic Search
```bash
cass search "{query}" --robot --limit 5
```

### Lean Output (less tokens)
```bash
cass search "{query}" --robot --fields minimal --limit 5
```

### Token-Budgeted
```bash
cass search "{query}" --robot --max-tokens 2000
```

---

## Search Query Patterns

### By Feature
```bash
cass search "authentication implementation" --robot
cass search "payment processing" --robot
cass search "file upload handling" --robot
```

### By Problem
```bash
cass search "timeout error" --robot
cass search "memory leak" --robot
cass search "race condition" --robot
```

### By Error Message
```bash
cass search "TypeError: Cannot read property" --robot
cass search "ECONNREFUSED" --robot
```

### By Library/Tool
```bash
cass search "prisma migration" --robot
cass search "react hooks" --robot
cass search "fastapi dependency" --robot
```

---

## Viewing Sessions

### View full session
```bash
cass view /path/to/session.jsonl --json
```

### View today's sessions
```bash
cass timeline --today --json
```

### View recent activity
```bash
cass timeline --since 7d --json
```

---

## Expanding Context

When you find a relevant match, expand around it:

```bash
# Line 42 with 3 lines of context
cass expand /path/session.jsonl -n 42 -C 3 --json
```

---

## Export for Reference

```bash
# Export to markdown for reading
cass export /path/session.jsonl --format markdown -o reference.md
```

---

## Search Refinement

| If results are... | Try... |
|-------------------|--------|
| Too broad | Add specific terms, use quotes |
| Too narrow | Remove terms, try synonyms |
| Wrong domain | Add project/feature qualifiers |
| Too old | Use `--since` flag |

---

## When to Use CASS vs cm

| Situation | Use |
|-----------|-----|
| Starting a task | `cm context` first |
| Need specific solution | `cass search` |
| Following up on cm suggestion | `cass search` with suggested query |
| Want full conversation | `cass view` |
| Debugging similar past issue | `cass search` + `cass expand` |

---

## Output Formats

```bash
--robot              # Default structured output
--robot-format jsonl # Streaming format
--robot-format compact # Minimal single-line JSON
--fields minimal     # Reduce output size
--fields summary     # Just summaries
```
