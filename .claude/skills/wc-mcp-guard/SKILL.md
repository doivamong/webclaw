---
name: wc-mcp-guard
origin: new
description: >
  GUARD — PHẢI kiểm tra khi thêm/sửa MCP tool trong crates/webclaw-mcp/.
  Verify rmcp API compat, JSON schema valid, tool name unique, version bump semver.
  USE WHEN: edit crates/webclaw-mcp/src/server.rs hoặc tools/*.rs,
  thêm MCP tool mới, sửa tool schema (input/output), bump rmcp version.
  Ví dụ trigger: "thêm MCP tool", "rmcp schema", "tool registration",
  "MCP server", "tool output format", "ServerCapabilities".
  Priority: LUÔN check khi chạm webclaw-mcp.
  DO NOT TRIGGER when: chỉ sửa logic internal của tool không đổi signature/schema.
paths: "crates/webclaw-mcp/src/**, crates/webclaw-mcp/Cargo.toml"
triggers:
  - "thêm MCP tool"
  - "sửa MCP tool"
  - "rmcp schema"
  - "tool registration"
  - "ServerCapabilities"
  - "MCP server"
  - "tool output format"
  - "tool input schema"
---

Announce: "Đang dùng wc-mcp-guard — verify MCP tool schema + rmcp compat."

# webclaw MCP Guard

## Hard Rules (CRITICAL)

### M1 — Tool name unique

MCP spec yêu cầu tool name unique trong server. Webclaw-mcp hiện có **10 tool**: scrape, search, research, crawl, map, batch, extract, summarize, diff, brand.

**Check trước thêm tool mới:**

```bash
grep -n '#\[tool(' crates/webclaw-mcp/src/server.rs
# hoặc
grep -rn '#\[tool(' crates/webclaw-mcp/src/tools/
```

Tên mới KHÔNG trùng. Nếu muốn reuse logic khác param → tạo subcommand trong tool hiện tại, không tool mới.

### M2 — JSON schema valid (schemars)

Tool input struct PHẢI derive `schemars::JsonSchema`:

```rust
#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ScrapeInput {
    /// URL to scrape
    pub url: String,
    /// Output formats (markdown, text, llm, json)
    #[serde(default = "default_formats")]
    pub formats: Vec<String>,
    /// CSS selectors to include
    #[serde(default)]
    pub include_selectors: Vec<String>,
}
```

**Required fields:**
- Doc comment trên mỗi field → appear trong schema description
- `#[serde(default)]` cho optional field
- `#[serde(default = "fn_name")]` cho custom default
- Derive `Deserialize` (required) + `JsonSchema` (required) + `Debug` (recommended) + `Clone` (if owned downstream)

**Check schema generate:**

```bash
cargo run -p webclaw-mcp -- list-tools --json | jq '.tools[0].input_schema'
# (nếu binary có list-tools command)
```

### M3 — rmcp version pin consistent

```toml
# crates/webclaw-mcp/Cargo.toml
[dependencies]
rmcp = "1.2"   # MAJOR.MINOR pin, patch auto-update
```

**Nếu bump rmcp:**
- Minor (1.2 → 1.3): review [rmcp CHANGELOG](https://github.com/modelcontextprotocol/rust-sdk), có thể breaking
- Major (1.x → 2.x): breaking, toàn bộ tool code cần audit

KHÔNG bump rmcp trong cùng commit thêm tool — split 2 commit riêng.

### M4 — Tool output stable (semver)

Tool output = public API. Breaking change rules:

| Change | Impact | Version bump |
|--------|--------|--------------|
| Thêm optional field | Non-breaking (client không expect) | MINOR |
| Thêm required field | Breaking | MAJOR |
| Rename field | Breaking | MAJOR |
| Đổi type (string → int) | Breaking | MAJOR |
| Remove field | Breaking | MAJOR |
| Đổi semantic (value meaning) | Breaking even same shape | MAJOR |

**Action khi breaking:**
- Bump version workspace root (`Cargo.toml`: `version = "0.4.0"` if 0.3.x)
- Update CHANGELOG với migration note
- Invoke `wc-release` skill

### M5 — Error handling consistent

Tool return type phải là `Result<CallToolResult, ErrorData>`:

```rust
#[tool(description = "Scrape a URL")]
async fn scrape(
    &self,
    Parameters(input): Parameters<ScrapeInput>,
) -> Result<CallToolResult, ErrorData> {
    validate_url(&input.url)
        .map_err(|e| ErrorData::invalid_params(e, None))?;
    // ...
    Ok(CallToolResult::success(vec![Content::text(json!({...}).to_string())]))
}
```

**CẤM:**
- `.unwrap()` trên parsed input (panic → MCP client crash)
- Silent `let _ = ...` swallow error
- Leak secret trong error message (API key, URL với token)

### M6 — Input validation boundary

Validate input NGAY ở tool entry, không defer xuống internal:

```rust
// GOOD
async fn scrape(Parameters(input): Parameters<ScrapeInput>) -> Result<...> {
    validate_url(&input.url).map_err(|e| ErrorData::invalid_params(e, None))?;
    // trust from here
    self.client.fetch(&input.url).await
}

// BAD — validate lặp ở internal
fn fetch(&self, url: &str) -> Result<...> {
    if !url.starts_with("http") { return Err(...) }  // redundant
    ...
}
```

Validation reference: [crates/webclaw-mcp/src/server.rs `validate_url()`](../../../crates/webclaw-mcp/src/server.rs).

### M7 — Async + Send/Sync

Tool fn là `async fn` trong `impl WebclawMcp`. Self (`&self`) phải Sync + Send:

- `Arc<T>` cho shared state (OK nếu T: Send + Sync)
- Tránh `Mutex<T>` hold across `.await` (use `tokio::sync::Mutex` nếu cần)
- `Option<T>` lazy init OK nếu init at startup

### M8 — qwen3 think-tag strip

Nếu tool gọi LLM provider chain (research, summarize, extract) → strip `<think>` trước return:

```rust
let raw_output = self.llm_chain.as_ref()
    .ok_or_else(|| ErrorData::internal_error("LLM not configured"))?
    .complete(prompt).await?;

// Strip think-tag (tầng 2, consumer-side)
static THINK_TAG: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)<think>.*?</think>").unwrap()
});
let cleaned = THINK_TAG.replace_all(&raw_output, "");
```

Refer [crate-boundaries.md H5](../../rules/crate-boundaries.md).

## Flow khi thêm tool mới

```
1. Đặt tên tool — check M1 unique
2. Design input struct — M2 schema + validation
3. Design output type — M4 semver consideration
4. Implement handler — M5 error handling + M6 validation + M7 async
5. Test với MCP client:
   - claude_desktop_config.json add webclaw entry
   - Reload Claude Desktop
   - Invoke tool, verify schema + behavior
6. Document trong SKILL.md (D:\webclaw\SKILL.md external skill file)
7. CHANGELOG entry
8. → wc-review-v2 → wc-pre-commit
```

## Tool Inventory (hiện tại, 10 tool)

| Tool | Purpose | Input key fields |
|------|---------|-----------------|
| scrape | Extract single URL | url, formats, include/exclude_selectors, only_main_content |
| search | SerpAPI Google search | query, num_results, country, language, recency |
| research | Multi-source research | query, deep, topic |
| crawl | BFS site crawl | url, depth, max_pages, same_origin |
| map | Sitemap discovery | url |
| batch | Parallel multi-URL scrape | urls[] |
| extract | JSON schema / prompt extraction | url, schema hoặc prompt |
| summarize | LLM summary | url, max_length |
| diff | Content change tracking | url, snapshot_path |
| brand | Brand identity extraction | url |

## DO NOT TRIGGER

| User nói | Skill đúng | KHÔNG dùng wc-mcp-guard |
|----------|-----------|------------------------|
| "sửa logic scrape" (không đổi schema) | (edit trực tiếp) | mcp-guard overkill |
| "sửa SerpAPI key env var" | wc-config-guard | mcp không liên quan |
| "add tool mới" | wc-mcp-guard + wc-cook | OK trigger |

## Output Format

```
## MCP Guard — [tool name / change]

Checks applied:
- M1 (unique name): [PASS / FAIL]
- M2 (schema valid): [PASS / FAIL]
- M3 (rmcp pin): [PASS / FAIL / N/A]
- M4 (semver): [non-breaking / MINOR / MAJOR]
- M5 (error handling): [PASS / FAIL]
- M6 (input validation): [PASS / FAIL]
- M7 (async Send/Sync): [PASS / FAIL]
- M8 (think-tag strip): [PASS / FAIL / N/A]

Violations:
- [M#]: [file:line] — [issue]
  → Fix: [action]

Version impact: [patch / minor / major bump]

Verdict: ALLOW EDIT | BLOCK EDIT
```

## Integration

- `wc-cook` Step 2 invoke wc-mcp-guard nếu plan thêm/sửa MCP tool
- `wc-review-v2` Stage 2 R6 overlap — schema stability check
- `wc-pre-commit` C7 verify schema unchanged (hoặc bump documented)
- `wc-release` check version bump consistent với breaking change nào
