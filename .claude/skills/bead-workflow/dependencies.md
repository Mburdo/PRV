# Dependency Management

How to work with bead dependencies.

---

## Dependency Types

| Type | Meaning | Example |
|------|---------|---------|
| `blocks` | A must complete before B can start | Schema blocks implementation |
| `discovered-from` | Found B while working on A | Bug found during feature work |

---

## Adding Dependencies

```bash
# B is blocked by A
bd dep add {child-id} {blocker-id} --type blocks

# Found new issue while working
bd dep add {new-id} {current-id} --type discovered-from
```

---

## Viewing Dependencies

```bash
# Tree view
bd dep tree {id}

# What's blocked?
bd blocked

# Full graph analysis
bv --robot-triage
```

---

## Common Patterns

### Schema → Implementation → Tests

```bash
bd dep add impl-id schema-id --type blocks
bd dep add test-id impl-id --type blocks
```

### Parent → Sub-beads

Sub-beads implicitly block parent completion. Close all sub-beads before closing parent.

### Discovered Issues

```bash
# While working on feature-123, found bug
bd create "Found: edge case in validation" -t bug
bd dep add {new-bug-id} feature-123 --type discovered-from
```

---

## Checking Before Claiming

```bash
# Is this task actually ready?
bd show {id}  # Check dependencies

# What's blocking the graph?
bv --robot-blocker-chain {id}

# What becomes ready if I complete this?
bv --robot-triage
```

---

## Cycle Prevention

Dependencies must be acyclic. If you get a cycle error:

```bash
# Check for cycles
bv --robot-suggest

# The tool will recommend which dep to remove
```
