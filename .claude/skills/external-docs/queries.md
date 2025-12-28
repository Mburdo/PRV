# Exa Query Examples

Query patterns that work well for grounding.

---

## Query Formula

```
{library} {feature} {version} 2024 2025
```

Add recency signals (2024, 2025) to surface current documentation.

---

## By Intent

### Current API Documentation
```
"{library} {method} documentation 2024"
"{library} {feature} API reference"
"{library} official docs {feature}"
```

### Migration Between Versions
```
"{library} v{old} to v{new} migration"
"{library} {version} breaking changes"
"{library} upgrade guide {version}"
```

### Code Examples
```
get_code_context_exa("{library} {pattern} implementation example")
get_code_context_exa("{library} {use_case} tutorial")
```

### Security/Auth Patterns
```
"{auth_method} best practices 2024"
"{library} authentication {pattern} security"
"OAuth PKCE {language} 2024"
```

### Error Resolution
```
"{library} {error_message} fix"
"{library} {error_type} troubleshooting"
```

---

## Tools Reference

| Tool | Use For |
|------|---------|
| `web_search_exa(query)` | General documentation search |
| `get_code_context_exa(query)` | Code examples from GitHub/tutorials |
| `crawling(url)` | Specific documentation page |

---

## Anti-Patterns (Don't Do This)

| Bad Query | Problem | Better Query |
|-----------|---------|--------------|
| `"how to use {library}"` | Too vague | `"{library} {specific_feature} 2024"` |
| `"{library} tutorial"` | May be outdated | `"{library} {feature} official docs"` |
| `"best {library}"` | Opinion, not docs | `"{library} {pattern} documentation"` |
| `"{library}"` alone | No specificity | Add feature + version + year |

---

## Query Strengthening

If initial query returns poor results:

1. **Add version**: `"React 19 useOptimistic"` vs `"React useOptimistic"`
2. **Add year**: `"FastAPI middleware 2024"` vs `"FastAPI middleware"`
3. **Add "official"**: `"Next.js official docs app router"`
4. **Be more specific**: `"Prisma findMany where clause"` vs `"Prisma queries"`
5. **Try alternate terms**: `"authentication"` vs `"auth"` vs `"login"`
