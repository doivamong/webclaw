---
name: wc-arch-guard
origin: new
inspired-by: itf-architecture-guard (structure only, Rust/webclaw content)
user-invocable: false
paths: "crates/webclaw-*/src/**, crates/*/Cargo.toml, Cargo.toml"
description: >
  GUARD — PHẢI kiểm tra TRƯỚC KHI viết code thay đổi cấu trúc crate.
  BẮT BUỘC khi: thêm/sửa import vào crates/webclaw-core/, thêm dependency
  vào Cargo.toml, thay đổi dependency direction, sửa [patch.crates-io],
  chạm webclaw-llm (không được dùng wreq), chạm qwen3 think-tag strip logic.
  Quy tắc bất biến: WASM-safe core, dependency direction cli → mcp → fetch/llm/pdf → core,
  primp patches workspace-only, qwen3 strip 2 tầng.
  Ví dụ trigger: "thêm dependency", "sửa Cargo.toml", "crate boundary",
  "WASM-safe", "dependency direction", "import", "feature flag".
  Priority: ALWAYS check FIRST before any structural code changes.
  DO NOT TRIGGER when: chỉ sửa logic bên trong 1 file không chạm import/deps,
  chỉ sửa doc/comment.
triggers:
  - "kiến trúc"
  - "crate boundary"
  - "dependency direction"
  - "WASM-safe"
  - "thêm dependency"
  - "thêm crate"
  - "Cargo.toml"
  - "patch.crates-io"
  - "import"
  - "feature flag"
---

Announce: "Đang dùng wc-arch-guard — kiểm tra invariant cấu trúc trước khi thay đổi."

# webclaw Architecture Guard

**Reference**: [.claude/rules/crate-boundaries.md](../../rules/crate-boundaries.md) là nguồn authoritative.

## Hard Rules (CRITICAL — BLOCK commit nếu vi phạm)

### H1 — Core ZERO network deps (WASM-safe)

`crates/webclaw-core/src/**/*.rs` KHÔNG được import:

| Banned import | Lý do | Replacement |
|---------------|-------|-------------|
| `use tokio` (bất kỳ sub-path) | WASM không có tokio runtime | Core là pure parser |
| `use reqwest`, `use wreq`, `use primp`, `use hyper` | WASM không có network | Nhận `&str` HTML, trả struct |
| `use std::fs`, `use std::net`, `use std::process` | WASM sandbox | Core không I/O |
| `use std::thread`, `use std::sync::Mutex` | WASM single-threaded | `Rc` / `RefCell` nếu cần |
| `use std::time::SystemTime` | WASM không có wall clock stable | `Instant` hoặc caller-provided |

**Check:**

```bash
grep -rn "use tokio\|use reqwest\|use wreq\|use primp\|use hyper\|use std::fs\|use std::net\|use std::thread" crates/webclaw-core/src/
```

Expected: 0 match. Nếu có → fix trước proceed.

**Escape hatch:** Nếu thực sự cần, phải có comment `// WASM-BOUNDARY-EXCEPTION: <reason>` trên dòng import + feature flag `wasm-unsafe` + discuss trước với maintainer.

### H2 — Dependency direction (một chiều)

```
cli → mcp → {fetch, llm, pdf} → core
cli can directly depend on any
```

**CẤM reverse direction:**
- core KHÔNG import webclaw-{fetch, llm, pdf, mcp, cli}
- fetch/llm/pdf KHÔNG import nhau, chỉ core
- mcp có thể import fetch, llm, pdf, core

**Check:**

```bash
cargo tree -p webclaw-core | grep "webclaw-"   # expected: no output
cargo tree -p webclaw-fetch | grep "webclaw-"  # expected: chỉ webclaw-core
cargo tree -p webclaw-llm | grep "webclaw-"    # expected: chỉ webclaw-core
```

### H3 — Patch isolation

`[patch.crates-io]` section CHỈ được ở workspace root `D:\webclaw\Cargo.toml`, KHÔNG ở crate-level.

**Lý do:** primp/wreq patched rustls/h2 forks phải apply đồng bộ toàn workspace. Crate-level patch → cargo resolve conflict, build fail.

**Check:**

```bash
grep -rn "patch.crates-io" crates/*/Cargo.toml
```

Expected: no output.

### H4 — webclaw-llm plain reqwest

`crates/webclaw-llm/` dùng **plain `reqwest`**, KHÔNG `wreq`/`primp`.

**Lý do:** LLM APIs (OpenAI, Anthropic, Ollama) không có bot protection / TLS fingerprinting. Impersonation chỉ cần cho web scraping (webclaw-fetch).

**Check:**

```bash
grep -l "wreq\|primp" crates/webclaw-llm/Cargo.toml crates/webclaw-llm/src/
cargo tree -p webclaw-llm | grep -E "wreq|primp"
```

Both expected: no output.

### H5 — qwen3 think-tag strip 2 tầng

qwen3 model output `<think>...</think>` reasoning tokens. Strip **2 lần**:

- **Tầng 1 — Provider**: trong `crates/webclaw-llm/src/providers/ollama.rs` hoặc provider-specific module, strip ngay trong provider response parsing
- **Tầng 2 — Consumer**: trong `crates/webclaw-llm/src/chain.rs` hoặc `crates/webclaw-mcp/src/server.rs`, strip lại trước khi serialize ra response

**Lý do 2 tầng:** defense in depth — nếu provider quên, consumer vẫn safe.

**Check:** grep cho `<think>` regex strip pattern ở cả 2 location:

```bash
grep -rn "<think>" crates/webclaw-llm/src/
```

Expected: ít nhất 2 occurrence (1 provider + 1 consumer).

## Flow trước khi edit

```
Claude muốn edit file X
    ↓
File X path match crate-boundary quan tâm?
  - crates/webclaw-core/src/**  → check H1 (WASM-safe)
  - crates/webclaw-llm/src/**   → check H4 (no wreq)
  - crates/webclaw-*/Cargo.toml → check H3 (patch isolation) + H2 (deps)
    ↓
Nếu edit THÊM import / dependency:
  - grep banned pattern trước (H1 check)
  - `cargo tree -p <crate>` trước (H2 check)
  - Nếu có violation → REFUSE edit, announce lý do + pointer đến rule
    ↓
Nếu edit OK:
  - Proceed với warning "Đã verify H1-H5"
```

## DO NOT TRIGGER patterns

| User nói | Skill đúng | KHÔNG dùng arch-guard |
|----------|-----------|----------------------|
| "fix typo comment" | (edit luôn) | arch-guard overkill |
| "sửa doc string" | (edit luôn) | arch-guard overkill |
| "rename local variable" | (edit luôn) | arch-guard overkill |
| "thêm #[derive(Debug)]" | (edit luôn) | arch-guard overkill |

**Arch-guard CHỈ trigger khi:**
- Edit `use` statement mới
- Edit `Cargo.toml` `[dependencies]`, `[workspace.dependencies]`, `[patch.*]`, `[features]`
- Đổi `mod` declaration
- Đổi `pub` visibility mức module

## Output Format

```
## Arch Guard — [file được edit]

Checks applied:
- H1 (WASM-safe core): [PASS / FAIL / N/A]
- H2 (dep direction): [PASS / FAIL / N/A]
- H3 (patch isolation): [PASS / FAIL / N/A]
- H4 (llm plain reqwest): [PASS / FAIL / N/A]
- H5 (qwen3 strip 2 tầng): [PASS / FAIL / N/A]

Violations (nếu có):
- H1: [file:line] — `use tokio::runtime::Runtime` trong core
  → Fix: core không dùng tokio. Nếu cần async parser, trả iterator sync.

Verdict: ALLOW EDIT | BLOCK EDIT
```

## Integration

- `wc-cook` Step 2 (Plan) invoke wc-arch-guard nếu plan chạm crate boundary
- `wasm_boundary_check.py` hook (Phase 5) auto-block edit H1 violation
- `wc-review-v2` Stage 2 R1, R5, R7 re-check các hard rule
- `wc-pre-commit` C1, C2, C3 verify lần cuối trước commit
