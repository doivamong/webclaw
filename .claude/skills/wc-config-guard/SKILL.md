---
name: wc-config-guard
origin: new
inspired-by: itf-config-guard (structure only, Rust/webclaw content)
user-invocable: false
paths: "Cargo.toml, crates/*/Cargo.toml, .cargo/config.toml, rustfmt.toml, clippy.toml, deny.toml, rust-toolchain.toml"
description: >
  GUARD — PHẢI kiểm tra khi sửa config Rust/Cargo hoặc env var.
  BẮT BUỘC khi: sửa workspace Cargo.toml, crate Cargo.toml (deps, features,
  version), .cargo/config.toml (RUSTFLAGS, target), rustfmt/clippy/deny config,
  env var dùng bởi code (SERPAPI_KEY, OLLAMA_HOST, ANTHROPIC_API_KEY, WEBCLAW_*).
  Ví dụ trigger: "config", "Cargo.toml", "feature flag", "RUSTFLAGS",
  "env var", "dependency version", "workspace dep".
  Priority: ALWAYS check khi user chạm config file.
  DO NOT TRIGGER when: chỉ sửa Rust source code không chạm config.
triggers:
  - "config"
  - "Cargo.toml"
  - "feature flag"
  - "RUSTFLAGS"
  - "env var"
  - "dependency version"
  - "workspace dep"
  - ".cargo/config"
---

Announce: "Đang dùng wc-config-guard — kiểm tra config Cargo/env trước khi sửa."

# webclaw Config Guard

## Hard Rules (CRITICAL)

### R1 — Workspace dependency centralization

Crate shared deps phải khai báo ở `workspace.dependencies` (root Cargo.toml), crate-level dùng `.workspace = true`.

**Current workspace deps** (từ root Cargo.toml):
- `tokio = { version = "1", features = ["full"] }`
- `serde = { version = "1", features = ["derive"] }`
- `serde_json = "1"`
- `thiserror = "2"`
- `tracing = "0.1"`
- `tracing-subscriber = { version = "0.3", features = ["env-filter"] }`
- `clap = { version = "4", features = ["derive", "env"] }`
- `dotenvy = "0.15"`
- Path deps: `webclaw-core`, `webclaw-fetch`, `webclaw-llm`, `webclaw-pdf`

**Rule:**
- Nếu thêm dep dùng >1 crate → khai báo workspace-level, không crate-level duplicate
- Crate-specific dep (chỉ 1 crate dùng) → crate-level OK
- Version range: prefer `"1"` hoặc `"1.0"` (minor update auto), tránh `"*"` (banned) hoặc `"=1.0.3"` (quá strict trừ khi lý do)

### R2 — workspace.package inheritance

Crate Cargo.toml phải inherit từ workspace:

```toml
[package]
name = "webclaw-<crate>"
version.workspace = true        # inherit
edition.workspace = true        # inherit
license.workspace = true        # inherit
repository.workspace = true     # inherit
```

**CẤM override** `version` / `edition` / `license` crate-level trừ khi lý do docs rõ (vd: một crate publish độc lập, khác version).

### R3 — Patch isolation (link H3 wc-arch-guard)

`[patch.crates-io]` chỉ ở workspace root. Không ở crate-level.

**Current patches** (check root Cargo.toml): nếu có section này cho rustls/h2/primp, giữ nguyên. Primp patches phải workspace-wide.

### R4 — RUSTFLAGS / target config

`.cargo/config.toml` hiện minimal:

```toml
# No special build flags needed.
# wreq handles TLS via BoringSSL internally.
```

**Rule:**
- KHÔNG thêm `[build] rustflags` trừ khi có lý do specific (vd: target WASM)
- Nếu cần target-specific: `[target.'cfg(target_os = "windows")']` section riêng
- Không lạm dụng `--cfg` flag (prefer feature flag trong Cargo.toml)

### R5 — Feature flag convention

Default features minimal:

```toml
[features]
default = []                       # prefer empty
llm = ["dep:ollama-rs"]             # opt-in external dep
pdf = ["dep:pdf-extract"]           # opt-in native dep (không WASM)
wasm = []                          # chỉ trong webclaw-core
```

**Rule:**
- `default = []` cho crate publish (consumer chọn feature)
- `default = ["cli"]` chỉ OK cho binary crate
- Feature name phải self-documenting (`llm`, `pdf`, `wasm`, không `extra`, `advanced`)
- Feature dep dùng `dep:foo` syntax (Rust 1.60+) để tránh auto-expose

### R6 — Environment variables

webclaw dùng các env var (check trước khi thêm mới):

| Env var | Dùng bởi | Required? | Default |
|---------|----------|-----------|---------|
| `SERPAPI_KEY` | webclaw-mcp/serpapi.rs, webclaw-fetch/search.rs | Optional (search tool không work nếu missing) | None |
| `OLLAMA_HOST` | webclaw-llm/providers/ollama.rs | Optional | `http://localhost:11434` |
| `ANTHROPIC_API_KEY` | webclaw-llm/providers/anthropic.rs | Optional | None |
| `OPENAI_API_KEY` | webclaw-llm/providers/openai.rs | Optional | None |
| `OLLAMA_RESEARCH_MODEL` | webclaw-mcp research tool | Optional | Provider default |
| `WEBCLAW_API_KEY` | External skill (SKILL.md) | Required cho API consumer | None |
| `WEBCLAW_HOOK_LOGGING` | .claude/hooks/ kill switch | Optional | `1` |

**Rule:**
- Env var PHẢI có fallback graceful (không panic nếu missing, trừ khi critical)
- Dùng `std::env::var(key).ok()` → `Option<String>`, không `.unwrap()`
- Document env var mới trong `CLAUDE.md` + README + `.env.example` (nếu có)
- KHÔNG commit `.env` file (gitignore check)

### R7 — rustfmt / clippy config

`rustfmt.toml` hiện minimal (1 dòng):

```toml
style_edition = "2024"
```

**Rule:**
- Giữ minimal, không custom quá nhiều (đội maintainer có thể khác)
- Nếu thêm config → document lý do trong comment
- `clippy.toml` chưa có — nếu thêm, prefer `msrv = "1.x"` và `cognitive-complexity-threshold`
- `deny.toml` chưa có — nếu thêm `cargo-deny`, start từ `cargo deny init`

## Flow trước khi edit config

```
Claude muốn edit file config
    ↓
File path match?
  - Cargo.toml (workspace root) → R1, R2, R3, R5 check
  - crates/*/Cargo.toml         → R1 (workspace inheritance), R2, R5
  - .cargo/config.toml          → R4 (RUSTFLAGS)
  - rustfmt.toml, clippy.toml   → R7
  - deny.toml                   → R7 + license check
  - .env, .env.example          → R6
    ↓
Nếu thêm dep / feature / env var mới:
  - Verify semver range hợp lý
  - Verify workspace-level nếu shared
  - Verify doc update
    ↓
Nếu OK → allow edit + announce check result
Nếu vi phạm → block + pointer đến rule
```

## DO NOT TRIGGER patterns

| User nói | Skill đúng | KHÔNG dùng config-guard |
|----------|-----------|------------------------|
| "sửa function body trong extractor" | (arch-guard nếu chạm import) | config-guard không liên quan |
| "rename local var" | (edit luôn) | overkill |
| "fix typo comment trong Cargo.toml" | (edit luôn) | overkill |

## Output Format

```
## Config Guard — [file được edit]

Checks applied:
- R1 (workspace centralization): [PASS / FAIL / N/A]
- R2 (package inheritance): [PASS / FAIL / N/A]
- R3 (patch isolation): [PASS / FAIL / N/A]
- R4 (RUSTFLAGS): [PASS / FAIL / N/A]
- R5 (feature flag): [PASS / FAIL / N/A]
- R6 (env var): [PASS / FAIL / N/A]
- R7 (tool config): [PASS / FAIL / N/A]

Violations:
- R1: `crates/webclaw-mcp/Cargo.toml:12 — serde = "1"` duplicate workspace
  → Fix: `serde.workspace = true`

Verdict: ALLOW EDIT | BLOCK EDIT
```

## Integration

- `wc-cook` Step 2 invoke wc-config-guard nếu plan chạm Cargo.toml / env
- `wc-arch-guard` H3 overlap với R3 (patch isolation) — cả 2 check
- `wc-pre-commit` C3 verify `[patch.crates-io]` lần cuối
