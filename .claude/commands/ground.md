---
description: Manually trigger grounding check before implementation
argument-hint: [question-or-task]
---

# /ground

Run the `external-docs` skill to verify external dependencies.

**Query:** $ARGUMENTS

Execute the grounding protocol from `.claude/skills/external-docs/SKILL.md`:

1. Determine where truth lives:
   - Codebase → Warp-Grep
   - Web → Exa (web_search_exa, get_code_context_exa)
   - History → `cm context` → `cass search`
   - Tasks → `bv --robot-*`

2. Run appropriate queries with recency signals (2024 2025)

3. Record grounding status table:
   ```markdown
   ## Grounding Status
   | Pattern | Query | Source | Status |
   |---------|-------|--------|--------|
   | `{method}` | "{query}" | {url} | ✅/⚠️/❌/❓ |
   ```
