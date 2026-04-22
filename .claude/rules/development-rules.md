# Development Rules — webclaw

<!-- Adapted from ITF_APP development-rules.md (user's own work) + inspired by claudekit-engineer (commercial, paraphrased). Rust/webclaw context. -->

## Nguyên tắc nền tảng

- **YAGNI** — You Aren't Gonna Need It. Không thêm feature/abstraction cho hypothetical future
- **KISS** — Keep It Simple, Stupid. 3 dòng lặp tốt hơn abstraction sớm
- **DRY** — Don't Repeat Yourself. Extract khi có ≥3 duplicate, không extract premature

## Conflict resolution

**Khi development-rules (file này) mâu thuẫn với domain rule webclaw — domain rule THẮNG.**

Domain rules được ưu tiên:
- [crate-boundaries.md](crate-boundaries.md) — WASM-safe core, primp isolation, dependency direction
- [orchestration-protocol.md](orchestration-protocol.md) — sub-agent delegation

## File naming

- **Rust files:** `snake_case.rs` (Rust idiom). Ví dụ: `data_island.rs`, `noise.rs`, `extractor.rs`
- **Cargo packages:** `kebab-case` trong name field, `snake_case` trong lib/bin target. Ví dụ: `webclaw-core` (package) → `webclaw_core` (crate name)
- **Rules/docs:** `kebab-case.md`. Ví dụ: `primary-workflow.md`, `crate-boundaries.md`
- **Skills:** `wc-skill-name/SKILL.md` (lowercase kebab-case, `wc-` prefix)
- **Tên dài OK nếu self-documenting.** `test_extractor_link_density_edge_case.rs` tốt hơn `test_ext.rs`

## File size guidance (KHÔNG phải rule cứng)

**Webclaw có file lớn có chủ đích:**

| File | Dòng | Lý do giữ nguyên |
|------|------|-------------------|
| `crates/webclaw-core/src/extractor.rs` | ~1486 | Readability scoring logic coherent, split = fragment flow |
| `crates/webclaw-core/src/markdown.rs` | ~1431 | HTML→MD conversion với URL resolve, asset collection trong 1 file dễ audit |
| `crates/webclaw-core/src/brand.rs` | ~1340 | Brand extraction (DOM + CSS analysis) self-contained |
| `crates/webclaw-cli/src/main.rs` | ~2372 | Entry point CLI + clap subcommands, section index rõ |

**Khi cân nhắc split:**
- File >500 dòng → xem có section rõ ràng không
- Function >80 dòng → nên tách
- Impl block >300 dòng → nên split thành sub-module

**Dùng `wc-graph` (cargo-modules) để scan.**

## Code quality

### Prioritize
- **Correctness > style** — `cargo test --workspace` pass + `cargo clippy -D warnings` là điều kiện tiên quyết
- **Readability > cleverness** — code rõ > code ngắn
- **Type annotations** cho public API — explicit `fn foo(x: &str) -> Result<Bar, FooError>`
- **Doc comments** cho public item — `/// Summary line. More detail if needed.`

### Avoid
- **Dead code** — xóa ngay, không leave commented-out blocks
- **Wildcard imports** — `use foo::*` — clippy lint `wildcard_imports`
- **Unused imports** — clippy/rustc warn, fix trước commit
- **`unwrap()` trong library code** — chỉ dùng trong `#[cfg(test)]` hoặc `bin` sau khi đã xử lý error
- **Magic numbers** — dùng `const` hoặc config struct
- **`String::new()` khi `""` đủ** — clippy `string_init`

## Error handling

### Boundary validation

Validate ở **system boundary** (user input, external API, HTTP response), không phải internal calls.

```rust
// GOOD — validate tại MCP tool boundary
#[tool]
async fn scrape(&self, url: String) -> Result<ScrapeOutput, McpError> {
    let parsed = Url::parse(&url)
        .map_err(|e| McpError::invalid_params(format!("invalid URL: {e}"), None))?;
    // trust internal calls từ đây
    self.client.fetch(parsed).await
}

// BAD — validate lặp lại mọi layer
async fn fetch(&self, url: Url) -> Result<..> {
    if url.scheme() != "http" && url.scheme() != "https" { // redundant, đã check ở boundary
        ...
    }
}
```

### Exception semantics

- **`Result<T, E>` over panic** trong library code (webclaw-core, webclaw-fetch, webclaw-llm, webclaw-pdf, webclaw-mcp)
- **Specific error types** — dùng `thiserror::Error` derive, không `Box<dyn Error>` trừ khi top-level boundary
- **`?` propagation** — không swallow silently
- **`tracing::error!` hoặc `eprintln!`** cho diagnostic ở CLI, không `println!`

## Testing

### Yêu cầu

- **Test mới cho code mới** (R9 trong wc-review-v2)
- **Test fail trước fix, pass sau fix** — prevent regression (wc-debug-map Step 5)
- **Integration > unit** khi có thể — test real parsing, real markdown output
- **Golden fixtures** cho complex extraction logic — `benchmarks/corpus/` + `benchmarks/ground-truth/`

### Rust test conventions

- **Inline `#[cfg(test)] mod tests`** cho unit test trong `src/*.rs`
- **`tests/*.rs` directory** cho integration test (webclaw-mcp có `serpapi.rs` với `#[tokio::test]`)
- **`benches/*.rs`** cho criterion benchmark (khi cần)
- **`#[tokio::test]`** cho async test, không blocking runtime trong sync test

## Commit messages

### Format (conventional commits)

```
<type>(<scope>): <summary VN có dấu>

<optional body — WHY, không WHAT>
```

### Type (English)

- `feat:` — tính năng mới
- `fix:` — sửa bug
- `docs:` — tài liệu
- `refactor:` — refactor không đổi behavior
- `test:` — thêm/sửa test
- `chore:` — maintenance (deps, config, tooling)
- `perf:` — tối ưu hiệu suất

### Scope (optional)

Crate name (không có prefix `webclaw-`): `core`, `fetch`, `llm`, `pdf`, `mcp`, `cli`.
Hoặc topic: `bench`, `ci`, `docs`, `release`.

Ví dụ: `fix(mcp): Windows HTTPS + Turnstile false positive` (thấy commit `80307d3`).

### Rule

- **Summary tiếng Việt có dấu** (trừ identifier code — `cargo`, `rmcp`, `primp`, etc.)
- **Subject ≤70 ký tự**
- **Body giải thích WHY**, không WHAT (code đã show)
- Attribution theo `.claude/settings.json` — không chèn AI attribution

## Pre-commit

Lớp kiểm tra trước commit (xem `wc-pre-commit` SKILL):

1. `cargo fmt --check`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo test --workspace`
4. `cargo audit` (security advisory)
5. `cargo deny check` (license + ban + duplicate)
6. MCP tool schema không đổi (nếu đổi → bump version)
7. CHANGELOG updated
8. Version bump consistent 6 crate (nếu release)
9. Benchmark regression <5% (nếu chạm `webclaw-core/`)
10. `CLAUDE.md` + `.claude/rules/` sync

**Không bypass** `--no-verify` trừ khi user request rõ.

## Web research

Skill `wc-research-guide` + `wc-github-ref` ưu tiên `webclaw` MCP tools (dogfooding):

- `webclaw.search` cho keyword search
- `webclaw.scrape` cho 1 URL
- `webclaw.batch` cho nhiều URL parallel
- `webclaw.research` cho deep research

**Fallback:** WebSearch/WebFetch khi webclaw tool fail.

## Các rule TUYỆT ĐỐI (trùng với CLAUDE.md)

Xem [CLAUDE.md](../../CLAUDE.md) section "Hard Rules". Tóm tắt:

1. **Core ZERO network dependencies** — `crates/webclaw-core/` không import tokio/reqwest/wreq/std::fs/std::net. WASM-compatible.
2. **primp patches workspace-level** — `[patch.crates-io]` chỉ ở workspace root Cargo.toml
3. **webclaw-llm dùng plain reqwest** — LLM APIs không cần TLS fingerprinting
4. **qwen3 `<think>` strip 2 tầng** — provider + consumer level
5. **Dependency direction:** `cli → mcp → {fetch, llm, pdf} → core`. Không reverse.

Vi phạm → `wasm_boundary_check.py` hook + `wc-arch-guard` skill block. Fix trước commit.
