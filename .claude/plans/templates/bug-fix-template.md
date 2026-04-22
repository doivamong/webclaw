# [Bug Fix] Plan

<!-- Template paraphrased for webclaw (Rust) — inspired by claudekit-engineer (commercial, not redistributed). -->

**Date**: YYYY-MM-DD
**Type**: Bug Fix
**Priority**: [Critical / High / Medium / Low]
**Crate(s) affected**: [core / fetch / llm / pdf / mcp / cli]
**Context Tokens**: <150 words

## Executive Summary

Mô tả ngắn bug và tác động (1-2 câu). Ai bị ảnh hưởng (maintainer dev loop / end-user CLI / MCP client).

## Issue Analysis

### Symptoms
- [ ] Symptom 1 (observable behavior)
- [ ] Symptom 2

### Root Cause
Giải thích ngắn nguyên nhân gốc. Không đoán — chỉ viết sau khi `wc-debug-map` Step 2 verify bằng evidence.

### Evidence
- **Logs / panic trace**: reference path hoặc commit hash, không paste full log
- **Error message**: key pattern (vd: `panicked at 'called Option::unwrap() on a None'`)
- **Affected files**: `crates/webclaw-*/src/...rs`
- **Repro steps**: command + input tối giản để reproduce

## Context Links

- **Related issues / PRs**: [GitHub link]
- **Recent changes**: commits chạm file liên quan 30 ngày qua (`git log -- crates/...`)
- **Dependencies**: crate downstream có dùng behavior này không

## Solution Design

### Approach
Fix strategy 2-3 câu. Có breaking change không? Ảnh hưởng API public crate không?

### Changes Required
1. **`crates/webclaw-<crate>/src/<file>.rs`**: mô tả ngắn
2. **`crates/webclaw-<crate>/src/<file>.rs`**: mô tả ngắn

### Testing Changes
- [ ] Test regression (fail trước fix, pass sau fix) trong `#[cfg(test)] mod tests`
- [ ] Test edge case bổ sung nếu root cause là missing boundary check
- [ ] Integration test trong `tests/*.rs` nếu bug xuất hiện ở crate boundary
- [ ] Benchmark corpus check nếu chạm `webclaw-core/extractor.rs` hoặc `markdown.rs`

## Implementation Steps

1. [ ] Write failing test capture behavior (RED)
2. [ ] Fix code - file: `crates/webclaw-<crate>/src/...rs`
3. [ ] Run `cargo test -p webclaw-<crate>` (GREEN)
4. [ ] Run `cargo clippy --workspace -- -D warnings`
5. [ ] Run `cargo fmt --check`
6. [ ] Full `cargo test --workspace`
7. [ ] Manual smoke test nếu bug là CLI/MCP surface

## Verification Plan

### Test Cases
- [ ] Case 1: behavior expected sau fix
- [ ] Case 2: edge case không regression
- [ ] Regression test: bug không tái xuất

### Rollback Plan
Nếu fix gây issue:
1. Revert commit: `git revert <commit-hash>`
2. Restore previous behavior trong file `X`, `Y`
3. Re-open issue với data mới

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Fix breaks public API | High | Version bump + CHANGELOG note. Deprecate trước remove. |
| Fix gây regression benchmark | Med | Run `benchmarks/` corpus, compare ground-truth, abort nếu >5% drop |
| Fix vi phạm WASM-safe core | High | `wasm_boundary_check.py` hook block + `wc-arch-guard` |

## TODO Checklist

- [ ] Root cause xác định + evidence ghi
- [ ] Test failing viết trước
- [ ] Implement fix
- [ ] `cargo test --workspace` pass
- [ ] `cargo clippy --workspace -- -D warnings` pass
- [ ] `cargo fmt --check` pass
- [ ] CHANGELOG entry (nếu public-facing)
- [ ] `wc-pre-commit` checklist
