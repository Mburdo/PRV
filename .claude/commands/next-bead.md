---
description: Find and safely claim the next Beads task (conflict-checked)
argument-hint: [focus_area]
---

# /next-bead

Run the `next-bead` skill to find and claim your next task.

**Focus:** $ARGUMENTS

Execute the full protocol from `.claude/skills/next-bead/SKILL.md`:
1. Close out previous work (if any)
2. Discover available tasks (bv --robot-triage, bd ready)
3. Verify no conflicts (file reservations, inbox, dependencies)
4. Claim task + reserve files + announce `[CLAIMED]`
