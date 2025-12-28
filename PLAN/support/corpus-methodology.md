# SPIKE-002: Corpus v1 Methodology

**Goal:** Build a small, labeled dataset of commit↔session pairs to calibrate the step‑ladder scoring approach (time, path hints, content overlap).

---

## 1) Data Sources

**Commits**
- Source: `git log --format="%H %ad %s" --date=iso-strict`
- Changed files: `git diff-tree --no-commit-id --name-only -r --root <sha>`

**Sessions**
- Source: local CASS‑captured JSONL in `~/.claude/projects/-Users-burdo-Desktop-PRV/*.jsonl`
- Parsed fields: `sessionId`, `timestamp`, first user message line
- Derived fields: `start`, `end` per session (min/max timestamp across file)

**Note:** This v1 corpus is time‑oriented and does **not** yet parse code blocks or file‑path hints from message content. Those will be added in a v2 corpus once we extract them directly from CASS messages.

---

## 2) Pair Construction

1) Enumerate commits for the PRV repo.  
2) Enumerate sessions for the PRV workspace.  
3) For each commit, select a small set of candidate sessions that are:
   - in the same workspace, and
   - temporally proximate (commit time between session start and session end + buffer)
4) Add **negative** candidates (out‑of‑window or unrelated “Warmup” sessions).
5) Assign labels:
   - `match_type`: `exact | partial | none`
   - `confidence`: `high | medium | low`
   - `notes`: brief rationale

---

## 3) JSONL Schema

Each line in `PLAN/support/corpus-v1.jsonl`:

```json
{
  "commit_sha": "…",
  "session_id": "…",
  "workspace": "/Users/burdo/Desktop/PRV",
  "file_paths": ["…"],
  "time_delta_sec": 123,
  "match_type": "partial",
  "confidence": "medium",
  "notes": "In-window; planning session overlap."
}
```

Field meanings:
- **time_delta_sec**: `commit_time - session_end` (seconds; negative means commit during session).
- **match_type**:
  - `exact`: commit likely derived directly from code blocks in session (not used in v1).
  - `partial`: plausible linkage based on timing and session intent.
  - `none`: no plausible linkage (negative example).
- **confidence**: manual assessment given limited evidence.

---

## 4) Known Limitations (v1)

- **No code‑block evidence** yet; labels rely on time + session intent.
- **Warmup sessions** appear in data; treated as low‑confidence negatives.
- **Path hints not extracted** yet; file‑level correlation is only from git.

This is acceptable for **time‑model calibration**, but insufficient for content scoring validation.

---

## 5) Next Steps (v2 Corpus)

1) Extract code blocks + file‑path mentions from CASS messages.  
2) Add `code_overlap_score` and `path_hint_score` for each pair.  
3) Promote some `partial` labels to `exact` where code overlap is confirmed.  
4) Refit the time prior using the expanded corpus.
