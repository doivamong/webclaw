# Crate Boundaries — webclaw

<!-- Codify Hard Rules từ CLAUDE.md thành rule reference cho skill + hook. -->

Rule này là **nguồn authoritative** cho crate dependency, WASM-safety, patch isolation. Skill `wc-arch-guard` + hook `wasm_boundary_check.py` enforce các rule dưới.

## Dependency direction (một chiều)

```
webclaw-cli ─┐
             ├─→ webclaw-mcp ──→ webclaw-fetch ──┐
             │                      ↓           │
             │                   webclaw-llm    ├─→ webclaw-core
             │                   webclaw-pdf ───┘
             └───────────────────────────────────→ webclaw-core (direct, skip middle layers OK)
```

**Rule:**
- `cli` có thể depend on bất kỳ crate nào
- `mcp` depend on `fetch` + `llm` + `pdf` + `core`
- `fetch` + `llm` + `pdf` chỉ depend on `core`
- `core` KHÔNG depend on bất kỳ webclaw crate nào khác
- **KHÔNG reverse direction** — core không biết về fetch/llm/mcp

**Check:** `cargo tree -p webclaw-core` chỉ thấy third-party deps, không thấy `webclaw-*`.

## Core WASM-safe (INVARIANT TUYỆT ĐỐI)

`crates/webclaw-core/` phải compile trong WASM environment. Implication:

### CẤM import

| Import | Lý do cấm | Replacement |
|--------|-----------|-------------|
| `tokio` | WASM không có tokio runtime | Core là pure parser, không async |
| `reqwest`, `wreq`, `primp`, `hyper` | WASM không có network | Core nhận `&str` HTML, return struct |
| `std::fs`, `std::net`, `std::process` | WASM sandbox | Core không đọc file/network |
| `std::thread`, `std::sync::Mutex` | WASM single-threaded | Dùng `Rc`/`RefCell` nếu cần |
| `std::time::SystemTime` | WASM không có wall clock stable | Dùng `Instant` hoặc caller-provided timestamp |

### CHO PHÉP

| Import | Ghi chú |
|--------|---------|
| `scraper`, `ego-tree`, `html5ever` | HTML parsing (WASM-compatible) |
| `serde`, `serde_json` | Serialization |
| `regex`, `url` | Parse utilities |
| `similar` | Diff engine |
| `rquickjs` (optional feature) | JS evaluation cho data island, WASM-compatible |
| `once_cell`, `lazy_static` | Static init |

### Hook enforcement

`wasm_boundary_check.py` (PostToolUse, Phase 5) grep các file `crates/webclaw-core/src/**/*.rs` cho banned import → block edit với message.

**Escape hatch:** nếu thực sự cần temporary (vd: dev feature behind `#[cfg(feature = "wasm-unsafe")]`), phải có comment `// WASM-BOUNDARY-EXCEPTION: <reason>` trên dòng import + discuss với maintainer trước.

## Patch isolation

`[patch.crates-io]` chỉ ở **workspace root** `Cargo.toml`, KHÔNG ở crate-level.

### Lý do
- primp/wreq patched rustls/h2 forks cần apply toàn workspace đồng bộ
- Patch ở crate-level → cargo resolve conflict, build fail
- Workspace-level → tất cả crate dùng cùng version patched

### Check
```bash
grep -n 'patch.crates-io' D:/webclaw/crates/*/Cargo.toml
# Expected: no match (chỉ workspace root được phép)
```

## webclaw-llm: plain reqwest only

`crates/webclaw-llm/` dùng **plain `reqwest`**, KHÔNG dùng `wreq`/`primp`.

### Lý do
- LLM APIs (OpenAI, Anthropic, Ollama) không có bot protection / TLS fingerprinting
- Impersonation chỉ cần cho web scraping (webclaw-fetch)
- Plain reqwest → ít deps, build nhanh hơn, ít surface attack

### Check
```
grep -l 'wreq\|primp' crates/webclaw-llm/Cargo.toml crates/webclaw-llm/src/**/*.rs
# Expected: no match
```

## qwen3 think-tag strip (2 tầng)

qwen3 model output `<think>...</think>` reasoning tokens. Strip **2 lần** để chắc chắn không leak xuống consumer:

### Tầng 1 — Provider

`crates/webclaw-llm/src/providers/ollama.rs` (hoặc tương tự): sau khi nhận response, strip `<think>...</think>` ngay trong provider.

### Tầng 2 — Consumer

`crates/webclaw-llm/src/chain.rs` hoặc `crates/webclaw-mcp/src/server.rs` (khi gọi LLM): strip lại 1 lần nữa trước khi serialize ra MCP response.

### Lý do 2 tầng
- Defense in depth: nếu provider quên strip (bug), consumer vẫn safe
- Khác model có thể có khác tag style — generic stripper ở consumer phòng hờ

### Regex
```rust
static THINK_TAG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)<think>.*?</think>").unwrap()
});
```

## Feature flags convention

- `default = []` nếu crate không có feature mặc định
- `wasm` feature flag cho opt-in WASM targeting (chỉ `webclaw-core`)
- `llm` feature cho optional LLM chain (consumer opt-in)
- `pdf` feature cho PDF extraction (pdf-extract là native dep, không WASM)

## Crate-level Cargo.toml discipline

### CẤM
- `[patch.crates-io]` (chỉ workspace root)
- `rust-version` khác workspace (inherit từ workspace)
- `edition` khác workspace (inherit)
- Direct duplicate của workspace dep (dùng `.workspace = true`)

### CHO PHÉP
- `[features]` riêng
- `[dependencies]` specific cho crate (không overlap workspace)
- `[dev-dependencies]` cho test
- `[lints]` config riêng (strict clippy level)

## Verify commands

```bash
# Check dependency direction
cargo tree -p webclaw-core | grep "webclaw-"
# Expected: no output (core không depend crate nào khác)

# Check WASM build của core
cargo build -p webclaw-core --target wasm32-unknown-unknown
# Expected: build success (khi có target wasm32)

# Check patch isolation
grep -rn 'patch.crates-io' crates/*/Cargo.toml
# Expected: no output

# Check reqwest vs wreq in llm
cargo tree -p webclaw-llm | grep -E 'wreq|primp'
# Expected: no output
```

## Khi vi phạm

1. **`wasm_boundary_check.py` hook** block edit với message lý do
2. **`wc-arch-guard` skill** trigger khi user edit file trong crate dir → hiển thị rule liên quan
3. **`wc-pre-commit` checklist** verify trước commit: `cargo tree` + WASM build (nếu CI có wasm target)
4. **CI workflow** `.github/workflows/ci.yml` có thể thêm WASM build matrix để enforce ở PR
