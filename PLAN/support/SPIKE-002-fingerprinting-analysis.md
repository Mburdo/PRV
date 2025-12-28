# SPIKE-002: Fingerprinting Strategy Analysis

**Status:** Closed — Research complete; implementation moves to Phase 1
**Date:** 2025-12-28
**Authors:** BrownLake, LilacDog (claude-code/opus-4.5)

---

## Executive Summary

After extensive research into code clone detection, locality-sensitive hashing, and first-principles analysis of the problem domain, we recommend a **step-ladder approach** that starts with the simplest possible solution and adds complexity only when data demands it.

**Key Finding:** No existing tool links AI sessions to git commits. PRV solves a genuinely novel problem. Time + workspace correlation (validated by WakaTime's approach) likely handles 80%+ of cases without complex fingerprinting.

**Recommendation:** Implement Step 0-2 first. SimHash and advanced fingerprinting become fallbacks only if validation shows we need them.

See also: `PLAN/support/SPIKE-002-first-principles.md` for exhaustive prior art research.

---

## Problem Statement

PRV needs to answer: "Which AI session produced this commit?"

**Inputs:**
- Git commit (SHA, timestamp, workspace path, diff)
- CASS sessions (session_id, workspace, time range, code blocks, file mentions)

**Constraints:**
- Performance: <500ms for LSP hover (95th percentile)
- Accuracy: >80% on AI-generated commits
- **Precision-first:** Wrong provenance is worse than "Unknown" (per `PLAN/00_north_star.md`)

---

## The Matching Chain

```
Line of Code
    ↓ (git blame)
Commit SHA
    ↓ (PRV linking)
Session ID(s)
    ↓ (CASS lookup)
Session Transcript
    ↓ (summarization)
Context Display
```

---

## Step-Ladder Strategy

Start simple. Add complexity only when data shows we need it.

| Step | Technique | Expected Coverage | Complexity |
|------|-----------|-------------------|------------|
| **0** | Time prior + workspace | ~80% | Trivial |
| **1** | + File path hints | +10% | Low |
| **2** | + Line hash exact match | +5% | Low |
| **3** | + Token overlap | +3% | Medium |
| **4** | + SimHash fuzzy | +2% | Medium |

### Why This Order?

**Observation:** In most real-world scenarios:
1. Developer has **one** AI session open per project
2. Commit happens **shortly after** session activity
3. Session workspace **matches** commit repo

If (1), (2), (3) are true → fingerprinting is **unnecessary**.

**WakaTime validation:** WakaTime successfully correlates developer activity to commits using time windows alone. This proves time-based correlation is viable.

---

## Hard Gates (Non-Negotiable Filters)

Before scoring, apply the **workspace** hard gate. Time is a prior, not a cutoff.

```rust
fn candidate_sessions(commit: &Commit) -> Vec<Session> {
    sessions.filter(|s| {
        // GATE 1: Workspace must match
        s.workspace == commit.repo_path &&

        // INDEX WINDOW: bound search for performance only (not correctness)
        within_index_window(s, commit.time, Duration::days(7))
    })
}
```

**Rationale:** A session from a different workspace cannot be correct. Time should not eliminate candidates; it should rank them (see scoring rubric).

---

## Time Prior (Not a Gate)

Use **last relevant activity** (code block touching changed files, or file mentions, else session end) and score by ECDF/decay. This is the dominant *prior* but never a hard cutoff.

See `PLAN/support/scoring-rubric.md` for the exact time scoring model and thresholds.

---

## Provenance Status Model

- **Confirmed:** commit-linked match persisted in `.prv/links/`
- **Provisional:** working-tree match updated on save

Provisional uses the same scoring rubric but substitutes `now` or file mtime for `commit_time`, and is always presented with explicit low confidence.

---

## Step 0: Time Prior + Workspace (Baseline)

The simplest possible approach. Handles the common case where one session per commit.

```rust
fn find_session_step0(commit: &Commit) -> Option<(Session, f64)> {
    let candidates = candidate_sessions(commit);

    match candidates.len() {
        0 => None,
        1 => {
            let time_score = time_prior(commit, &candidates[0]);
            let confidence = 0.6 + (0.3 * time_score);
            Some((candidates[0], confidence))
        }
        _ => None,  // Ambiguous: escalate to Step 1
    }
}
```

**Expected coverage:** ~80% of commits have exactly one candidate session.

---

## Step 1: File Path Hints

When multiple sessions pass hard gates, use file path mentions to disambiguate.

```rust
fn find_session_step1(commit: &Commit, candidates: &[Session]) -> Option<(Session, f64)> {
    let commit_files: HashSet<_> = commit.changed_files.iter().collect();

    for session in candidates {
        let mentioned_files: HashSet<_> = session.file_mentions().collect();
        let overlap = commit_files.intersection(&mentioned_files).count();

        if overlap > 0 {
            let confidence = 0.85 + (0.05 * overlap.min(3) as f64);
            return Some((session, confidence));
        }
    }

    None  // Escalate to Step 2
}
```

**Signal sources:**
- File paths mentioned in session messages
- File paths in code block annotations (```rust:src/lib.rs)
- User queries like "edit the auth module"

---

## Step 2: Line Hash Exact Match

For commits where Step 0-1 fail, check if committed lines appear verbatim in session.

```rust
fn find_session_step2(commit: &Commit, candidates: &[Session]) -> Option<(Session, f64)> {
    let diff_lines: HashSet<u64> = commit.added_lines()
        .map(|line| hash_normalized(line))
        .collect();

    let mut best_match: Option<(Session, f64)> = None;

    for session in candidates {
        let session_lines: HashSet<u64> = session.code_blocks()
            .flat_map(|block| block.lines())
            .map(|line| hash_normalized(line))
            .collect();

        let intersection = diff_lines.intersection(&session_lines).count();
        let overlap_ratio = intersection as f64 / diff_lines.len() as f64;

        if overlap_ratio > 0.5 {
            let confidence = 0.8 + (overlap_ratio * 0.2);
            if best_match.map_or(true, |(_, c)| confidence > c) {
                best_match = Some((session, confidence));
            }
        }
    }

    best_match
}

fn hash_normalized(line: &str) -> u64 {
    let normalized = line
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    fxhash::hash64(&normalized)
}
```

---

## Step 3: Token Overlap (Fallback)

When code was edited before commit, exact line matching fails. Token overlap tolerates minor changes.

```rust
fn find_session_step3(commit: &Commit, candidates: &[Session]) -> Option<(Session, f64)> {
    let diff_tokens: HashSet<Token> = tokenize_diff(commit);

    for session in candidates {
        let session_tokens: HashSet<Token> = tokenize_session(session);

        let jaccard = jaccard_similarity(&diff_tokens, &session_tokens);
        if jaccard > 0.6 {
            return Some((session, 0.7 + (jaccard * 0.2)));
        }
    }

    None
}

fn jaccard_similarity<T: Hash + Eq>(a: &HashSet<T>, b: &HashSet<T>) -> f64 {
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    intersection as f64 / union as f64
}
```

---

## Step 4: SimHash Fuzzy Matching (Optional Fallback)

For heavily edited code, SimHash provides fuzzy structural matching. **Only implement if Steps 0-3 fail to hit 80% accuracy on validation corpus.**

```rust
fn find_session_step4(commit: &Commit, candidates: &[Session]) -> Option<(Session, f64)> {
    let diff_fp = simhash_fingerprint(commit);

    candidates.iter()
        .filter_map(|session| {
            let session_fp = simhash_fingerprint(session);
            let similarity = simhash_similarity(diff_fp, session_fp);

            if similarity > 0.7 {
                Some((session, 0.5 + (similarity * 0.3)))  // Lower confidence
            } else {
                None
            }
        })
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
}
```

### SimHash Algorithm (Reference)

```rust
fn simhash(ngrams: &[u64]) -> u64 {
    let mut v = [0i32; 64];

    for &ngram in ngrams {
        let hash = fxhash::hash64(&ngram);
        for i in 0..64 {
            if (hash >> i) & 1 == 1 {
                v[i] += 1;
            } else {
                v[i] -= 1;
            }
        }
    }

    let mut fingerprint = 0u64;
    for i in 0..64 {
        if v[i] > 0 {
            fingerprint |= 1 << i;
        }
    }
    fingerprint
}

fn simhash_similarity(fp1: u64, fp2: u64) -> f64 {
    let hamming = (fp1 ^ fp2).count_ones();
    1.0 - (hamming as f64 / 64.0)
}
```

---

## Complete Matching Pipeline

```rust
pub fn find_session(commit: &Commit) -> Option<(Session, f64)> {
    // Hard gate: workspace (time is a prior, not a cutoff)
    let candidates = candidate_sessions(commit);

    if candidates.is_empty() {
        return None;  // No session could have produced this commit
    }

    // Base scoring: time prior
    let mut scored = score_time_prior(commit, &candidates);

    // Step 1: Disambiguate by file path hints
    scored = add_path_scores(commit, scored);

    // Step 2: Disambiguate by line hash exact match
    scored = add_line_hash_scores(commit, scored);

    // Step 3: Token overlap (if implemented)
    // scored = add_token_overlap_scores(commit, scored);

    // Step 4: SimHash fuzzy (if implemented and needed)
    // scored = add_simhash_scores(commit, scored);

    // Final decision policy (see scoring rubric):
    // - if top score < 0.65 => Unknown
    // - if top two within 0.10 => return both (low confidence)
    pick_best(scored)
}
```

---

## What We DON'T Need (Initially)

Based on first-principles research, these are **fallbacks**, not core requirements:

| Technique | When Needed |
|-----------|-------------|
| Tokenizers | Only if Step 2 fails validation |
| N-gram extraction | Only for Step 3-4 |
| SimHash/MinHash | Only if Step 0-2 < 80% accuracy |
| AST parsing | Not needed (language-agnostic goal) |
| Complex normalization | Only if exact match fails |
| Identifier abstraction | Not needed (VUDDY-style overkill) |

---

## Validation Plan

Before building Steps 3-4, validate Steps 0-2 on real data.

### Corpus Requirements
- 50 commit ↔ session pairs
- Ground truth manually verified
- Include: single-session, multi-session, edited code, no-match cases

### Evaluation Protocol
1. Run Step 0 alone → measure accuracy
2. Add Step 1 → measure incremental improvement
3. Add Step 2 → measure incremental improvement
4. **Only if accuracy < 80%:** implement Steps 3-4

### Success Criteria
- Precision > 85% (wrong provenance is worse than "Unknown")
- Recall > 75%
- F1 Score > 80%
- p95 latency < 100ms

---

## Research Validation

### PRV is Genuinely Novel

No existing tool links AI sessions to commits:

| Domain | Tools Found | Gap |
|--------|-------------|-----|
| Code Provenance | SLSA, SCANOSS | Track code origin, not AI sessions |
| AI Detection | CodeT5-Authorship | Identify which LLM, not which session |
| Activity Tracking | WakaTime | Correlate time to commits, not conversations |
| Session Search | CASS, AgentBase | Search sessions, don't link to commits |

### WakaTime Validates Time-Based Approach

WakaTime successfully correlates developer activity to commits using time windows. This proves our Step 0 approach is viable for the majority of cases.

### Adjacent Research (Useful but Not Drop-In)

| Research | Relevance |
|----------|-----------|
| SourcererCC | Token overlap technique for Step 3 |
| NiCad/SimCad | Normalization strategies |
| VUDDY | Length filtering optimization |
| SimHash (Google) | Fuzzy matching for Step 4 |

---

## Implementation Phases

### Phase 1: Core Pipeline (3 days)
- [ ] Implement hard gate (workspace) + time prior scoring (no hard cutoff)
- [ ] Implement Step 0 (single candidate)
- [ ] Implement Step 1 (file path hints)
- [ ] Unit tests with mock data

### Phase 2: Line Matching (2 days)
- [ ] Implement Step 2 (line hash exact match)
- [ ] Implement normalization rules
- [ ] Integration tests with CASS data

### Phase 3: Validation (2 days)
- [ ] Build 50-pair corpus (see `corpus-methodology.md`)
- [ ] Run step-ladder evaluation
- [ ] Document accuracy at each step

### Phase 4: Fallbacks (IF NEEDED) (3-5 days)
- [ ] Implement tokenizer (if Step 2 < 80%)
- [ ] Implement token overlap (Step 3)
- [ ] Implement SimHash (Step 4)
- [ ] Re-validate on corpus

---

## Appendix: Rust Crate Recommendations

| Purpose | Crate | Notes |
|---------|-------|-------|
| Hashing | `fxhash` | Fast non-cryptographic hash |
| Git operations | `git2` | libgit2 bindings |
| CASS queries | `rusqlite` | SQLite bindings |
| SimHash | Custom | ~50 LOC, not worth a dep |
| Tokenizing | `logos` | Only if Step 3 needed |

---

## References

1. **First-Principles Analysis** — `PLAN/support/SPIKE-002-first-principles.md`
2. **WakaTime** — Time-based commit correlation validation
3. **CASS** — Session data source (github.com/Dicklesworthstone/coding_agent_session_search)
4. **VUDDY** — Kim et al., "Scalable Vulnerable Code Clone Discovery" (IEEE S&P 2017)
5. **SimHash** — Charikar, "Similarity Estimation from Rounding Algorithms" (STOC 2002)
6. **SourcererCC** — Sajnani et al., "Large-Scale Code Clone Detection" (ICSE 2016)
