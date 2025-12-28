---
description: Decompose Task - Turn a Phase Plan into Beads
argument-hint: <phase-description-or-id>
---

# /decompose-task

Run the `decompose-task` skill to break down a phase into beads.

**Phase:** $ARGUMENTS

Execute the full protocol from `.claude/skills/decompose-task/SKILL.md`:
1. Understand the phase (goal, files, dependencies, deliverables)
2. Create content manifest (list everything that must be captured)
3. Create beads (parent + sub-beads, LOSSLESS, full code)
4. Set dependencies (schema → implementation → tests)
5. Validate with `bv --robot-suggest`, `bv --robot-plan`
