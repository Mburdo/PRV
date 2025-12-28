# PRV

PRV is a Git‑native “memory layer” for AI‑assisted development.

It captures what your AI tool ran and said, links that context to the commit it produced, and (optionally) shares that context with teammates through Git—without requiring any server.

---

## What PRV should do (in one breath)

When a developer uses an AI tool to change code, PRV should:

1. Record the session as an immutable **Draft** (a local artifact).
2. Fingerprint the resulting code changes so they can be matched later.
3. After the developer commits, create a **Link** from the commit to the Draft(s) that likely produced it.
4. Let teams share Drafts/Links via a dedicated Git branch (`prv-memory`) that doesn’t touch main history.
5. Surface that context where developers work (CLI now; VS Code extension later).

The end result: Git still tells you *what* changed; PRV makes it easy to see *why*.

---

## The promise (product principles)

- **Git-native:** no new backend, no hosted service requirement.
- **Local-first & private by default:** everything stays on your machine unless you explicitly push it.
- **Immutable artifacts:** Drafts and Links are append-only; never silently edited.
- **Non-invasive:** PRV should not change how you commit code; it should not block normal Git workflows.
- **Deterministic:** given the same inputs, linking should produce the same results.

---

## The mental model

### Draft

A Draft is one recorded tool session: “this is what the AI tool did.”

It should contain:
- **Transcript:** combined output of the tool run (bounded; if truncated, the Draft must say so).
- **Repo context at start:** repo root, `HEAD` before the run, staged files before the run, whether the repo was dirty, git author, hostname.
- **Change fingerprints:** stable hashes derived from the resulting diff hunks (+/- lines only).
- **Files touched (best effort):** derived from the patch used for fingerprinting.

Drafts should be **immutable** after creation.

### Link

A Link is: “this commit was likely produced by these Draft(s).”

It should contain:
- commit SHA (full)
- parent commit SHA (used for filtering)
- a list of matches with:
  - intersection count (overlapping hunks)
  - counts for draft hunks / commit hunks
  - a score (overlap ratio)
- deterministic ordering (same sort every time)

Links should be **immutable** after creation.

### Index

The Index is an internal performance optimization: “which drafts mention this hunk hash?”

PRV should maintain an index so linking doesn’t require loading every Draft in the repo.

If the index is missing or corrupted, PRV should rebuild it from existing Drafts.

### `prv-memory` branch

`prv-memory` is how PRV shares context through Git without polluting normal history.

Drafts and Links are pushed to an orphan branch (not to your feature branch), so teams can collaborate on context with normal Git remotes and permissions.

---

## Where PRV stores data

In a repo, PRV should create a `.prv/` directory:

```
.prv/
├── drafts/           # Draft JSON files (immutable)
├── links/            # Link JSON files (immutable)
├── index/            # Index files (rebuildable)
├── hooks/            # Git hook scripts (installed by prv)
└── worktrees/        # Worktree used for prv-memory operations
```

Storage rules:
- Drafts/links should be written with restrictive permissions (`0600`).
- Hooks should be executable (`0755`).
- PRV should add `.prv/` to `.git/info/exclude` to avoid accidental commits.

---

## How PRV should work (user flows)

### 1) Install into a repo (hooks + ignore)

`prv install` should:
- create required `.prv/` subdirectories
- write hook scripts into `.prv/hooks/`
- set `git config core.hooksPath .prv/hooks`
- add `.prv/` to `.git/info/exclude`
- detect and warn about pre-existing hooks/hook paths (never delete user hooks silently)

Hooks must be “soft-fail”: they should never block `git commit`, `git push`, or `git merge`. If PRV fails, hooks should print a warning and exit `0`.

### 2) Record an AI tool session (Draft)

`prv wrap <tool> [args...]` should:
- run the tool exactly as requested
- capture a transcript (stdout/stderr, best-effort merged)
- record pre-run repo context
- compute the post-run patch (tracked changes + untracked file diffs)
- compute hunk fingerprints from that patch
- write an immutable Draft to `.prv/drafts/<uuid>.json`
- update the index

Important behavior:
- `prv wrap` should return the wrapped tool’s exit code.
- If a timeout occurs, PRV should still produce a Draft and mark it as truncated/timeout.
- If run outside a Git repo, PRV should still run the tool and capture transcript, but it cannot link to commits; it should not pretend otherwise.

### 3) Commit normally (PRV stays out of the way)

Developers keep using normal Git:
- edit files
- `git add` as desired
- `git commit`

PRV should not require special commit messages or staging behavior.

### 4) Link a commit to Drafts

After a commit, `prv link --commit HEAD` (typically via `post-commit`) should:
- compute the commit’s hunk fingerprints from `git show` (unified=0, no color)
- use the index to find candidate Drafts that share any fingerprints
- prefer Drafts created against the commit’s parent (base-head filter), while allowing unknown base-head drafts
- score candidates by overlap ratio:
  - `score = intersection / max(draft_hunks, commit_hunks)`
- select matches that pass thresholds:
  - `MinIntersection` (default `2`) OR `MinScore` (default `0.3`)
- write an immutable Link to `.prv/links/<commit>.json`

Correctness expectations:
- matching must be deterministic (stable sorting, stable thresholds)
- corrupted Draft files should be skipped (not crash the whole link)
- if the index is corrupted, PRV should rebuild it and retry

Known trade-off (by design):
- purely formatting-only changes may not link well (fingerprinting is content-based and diff-driven).

### 5) Share context with teammates (explicit, safe)

PRV should support sharing Drafts/Links through a remote via the orphan branch `prv-memory`.

#### Queue (hook-driven)

On `git push`, the `pre-push` hook should call:

```
prv sync queue --remote origin
```

This should **not** push anything. It should only record that new Drafts/Links exist and remind the user to push PRV context explicitly.

#### Push (human-driven)

`prv sync push` should:
- show how many Drafts/Links will be shared
- warn that transcripts may contain sensitive data
- require explicit confirmation unless `--force` is provided
- copy only `.json` Draft/Link artifacts into a `prv-memory` worktree
- commit and push to the remote branch `prv-memory`
- handle concurrent pushes with retry (fetch/rebase/retry up to a small max)
- set `PRV_SYNCING=1` during push so hooks don’t recurse

#### Pull (merge-driven)

After `git merge` / `git pull`, the `post-merge` hook should call:

```
prv sync pull
```

This should:
- fetch remote `prv-memory`
- hydrate missing Draft/Link files locally
- never overwrite local files
- rebuild the index so new Drafts can be linked

---

## How developers should consume PRV context

PRV should make it easy to answer:
- “Why did we do this?”
- “What prompt/constraints produced this change?”
- “Which tool run(s) led to this commit?”

### CLI (should provide)

PRV should provide simple read paths:
- `prv query <commit>`: show linked Draft(s) and transcript snippets for the commit
- `prv status`: show counts (drafts, links, queued items) and basic health
- `prv diagnose`: verify hooks installed, index readable, remote branch reachable (best effort)

### VS Code extension (planned)

The extension should:
- use `git blame` to map a line to a commit
- read `.prv/links/<sha>.json` to find Draft IDs
- read `.prv/drafts/<id>.json` to show the transcript/context
- render a clean hover (markdown), stripping ANSI noise
- cache aggressively (LRU + invalidation) to stay fast while scrolling

---

## Safety & privacy (must-haves)

- PRV must never auto-share transcripts without an explicit push action.
- PRV must make the risk obvious at the moment of sharing (`sync push` confirmation text).
- Drafts/Links should be treated like logs and stored with restrictive permissions.
- Hooks must not block normal Git operations (ever).

Non-goal (for v1): secret detection/redaction. PRV should assume transcripts may contain secrets and require human judgment before sharing.

---

## Failure modes (how it should behave under stress)

- **Not a Git repo:** `wrap` still runs the tool and captures transcript; linking/syncing should clearly report “not in a repo”.
- **Index missing/corrupt:** PRV should rebuild from Drafts and continue.
- **Draft file corrupt:** skip it; do not break linking for the repo.
- **Remote branch missing:** `sync pull` should be a no-op (not an error).
- **Concurrent pushes:** `sync push` should retry safely.
- **Very large output:** capture should truncate and mark Draft as truncated (without crashing).

---

## What PRV is not

- Not a replacement for Git history.
- Not a server product (transport is Git).
- Not an “AI code generator”; it records and links what your tools did.
- Not a guarantee of perfect attribution (linking is heuristic and may have false positives/negatives).

---

## Glossary

| Term | Meaning |
|---|---|
| Draft | Immutable record of one tool session |
| Link | Immutable mapping from a commit → Draft(s) |
| Index | Fast lookup map from hunk hash → Draft IDs |
| prv-memory | Orphan branch used to share Drafts/Links |
| Wrap | Run a tool and record a Draft |
| Link (verb) | Create/update a Link file for a commit |

