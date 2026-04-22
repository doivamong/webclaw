---
name: wc-pre-commit
origin: adapted-from-itf
inspired-by: itf-pre-commit-check (2026-04-22, paraphrased)
description: >
  GATE — BẮT BUỘC chạy TRƯỚC MỌI commit hoặc khi task hoàn thành.
  USE WHEN: user nói "xong", "done", "hoàn thành", "commit", "finish",
  "kết thúc", "hoàn tất", "xong rồi", "lưu", "ok done", "wrap up",
  hoặc chuẩn bị kết thúc task.
  Priority: ALWAYS the LAST skill in any workflow chain.
  DO NOT TRIGGER when: user đang hỏi câu hỏi, chưa viết code, hoặc đang giữa workflow.
triggers:
  - "xong"
  - "hoàn thành"
  - "done"
  - "commit"
  - "finish"
  - "kết thúc"
  - "hoàn tất"
  - "xong rồi"
  - "lưu"
  - "ok done"
  - "wrap up"
---

Announce: "Đang chạy webclaw pre-commit checklist — 10 mục bắt buộc."

Tạo TodoWrite với TẤT CẢ 10 mục. KHÔNG bỏ qua bất kỳ mục nào.

## Failure Modes Registry (tích lũy theo thời gian)

> Mỗi khi gây failure mới, thêm entry cuối bảng. KHÔNG xóa entry cũ — memory collective.

| # | Don't | Do Instead | Auto? | Severity |
|---|-------|-----------|-------|----------|
| W1 | `use tokio::*` trong `crates/webclaw-core/` | Core WASM-safe, không async runtime | wasm_boundary hook ✓ | CRITICAL |
| W2 | `use reqwest` / `use wreq` trong core | Core là pure parser, không network | wasm_boundary hook ✓ | CRITICAL |
| W3 | `[patch.crates-io]` ở crate-level Cargo.toml | Chỉ workspace root | Manual + CI | HIGH |
| W4 | `use wreq` trong `crates/webclaw-llm/` | LLM APIs dùng plain `reqwest` | Manual | HIGH |
| W5 | `.unwrap()` trong lib code (không `#[cfg(test)]`) | `?` propagation hoặc `ok_or_else` | clippy + manual | MEDIUM |
| W6 | Version bump lệch giữa 6 crate khi release | Bump đồng bộ toàn workspace | Manual + wc-release | HIGH |
| W7 | Rename/đổi type MCP tool output | Bump major version, CHANGELOG | wc-mcp-guard | CRITICAL |
| W8 | qwen3 `<think>` leak xuống MCP response | Strip 2 tầng (provider + consumer) | wc-arch-guard | HIGH |
| W9 | Threshold `is_bot_protected()` giảm mà không corpus data | Corpus fixture trước bump | wc-bot-detection-audit | HIGH |
| W10 | `cargo test` skip fail intermittent | Flaky test phải fix root cause, không skip | Manual | HIGH |

## Lưu ý

Nếu ĐÃ chạy `wc-review-v2` Stage 2 (R1-R8) trong session này → C1-C5 chỉ cần verify "đã pass review-v2?" (không cần re-check). Nếu CHƯA chạy review-v2 → check đầy đủ.

---

## 10-item Checklist (CRITICAL)

### C1 — Crate boundary + WASM-safe (CRITICAL)

```bash
grep -rn "use tokio\|use reqwest\|use wreq\|use std::fs\|use std::net" crates/webclaw-core/src/
```

Expected: 0 match. Nếu có → vi phạm invariant, block commit.

### C2 — Dependency direction (CRITICAL)

```bash
cargo tree -p webclaw-core | grep "webclaw-"
```

Expected: no output (core không depend crate nào khác).

`cargo tree -p webclaw-llm | grep -E "wreq|primp"` → expected no output.

### C3 — Patch isolation (CRITICAL)

```bash
grep -rn "patch.crates-io" crates/*/Cargo.toml
```

Expected: no output. `[patch.crates-io]` chỉ ở workspace root `Cargo.toml`.

### C4 — Cargo fmt + clippy (CRITICAL)

```bash
cargo fmt --check
cargo clippy --workspace -- -D warnings
```

Both phải pass với 0 warning.

### C5 — Tests pass (CRITICAL)

```bash
cargo test --workspace
```

Expected: 0 failure. Nếu có flaky test → fix root cause, không skip.

### C6 — Deps audit (IMPORTANT)

```bash
cargo audit           # advisory database
cargo deny check      # license + ban + duplicate
```

Nếu advisory mới → đánh giá impact. Không merge nếu có HIGH severity chưa mitigated.

### C7 — MCP schema stability (CRITICAL nếu chạm mcp)

Nếu sửa `crates/webclaw-mcp/src/server.rs` tool registration:

- Diff tool signature cũ vs mới
- Breaking change (remove field, change type, required→optional) → bump **MAJOR** version MCP server
- Addition-only → bump **MINOR**
- Bug fix behavior → bump **PATCH**

Check bằng `wc-mcp-guard` skill nếu available.

### C8 — CHANGELOG updated (IMPORTANT nếu public-facing)

Nếu commit chạm behavior public (CLI flag, MCP tool, library API) → thêm entry `CHANGELOG.md` section `[Unreleased]`:

- `### Added` — feature mới
- `### Changed` — behavior đổi (breaking hay không)
- `### Fixed` — bug fix
- `### Deprecated` — API sắp bỏ

### C9 — No truncated code (CRITICAL)

Không có placeholder trong code submit:

```bash
grep -rn "// \.\.\." crates/*/src/
grep -rn "// TODO: implement" crates/*/src/
grep -rn "todo!()" crates/*/src/
grep -rn "unimplemented!()" crates/*/src/
```

`todo!()`/`unimplemented!()` chỉ OK trong branch chưa scoped. Nếu file dài → đã dùng Long File Protocol từ `wc-output-guard`.

### C10 — Benchmark regression (IMPORTANT nếu chạm core/extractor/markdown)

Nếu commit chạm `crates/webclaw-core/src/{extractor,markdown,brand,data_island}.rs`:

```bash
cd benchmarks/
cargo run --release -- compare --baseline baseline.json --current current.json
```

Expected: regression <5% trên corpus 50 trang. Nếu >5% → root cause analysis trước commit.

---

## Output Format

```
✓ C1: Crate boundary + WASM-safe — 0 violations
✓ C2: Dependency direction — core: 0 deps, llm: 0 wreq
✓ C3: Patch isolation — no crate-level patch
✓ C4: cargo fmt + clippy — pass, 0 warnings
✓ C5: cargo test --workspace — 45 tests, 0 failures
✓ C6: cargo audit + deny — 0 advisories
✓ C7: MCP schema — unchanged (or bump noted)
✓ C8: CHANGELOG — entry added under [Unreleased]
✓ C9: No truncated code — 0 banned patterns
✓ C10: Benchmark — within 5% of baseline

Verdict: READY TO COMMIT
```

Nếu bất kỳ check FAIL → `Verdict: BLOCKED — fix [items] trước commit`.
