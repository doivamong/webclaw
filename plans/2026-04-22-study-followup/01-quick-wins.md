# 01 — Quick Wins (Ship Today)

**Date**: 2026-04-22
**Type**: Config + docs hardening
**Status**: Ready to execute
**Crate(s) affected**: workspace root, `webclaw-core`
**Context**: 6 independent commits, mỗi cái <1h. Source từ study audit (kreuzberg config patterns, llm_readability CJK regex, Govcraft release profile, curl-impersonate docs drift).

## Executive Summary

Ship 6 patches độc lập nhau, không blocker, mỗi patch ≤50 LOC change. Benefit: safety hardening (`unsafe_code = forbid`), i18n (CJK regex), binary size (release profile), docs accuracy (wreq), AGPL hygiene (ATTRIBUTIONS), ops flex (workflow_dispatch).

## Requirements

- [ ] Mỗi commit pass `cargo build --workspace` + `cargo test --workspace`
- [ ] 6 commits riêng biệt (không squash), chuẩn conventional commit
- [ ] Không breaking change public API
- [ ] WASM-safe giữ nguyên cho core

## Commits

### Commit 1 — workspace lints hardening

**File**: `D:/webclaw/Cargo.toml`

Add after `[workspace.dependencies]` block:
```toml
[workspace.lints.rust]
unsafe_code = "forbid"
unused_must_use = "deny"

[workspace.lints.clippy]
all = { level = "deny", priority = -1 }
pedantic = { level = "warn", priority = -1 }
```

Then per-crate, add `[lints] workspace = true` in each `crates/webclaw-*/Cargo.toml` (6 crate).

**Acceptance**:
- [ ] `cargo clippy --workspace -- -D warnings` pass (có thể trigger nhiều pedantic warning → fix riêng commit 1.1 nếu cần)
- [ ] Không crate nào có `unsafe` block (verify: `grep -rn 'unsafe' crates/*/src/ | grep -v test` = 0)

**Commit message**: `chore(workspace): add unsafe_code forbid + clippy pedantic lints`

**Risk**: pedantic có thể trigger hàng trăm warning lần đầu. Mitigation: nếu >20 warning → split thành 2 commit (add lint, fix warning batch).

---

### Commit 2 — release profile tuning

**File**: `D:/webclaw/Cargo.toml`

Add:
```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
debug = 1

[profile.release.package.webclaw-mcp]
opt-level = "z"
```

**Rationale**: `webclaw` CLI ưu tiên speed (opt-level=3), `webclaw-mcp` MCP binary ưu tiên size (opt-level="z") vì ship qua Claude Desktop/Code.

**Acceptance**:
- [ ] `cargo build --release` pass cho cả 2 binary
- [ ] `ls -l target/release/webclaw{,-mcp}` — both exist
- [ ] Binary size measurable smaller vs baseline (document trong commit message)

**Commit message**: `perf(workspace): tune release profile per-package (CLI speed, MCP size)`

---

### Commit 3 — ATTRIBUTIONS.md scaffold

**File**: `D:/webclaw/ATTRIBUTIONS.md` (new)

```markdown
# Attributions

webclaw is licensed under AGPL-3.0. Portions of this codebase are adapted from third-party projects under permissive licenses. This file tracks attribution required by those licenses.

## Ported/adapted code

<!-- Add entry per port:
## <function/pattern name>

- **Source**: https://github.com/<owner>/<repo> (<license>)
- **Original**: <URL to specific file/line>
- **Used in**: `crates/webclaw-<crate>/src/<file>.rs` (<function name>)
- **Adaptations**: <brief note>
-->

_No ports yet._

## Reference-only studies

Study reports tại `research/github_*/_wc_ref_meta.md` không phải port code, chỉ học pattern. Không cần attribution vì không copy source.
```

**Acceptance**:
- [ ] File tồn tại
- [ ] Template ready cho future port

**Commit message**: `docs: add ATTRIBUTIONS.md scaffold for AGPL compliance`

---

### Commit 4 — docs drift `primp` → `wreq`

**Files**:
- `D:/webclaw/CLAUDE.md` (lines 14, 41, 66, 68 per grep earlier)
- `D:/webclaw/.claude/rules/crate-boundaries.md` (lines 36, 63, 75, 84, 148)

**Replacement rules**:
1. `"HTTP client via primp"` → `"HTTP client via wreq (BoringSSL-based TLS impersonation)"`
2. `"primp TLS impersonation"` → `"wreq TLS impersonation"`
3. `"primp requires [patch.crates-io]..."` → verify `D:/webclaw/Cargo.toml` KHÔNG có `[patch.crates-io]` hiện tại. Update rule: `"wreq 6.0.0-rc.28 self-contained — không cần [patch.crates-io] ở workspace root."`
4. `"NOT primp-patched"` → `"NOT wreq-patched"` (trong webclaw-llm context)
5. `grep` command trong rules: `'wreq\|primp'` giữ nguyên (grep both để catch legacy), comment rõ primp là legacy reference.

**Acceptance**:
- [ ] `grep -rn 'primp' D:/webclaw/{CLAUDE.md,.claude/}` — chỉ còn trong grep-command examples hoặc comment "legacy"
- [ ] `grep -rn 'wreq' D:/webclaw/CLAUDE.md` ≥ 3 match

**Commit message**: `docs: fix primp→wreq drift in CLAUDE.md and crate-boundaries rule`

---

### Commit 5 — `workflow_dispatch` manual trigger

**File**: `D:/webclaw/.github/workflows/release.yml` lines 3-5

**Before**:
```yaml
on:
  push:
    tags: ["v*"]
```

**After**:
```yaml
on:
  push:
    tags: ["v*"]
  workflow_dispatch:
    inputs:
      version:
        description: 'Version tag (e.g., v0.3.5)'
        required: true
        type: string
```

**Acceptance**:
- [ ] GitHub Actions UI cho phép "Run workflow" trên workflow Release
- [ ] YAML valid (local check: `actionlint .github/workflows/release.yml` nếu installed)

**Commit message**: `ci(release): add workflow_dispatch manual trigger`

---

### Commit 6 — CJK punctuation heuristic port

**Files**:
- `D:/webclaw/crates/webclaw-core/src/extractor.rs` (modify `score_node` around line 762-847)
- `D:/webclaw/ATTRIBUTIONS.md` (add attribution entry)

**Pattern to port** (from `D:/webclaw/research/github_spider-rs_readability/src/scorer.rs:21`):
```rust
// Add near imports
use once_cell::sync::Lazy;
use regex::Regex;

static CJK_PUNCTUATIONS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"([、。，．！？])").unwrap()
});

// Add to score_node(), after link density penalty:
// Bonus for CJK text: count CJK sentence-ending punctuation.
// Adapted from github.com/spider-rs/readability (MIT) — PUNCTUATIONS_REGEX
let cjk_sentence_count = CJK_PUNCTUATIONS.find_iter(&text).count() as f64;
if cjk_sentence_count > 0.0 {
    score += cjk_sentence_count.min(10.0);
}
```

**Also add 3 test fixtures sau** (`benchmarks/fixtures/cjk/`) — defer vào `02-benchmark-harness.md` vì cần harness crate.

**Acceptance**:
- [ ] Unit test `score_node` trên HTML Japanese trả về score > English same-length baseline (thêm test inline `#[cfg(test)] mod tests` trong extractor.rs)
- [ ] `cargo test -p webclaw-core` pass
- [ ] ATTRIBUTIONS.md có entry cho CJK port

**Commit message**: `feat(core): add CJK punctuation heuristic to score_node`

**Risk**: Regex add per-call overhead trên Latin-only content. Mitigation: regex find_iter không match Latin → fast path; đo benchmark sau khi 02 ship.

## Execution order suggestion

1. Commit 3 (ATTRIBUTIONS scaffold) — zero risk, 5 min
2. Commit 5 (workflow_dispatch) — zero code risk, 10 min
3. Commit 4 (docs drift) — doc only, 30 min
4. Commit 2 (release profile) — config only, 15 min + build verify
5. Commit 1 (workspace lints) — may trigger warning storm, 30 min + potential fix
6. Commit 6 (CJK regex) — code change, 30 min

Total: ~3-4h.

## Verification sau 6 commit

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo build --release --workspace
cargo test --workspace
```

## Next skill

`wc-pre-commit` sau mỗi commit (6 lần). Sau hết 6 commit → ship xong, bắt đầu `02-benchmark-harness.md`.
