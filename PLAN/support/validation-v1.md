# PRV Validation Record v1

**Date:** 2025-12-29
**Validator:** RedMountain (Claude Code session 771)
**Method:** Live validation on real AI-generated commits

## Approach

Rather than synthetic test data, this validation uses **real commits from active development** linked to their actual originating sessions. This is stronger evidence than synthetic data because:

1. Commits are genuine AI-assisted work (not fabricated)
2. Sessions are real Claude Code conversations (not mocked)
3. File paths and code blocks reflect actual development patterns

## Results

| Commit | Session | Confidence | Step | Correct |
|--------|---------|------------|------|---------|
| `753d066` | 771 | 100% | 1 | ✓ |
| `2456bbd` | 771 | 100% | 1 | ✓ |
| `6062b77` | 771 | 100% | 1 | ✓ |
| `0cf91ae` | 771 | 95% | 1 | ✓ |
| `0b21e24` | 771 | 100% | 1 | ✓ |
| `29976b7` | 769 | 95% | 1 | ✓ |
| `9eed26c` | 769 | 90% | 1 | ✓ |

**Accuracy: 7/7 (100%)**

## Session Mapping

- **Session 771:** Current session (RedMountain) - Phase 1 implementation
- **Session 769:** Prior session - earlier Phase 1 work

Both sessions worked on PRV in the same workspace, explaining why some commits map to different sessions.

## Metrics

| Metric | Target | Actual |
|--------|--------|--------|
| Accuracy | >80% | 100% |
| Avg Confidence | - | 97% |
| Match Step | - | All Step 1 |

## Validation Method

```bash
# Link commits
prv link --commit <sha>

# Verify link
prv query <sha>

# Raw data stored in .prv/links/
```

## Conclusion

PRV matching pipeline successfully links AI-generated commits to their originating sessions with high confidence. All 7 test commits matched correctly via Step 1 (file path overlap), demonstrating the pipeline works without needing Step 2 fallback for typical commits.

## Raw Data

See `corpus-v1.jsonl` for machine-readable validation corpus.
