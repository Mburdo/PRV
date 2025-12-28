# Phase 1 Decomposition: Core Linking

**Date:** 2025-12-28
**Author:** BrownLake
**Status:** Draft (pending SPIKE-002 closure)

---

## North Star

Context tracing for code. Phase 1 delivers the core value proposition: linking git commits to AI sessions.

**Milestone M1: "It Links"**
- PRV can link commits to sessions
- Accuracy >80% on AI-generated commits
- Links persist in `.prv/`

---

## Decomposition Strategy

**Feature-based** with dependency awareness:
- **Track A:** Matching Algorithm (hard gates → step-ladder)
- **Track B:** Link Storage (model → persistence → index)
- **Track C:** CLI Commands (`prv link`, `prv query`)
- **Integration:** Validation with corpus

---

## Requirements Addressed

| REQ | Scope | Beads |
|-----|-------|-------|
| REQ-002 | Commit-session linking (full) | A.1-A.4, B.1-B.3 |
| REQ-005 | CLI query (partial: `prv query`) | C.2 |
| REQ-007 | Git hook linking (full) | C.1 |
| REQ-008 | Index for fast lookup (full) | B.3 |

---

## Exit Criteria (from PLAN/06_phases.md)

- [ ] `prv link --commit HEAD` creates link file
- [ ] `prv query <sha>` returns matching session(s) with confidence
- [ ] Manual audit of 50 commits shows >80% accuracy
- [ ] Index rebuild < 30s for repo with 1000 commits

---

## Epic

| ID | Title | Status |
|----|-------|--------|
| PRV-p1 | Phase 1: Core Linking | open |

---

## Task Beads

### Track A: Matching Algorithm

| ID | Title | Dependencies | Description |
|----|-------|--------------|-------------|
| PRV-p1.1 | Workspace gate + time index window | - | Filter by workspace; apply a wide time window for indexing only |
| PRV-p1.2 | Step 0: Single candidate matching | .1 | When exactly one session passes gates, return with 0.9 confidence |
| PRV-p1.3 | Step 1: File path hints disambiguation | .2 | Disambiguate multiple candidates by file path mentions |
| PRV-p1.4 | Step 2: Line hash exact matching | .3 | Disambiguate by normalized line hash overlap |

### Track B: Link Storage

| ID | Title | Dependencies | Description |
|----|-------|--------------|-------------|
| PRV-p1.5 | Link model and serialization | - | `Link` struct with commit_sha, session_id, confidence, timestamp |
| PRV-p1.6 | Link persistence to `.prv/links/` | .5 | Write/read link JSON files organized by commit prefix |
| PRV-p1.7 | Link index for fast lookup | .6 | In-memory index with disk persistence for O(1) commit→session lookup |

### Track C: CLI Commands

| ID | Title | Dependencies | Description |
|----|-------|--------------|-------------|
| PRV-p1.8 | `prv link` command | .4, .6 | Link commits to sessions, create link files |
| PRV-p1.9 | `prv query` command | .7, .8 | Query session(s) for a commit with confidence |

### Integration

| ID | Title | Dependencies | Description |
|----|-------|--------------|-------------|
| PRV-p1.10 | Corpus validation | .9 | Validate >80% accuracy on 50-commit corpus |

---

## Dependency Graph

```
                    PRV-p1.1 (Hard gates)
                         │
                         ▼
                    PRV-p1.2 (Step 0)
                         │
                         ▼
                    PRV-p1.3 (Step 1: file hints)
                         │
                         ▼
                    PRV-p1.4 (Step 2: line hash)
                         │
                         │
    PRV-p1.5 ────────────┤
    (Link model)         │
         │               │
         ▼               │
    PRV-p1.6 ────────────┤
    (Persistence)        │
         │               │
         ▼               │
    PRV-p1.7             │
    (Index)              │
         │               │
         ├───────────────┘
         │               │
         ▼               ▼
    PRV-p1.9        PRV-p1.8
    (prv query)     (prv link)
         │               │
         └───────┬───────┘
                 │
                 ▼
            PRV-p1.10
       (Corpus validation)
```

---

## Execution Order

**Parallel Track A:** .1 → .2 → .3 → .4
**Parallel Track B:** .5 → .6 → .7
**Parallel Track C:** .8 (after A.4, B.6), .9 (after B.7, C.8)
**Integration:** .10 (after all)

**Recommended start:** PRV-p1.1 and PRV-p1.5 in parallel (unblock both tracks).

---

## Detailed Bead Specifications

### PRV-p1.1: Hard gates: workspace + time filter

**North Star:** Precision-first matching. Wrong provenance is worse than "Unknown."

**File:** `crates/prv-core/src/matcher/gates.rs`

```rust
use prv_cass::{CassDb, Session};
use std::path::Path;
use chrono::{DateTime, Utc, Duration};

/// Filter sessions that could have produced this commit.
/// Hard gate: workspace must match. Time window only bounds indexing.
pub fn candidate_sessions(
    db: &CassDb,
    repo_path: &Path,
    commit_time: DateTime<Utc>,
) -> anyhow::Result<Vec<Session>> {
    let workspace = db.find_workspace_for_path(repo_path)?;

    let Some(ws) = workspace else {
        return Ok(vec![]);  // No CASS workspace for this repo
    };

    let index_window = Duration::days(7);

    let sessions = db.sessions_for_workspace(ws.id)?
        .into_iter()
        .filter(|s| {
            let session_end = s.ended_at.unwrap_or(s.started_at);
            let start_ok = commit_time + index_window >= s.started_at;
            let end_ok = commit_time - index_window <= session_end;
            start_ok && end_ok
        })
        .collect();

    Ok(sessions)
}
```

**Tests (TDD):**
```rust
#[test]
fn test_no_workspace_returns_empty() { ... }

#[test]
fn test_outside_index_window_filtered() { ... }

#[test]
fn test_workspace_mismatch_filtered() { ... }
```

**Verification:**
- [ ] Tests written and fail
- [ ] Workspace matching works (case-insensitive on macOS)
- [ ] Index window bounds candidate set (configurable; not correctness gate)
- [ ] Returns empty vec (not error) for no matches

---

### PRV-p1.2: Step 0: Single candidate matching

**File:** `crates/prv-core/src/matcher/step0.rs`

```rust
use crate::matcher::gates::candidate_sessions;
use prv_cass::Session;

pub struct MatchResult {
    pub session: Session,
    pub confidence: f64,
    pub step: u8,  // Which step produced this match
}

/// Step 0: If exactly one candidate, return with high confidence.
pub fn match_step0(candidates: &[Session]) -> Option<MatchResult> {
    if candidates.len() == 1 {
        Some(MatchResult {
            session: candidates[0].clone(),
            confidence: 0.9,
            step: 0,
        })
    } else {
        None
    }
}
```

**Tests:**
```rust
#[test]
fn test_single_candidate_high_confidence() { ... }

#[test]
fn test_no_candidates_returns_none() { ... }

#[test]
fn test_multiple_candidates_returns_none() { ... }
```

---

### PRV-p1.3: Step 1: File path hints disambiguation

**File:** `crates/prv-core/src/matcher/step1.rs`

```rust
use std::collections::HashSet;
use prv_cass::Session;

/// Step 1: Disambiguate by file path mentions in session.
pub fn match_step1(
    candidates: &[Session],
    commit_files: &[String],
) -> Option<MatchResult> {
    let commit_set: HashSet<_> = commit_files.iter().collect();

    for session in candidates {
        let mentioned = session.file_mentions();  // Implement in prv-cass
        let overlap = mentioned.iter()
            .filter(|f| commit_set.contains(f))
            .count();

        if overlap > 0 {
            let bonus = (overlap.min(3) as f64) * 0.05;
            return Some(MatchResult {
                session: session.clone(),
                confidence: 0.85 + bonus,
                step: 1,
            });
        }
    }

    None
}
```

**Requires:** Add `file_mentions()` method to Session in prv-cass.

---

### PRV-p1.4: Step 2: Line hash exact matching

**File:** `crates/prv-core/src/matcher/step2.rs`

```rust
use std::collections::HashSet;
use fxhash::hash64;

/// Normalize and hash a line for comparison.
fn hash_normalized(line: &str) -> u64 {
    let normalized: String = line
        .trim()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    hash64(&normalized)
}

/// Step 2: Match by exact line hash overlap.
pub fn match_step2(
    candidates: &[Session],
    diff_lines: &[String],
) -> Option<MatchResult> {
    let diff_hashes: HashSet<u64> = diff_lines.iter()
        .map(|l| hash_normalized(l))
        .collect();

    let mut best: Option<(Session, f64)> = None;

    for session in candidates {
        let session_hashes: HashSet<u64> = session.code_block_lines()
            .map(|l| hash_normalized(&l))
            .collect();

        let intersection = diff_hashes.intersection(&session_hashes).count();
        let overlap_ratio = intersection as f64 / diff_hashes.len().max(1) as f64;

        if overlap_ratio > 0.5 {
            let confidence = 0.8 + (overlap_ratio * 0.2);
            if best.as_ref().map_or(true, |(_, c)| confidence > *c) {
                best = Some((session.clone(), confidence));
            }
        }
    }

    best.map(|(session, confidence)| MatchResult {
        session,
        confidence,
        step: 2,
    })
}
```

---

### PRV-p1.5: Link model and serialization

**File:** `crates/prv-core/src/link.rs`

```rust
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub commit_sha: String,
    pub session_id: i64,
    pub confidence: f64,
    pub match_step: u8,
    pub created_at: DateTime<Utc>,
}

impl Link {
    pub fn new(commit_sha: &str, session_id: i64, confidence: f64, step: u8) -> Self {
        Self {
            commit_sha: commit_sha.to_string(),
            session_id,
            confidence,
            match_step: step,
            created_at: Utc::now(),
        }
    }
}
```

---

### PRV-p1.6: Link persistence to `.prv/links/`

**File:** `crates/prv-core/src/link_storage.rs`

```rust
use crate::Link;
use std::path::{Path, PathBuf};
use anyhow::Result;

pub struct LinkStorage {
    base_path: PathBuf,  // .prv/links/
}

impl LinkStorage {
    pub fn new(repo_root: &Path) -> Self {
        Self {
            base_path: repo_root.join(".prv").join("links"),
        }
    }

    /// Path for a commit's link file: .prv/links/ab/cdef123.json
    fn link_path(&self, commit_sha: &str) -> PathBuf {
        let prefix = &commit_sha[..2];
        self.base_path.join(prefix).join(format!("{}.json", commit_sha))
    }

    pub fn save(&self, link: &Link) -> Result<()> {
        let path = self.link_path(&link.commit_sha);
        std::fs::create_dir_all(path.parent().unwrap())?;
        let json = serde_json::to_string_pretty(link)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    pub fn load(&self, commit_sha: &str) -> Result<Option<Link>> {
        let path = self.link_path(commit_sha);
        if !path.exists() {
            return Ok(None);
        }
        let json = std::fs::read_to_string(&path)?;
        let link: Link = serde_json::from_str(&json)?;
        Ok(Some(link))
    }
}
```

---

### PRV-p1.7: Link index for fast lookup

**File:** `crates/prv-core/src/link_index.rs`

```rust
use crate::Link;
use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;

pub struct LinkIndex {
    /// commit_sha → session_id
    index: HashMap<String, i64>,
}

impl LinkIndex {
    pub fn new() -> Self {
        Self { index: HashMap::new() }
    }

    pub fn load(repo_root: &Path) -> Result<Self> {
        let index_path = repo_root.join(".prv").join("index.json");
        if !index_path.exists() {
            return Ok(Self::new());
        }
        let json = std::fs::read_to_string(&index_path)?;
        let index: HashMap<String, i64> = serde_json::from_str(&json)?;
        Ok(Self { index })
    }

    pub fn save(&self, repo_root: &Path) -> Result<()> {
        let index_path = repo_root.join(".prv").join("index.json");
        std::fs::create_dir_all(index_path.parent().unwrap())?;
        let json = serde_json::to_string(&self.index)?;
        std::fs::write(&index_path, json)?;
        Ok(())
    }

    pub fn insert(&mut self, link: &Link) {
        self.index.insert(link.commit_sha.clone(), link.session_id);
    }

    pub fn get(&self, commit_sha: &str) -> Option<i64> {
        self.index.get(commit_sha).copied()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }
}
```

---

### PRV-p1.8: `prv link` command

**File:** `crates/prv/src/commands/link.rs`

```rust
use clap::Args;
use anyhow::Result;
use prv_core::{Link, LinkStorage, LinkIndex, matcher};
use prv_cass::CassDb;

#[derive(Args)]
pub struct LinkArgs {
    /// Commit SHA to link (default: HEAD)
    #[arg(long, default_value = "HEAD")]
    commit: String,

    /// Link all commits in range
    #[arg(long)]
    all: bool,
}

pub fn run(args: LinkArgs) -> Result<()> {
    let repo = git2::Repository::open_from_env()?;
    let cass = CassDb::open()?;
    let storage = LinkStorage::new(repo.workdir().unwrap());
    let mut index = LinkIndex::load(repo.workdir().unwrap())?;

    let commit_sha = resolve_commit(&repo, &args.commit)?;
    let commit = repo.find_commit(git2::Oid::from_str(&commit_sha)?)?;
    let commit_time = commit.time();

    // Get changed files from commit
    let changed_files = get_changed_files(&repo, &commit)?;
    let diff_lines = get_diff_lines(&repo, &commit)?;

    // Run matcher
    let result = matcher::find_session(
        &cass,
        repo.workdir().unwrap(),
        commit_time,
        &changed_files,
        &diff_lines,
    )?;

    match result {
        Some(match_result) => {
            let link = Link::new(
                &commit_sha,
                match_result.session.id,
                match_result.confidence,
                match_result.step,
            );
            storage.save(&link)?;
            index.insert(&link);
            index.save(repo.workdir().unwrap())?;

            println!("Linked {} → session {} (confidence: {:.0}%, step {})",
                &commit_sha[..7],
                match_result.session.id,
                match_result.confidence * 100.0,
                match_result.step,
            );
        }
        None => {
            println!("No matching session found for {}", &commit_sha[..7]);
        }
    }

    Ok(())
}
```

---

### PRV-p1.9: `prv query` command

**File:** `crates/prv/src/commands/query.rs`

```rust
use clap::Args;
use anyhow::Result;
use prv_core::{LinkStorage, LinkIndex};
use prv_cass::CassDb;

#[derive(Args)]
pub struct QueryArgs {
    /// Commit SHA to query
    commit: String,

    /// Output as JSON
    #[arg(long)]
    json: bool,
}

pub fn run(args: QueryArgs) -> Result<()> {
    let repo = git2::Repository::open_from_env()?;
    let storage = LinkStorage::new(repo.workdir().unwrap());
    let cass = CassDb::open()?;

    let commit_sha = resolve_commit(&repo, &args.commit)?;

    if let Some(link) = storage.load(&commit_sha)? {
        let session = cass.get_session(link.session_id)?;

        if args.json {
            println!("{}", serde_json::to_string_pretty(&link)?);
        } else {
            println!("Commit: {}", &commit_sha[..7]);
            println!("Session: {} ({})", link.session_id, session.title.unwrap_or_default());
            println!("Confidence: {:.0}%", link.confidence * 100.0);
            println!("Match step: {}", link.match_step);
            println!("Linked at: {}", link.created_at);
        }
    } else {
        println!("No link found for {}. Run `prv link --commit {}` first.",
            &commit_sha[..7], &commit_sha[..7]);
    }

    Ok(())
}
```

---

### PRV-p1.10: Corpus validation

**Depends on:** LilacDog's corpus (`PLAN/support/corpus-v1.jsonl`)

**Validation script:**
```bash
#!/bin/bash
# Run step-ladder evaluation on corpus

CORPUS="PLAN/support/corpus-v1.jsonl"
TOTAL=0
CORRECT=0

while read -r line; do
    COMMIT=$(echo "$line" | jq -r '.commit_sha')
    EXPECTED=$(echo "$line" | jq -r '.session_id')

    RESULT=$(prv query "$COMMIT" --json 2>/dev/null | jq -r '.session_id')

    TOTAL=$((TOTAL + 1))
    if [ "$RESULT" = "$EXPECTED" ]; then
        CORRECT=$((CORRECT + 1))
    fi
done < "$CORPUS"

echo "Accuracy: $CORRECT / $TOTAL = $(echo "scale=2; $CORRECT * 100 / $TOTAL" | bc)%"
```

**Success criteria:**
- [ ] Accuracy > 80%
- [ ] Precision > 85%
- [ ] p95 latency < 100ms

---

## Notes

1. **SPIKE-002 must close first** — This decomposition assumes the step-ladder approach is validated.

2. **Corpus dependency** — PRV-p1.10 depends on LilacDog's corpus delivery.

3. **Incremental validation** — Can validate Steps 0-1 before implementing Step 2.

4. **Steps 3-4 not included** — Per SPIKE-002 analysis, only implement if Steps 0-2 fail to hit 80% accuracy.

---

## Ready for Review

This decomposition is a **draft** pending:
- [ ] SPIKE-002 closure (corpus + rubric)
- [ ] Review by LilacDog
- [ ] User approval to create beads
