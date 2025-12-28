# SPIKE-002: Scoring Rubric (Precision-First)

**Purpose:** Rank candidate CASS sessions for a commit using a precision-first, evidence-weighted score. Time is a *prior*, not a hard gate.

**Principles:**
- Prefer **Unknown** over incorrect attribution.
- Use **workspace** as a hard filter.
- Use **time** as a *decay prior* (not a fixed 30m cutoff).
- Escalate complexity only as needed (step-ladder).

---

## 1) Inputs

- Commit: `commit_sha`, `commit_time`, `changed_files`, `diff_lines`
- Session: `session_id`, `workspace_path`, `started_at`, `ended_at`, `messages[]`
- Derived: `code_blocks[]`, `file_path_hints[]`

---

## 2) Candidate Selection (Hard Filters)

1. **Workspace match** (hard gate): session workspace path must match repo root (REQ-004).
2. **Time search window for indexing only**: use a wide window (e.g., +/- 7 days) to keep the candidate set bounded. Do **not** use 30m as a final cutoff.

---

## 3) Time Scoring (Best-Available Model)

### 3.1 Use **last relevant activity** time, not session end

`last_activity_at` priority:
1) timestamp of the **last code block** in the session **touching a changed file**, if available  
2) timestamp of the **last message mentioning a changed file**, if available  
3) fallback: `session.ended_at`

`delta = commit_time - last_activity_at` (seconds)

If `delta <= 0`, the commit occurred during the session → **time_score = 1.0**

### 3.2 Preferred: **Empirical Survival Function**

Inter-event times in human activity are heavy-tailed; a non-parametric prior is more robust than a hard cutoff.

**Model:** Build an empirical CDF from the labeled corpus, then:

```
time_score = 1 - ECDF(delta)
```

Interpretation:
- If `delta` is at the 50th percentile → `time_score = 0.5`
- If `delta` is at the 90th percentile → `time_score = 0.1`

This avoids assuming a specific distribution and adapts to real user behavior.

### 3.3 Fallback: **Exponential Decay with Half-Life**

If corpus is too small (<30 pairs), use an exponential decay (widely used for recency ranking).

```
if delta <= offset:
    time_score = 1.0
else:
    time_score = exp(-ln(2) * (delta - offset) / half_life)
```

Defaults (until corpus fit is available):
- `offset = 5 min` (full credit window)
- `half_life = 30 min` (score halves every 30 min)

**Rationale:** Exponential decay emphasizes recency but preserves a long tail.

### 3.4 Evidence

- Exponential/decay rankers are standard for recency weighting in search systems (Milvus decay ranker docs).
- Human activity and inter-event times are heavy-tailed; non-parametric or lognormal-like priors outperform hard cutoffs.
- Commit intervals in OSS show heavy tails and project-specific variance; avoid a universal 30m gate.

References:
- https://milvus.io/docs/exponential-decay.md
- https://milvus.io/docs/decay-ranker-overview.md
- https://pengcui.thumedialab.com/papers/PlosOne_BehaviorDynamics.pdf
- https://arxiv.org/abs/1309.0897

---

## 4) Evidence Signals (Beyond Time)

### 4.1 File Path Hints

Score higher if any of `changed_files` are explicitly mentioned in session text or code blocks.

```
path_score = 1.0 if exact path mention
           = 0.6 if basename match
           = 0.3 if directory match
           = 0.0 otherwise
```

### 4.2 Content Overlap (Step-Ladder)

**Step 2:** normalized line-hash overlap  
**Step 3:** token overlap (bag-of-tokens)  
**Step 4:** SimHash (only if needed)

Content score should be **bounded** and never override a low time_score alone.

---

## 5) Overall Scoring (Precision-First)

Recommended weighted sum (tunable after corpus eval):

```
score = 0.35 * time_score
      + 0.35 * content_score
      + 0.20 * path_score
      + 0.10 * session_context_score  # optional: agent/tool match, prompt relevance
```

**Decision policy:**
- If top score < 0.65 → return **Unknown**
- If top two scores within 0.10 → return both, mark **low confidence**

---

## 5.1) Provenance Status Model

We distinguish **confirmed** vs **provisional** provenance:

- **Confirmed**: commit-linked match (persisted in `.prv/links/`)
- **Provisional**: working-tree match (ephemeral, updates on save)

**Provisional scoring:** use the same rubric, but replace `commit_time` with **file mtime** (fallback to `now`) to compute `delta`.  
Always present provisional matches with explicit status and reduced confidence.

**Priority rule:** confirmed beats provisional. Provisional never overrides a confirmed match.

---

## 6) Step-Ladder Evaluation Plan

Evaluate on corpus in order:

0) Time + workspace only  
1) + file path hints  
2) + normalized line-hash  
3) + token overlap  
4) + SimHash (only if needed)

Stop when precision target is met.

---

## 7) Parameters to Fit from Corpus

- `offset` and `half_life` (if using exponential)
- ECDF quantiles for time_score mapping
- Score threshold (`0.65`) and tie margin (`0.10`)

---

## 8) Notes

- Time score is a **prior**, not a veto. Strong content match can outweigh a weak time score, but only within a reasonable horizon.
- Maintain a **max search horizon** (e.g., 7–30 days) for index performance, not for correctness.
