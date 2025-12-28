---
description: New agent startup checklist (Agent Mail + Beads)
argument-hint: [task_focus]
---

# /prime

Run the `prime` skill to initialize a new agent session.

**Focus:** $ARGUMENTS

Execute the full startup protocol from `.claude/skills/prime/SKILL.md`:
1. Announce identity (terminal title + banner)
2. Register with Agent Mail (ensure_project + register_agent + set_contact_policy)
3. Orient (read AGENTS.md, check CASS for recent history)
4. Coordinate (check inbox, discover agents, send greeting)
5. Discover work (git state, ready tasks, bv recommendations)
6. Claim a task (reserve files, announce)
7. Output startup summary
