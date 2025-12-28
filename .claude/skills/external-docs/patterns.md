# Grounding Patterns

How to ground for any framework or library.

---

## What Categories Need Grounding

| Category | Why | Query Pattern |
|----------|-----|---------------|
| **Imports/Initialization** | Syntax changes between versions | `{lib} import {feature} {version}` |
| **API Methods** | Methods get renamed/deprecated | `{lib} {method} {version} 2024` |
| **Configuration** | Options/flags evolve | `{lib} config {option} {version}` |
| **Async Patterns** | Async APIs vary significantly | `{lib} async {pattern} {version}` |
| **Auth/Security** | Security best practices change | `{lib} {auth_pattern} security 2024` |
| **Data Validation** | Validators/schemas evolve | `{lib} validation {pattern} {version}` |

---

## Query Construction Formula

```
{library_name} {specific_feature} {version_if_known} {year}
```

**Strengthen queries by adding:**
- Version number if you know it
- Year for recency (2024, 2025)
- "current" or "latest" for edge cases
- "migration" if moving between versions

---

## Grounding Depth Levels

| Depth | When | What to Check |
|-------|------|---------------|
| **Quick** | Familiar pattern, just confirming | One query, verify method exists |
| **Standard** | Normal implementation | Query + check for deprecation warnings |
| **Deep** | Security/auth, new library, major version | Multiple queries, read changelog, check issues |

---

## Version Sensitivity Signals

Ground more carefully when you see:

| Signal | Risk |
|--------|------|
| Major version in deps (v1 → v2) | Breaking changes likely |
| Library < 2 years old | API still evolving |
| "experimental" or "beta" in docs | May change without notice |
| Security-related code | Best practices evolve |
| AI training data gap | Libs released after training cutoff |

---

## Recording Template

After grounding, record in your bead:

```markdown
## Grounding Status
| Pattern | Query Used | Source | Status |
|---------|------------|--------|--------|
| `{what}` | "{query}" | {url} | {status} |
```

**Status values:**
- ✅ Verified — Matches current docs
- ⚠️ Changed — API changed, updated approach
- ❌ Deprecated — Found alternative
- ❓ Unverified — Couldn't confirm, flagged

---

## Common Grounding Failures

| Failure | Cause | Fix |
|---------|-------|-----|
| No results | Query too specific | Broaden: remove version, use lib name only |
| Outdated results | No recency signal | Add year: "2024 2025" |
| Conflicting info | Multiple versions | Specify exact version |
| Tutorial vs docs | Tutorial may be old | Prefer official docs |

---

## Progressive Grounding

For large implementations:

1. **Start**: Ground the core imports/setup
2. **As you go**: Ground each new external method before using
3. **Before commit**: Review grounding table, verify nothing ❓

Don't try to ground everything upfront — ground just-in-time as you encounter external deps.
