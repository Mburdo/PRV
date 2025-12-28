---
name: exa-search
description: Web and code search with Exa MCP. Use for current documentation, API references, code examples, latest library info, or when the user mentions "exa", "web search", "docs", or "current API".
---

# Exa (MCP Server)

Real-time web and code search for current documentation and examples.

## When This Applies

| Signal | Action |
|--------|--------|
| Need current API docs | Use Exa |
| Library/framework documentation | Use Exa |
| Code examples from GitHub | `get_code_context_exa` |
| Latest versions/deprecations | Use Exa |
| Deep research | `deep_search_exa` |

---

## Available Tools

```python
web_search_exa        # Real-time web search
get_code_context_exa  # Search GitHub, docs, StackOverflow
deep_search_exa       # Deep research with query expansion
crawling              # Extract content from specific URLs
```

---

## Query Patterns

### Current Documentation

```
{library} {feature} {version} 2024 2025
```

**Examples:**
```
FastAPI Pydantic v2 model_validator 2024 2025
Next.js 14 app router server components
React useOptimistic hook 2024
```

### Migration Between Versions

```
{library} v{old} to v{new} migration
{library} {version} breaking changes
{library} upgrade guide {version}
```

### Code Examples

```python
get_code_context_exa("{library} {pattern} implementation example")
get_code_context_exa("{library} {use_case} tutorial")
```

### Security/Auth Patterns

```
{auth_method} best practices 2024
{library} authentication {pattern} security
OAuth PKCE {language} 2024
```

---

## When to Use Exa

| Use Case | Use Exa? |
|----------|---------|
| Current API documentation | **YES** |
| Latest library changes | **YES** |
| Code examples | **YES** |
| Security best practices | **YES** |
| Deprecation notices | **YES** |

---

## When NOT to Use Exa

| Use Case | Use Instead |
|----------|-------------|
| Information in codebase | Warp-Grep or Grep |
| Past session content | CASS |
| Historical context | cass-memory (`cm`) |
| Task information | Beads (`bd`, `bv`) |

---

## Query Strengthening

If initial query returns poor results:

1. **Add version**: "React 19 useOptimistic" vs "React useOptimistic"
2. **Add year**: "FastAPI middleware 2024" vs "FastAPI middleware"
3. **Add "official"**: "Next.js official docs app router"
4. **Be more specific**: "Prisma findMany where clause" vs "Prisma queries"

---

## Verification Checklist

After grounding with Exa:

| Check | Pass If |
|-------|---------|
| Source | Official docs or reputable repo |
| Freshness | Updated within 12 months |
| Version | Matches your dependency |
| Completeness | Full import + usage pattern |
| Status | Not deprecated |

---

## Recording Results

Track grounding status in your work:

```markdown
## Grounding Status
| Pattern | Query | Source | Status |
|---------|-------|--------|--------|
| `@model_validator` | "Pydantic v2 2024" | docs.pydantic.dev | ✅ Verified |
| `useOptimistic` | "React 19 2024" | react.dev | ✅ Verified |
```

**Status values:**
- ✅ Verified — Matches current docs
- ⚠️ Changed — API changed, updated approach
- ❌ Deprecated — Found alternative
- ❓ Unverified — Couldn't confirm, flagged

---

## Requirements

Requires Exa API key configured:

```bash
claude mcp add exa -s user \
  -e EXA_API_KEY=your-key \
  -- npx -y @anthropic-labs/exa-mcp-server
```

---

## See Also

- `external-docs/` — Full grounding skill with patterns
- `warp-grep/` — Codebase search
- `cass-search/` — Past session search
