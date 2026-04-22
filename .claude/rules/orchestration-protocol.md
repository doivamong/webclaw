# Orchestration Protocol — webclaw

<!-- Adapted from ITF_APP orchestration-protocol.md (user's own work) + inspired by claudekit-engineer (commercial, paraphrased). Rust/webclaw context. Team Mode removed (single-maintainer project). -->

Rules cho việc delegate task tới sub-agents (`Agent` tool) và giữ context sạch giữa main ↔ sub-agent.

## Delegation Context (MANDATORY)

Khi spawn sub-agent qua `Agent` tool, prompt PHẢI bao gồm:

1. **Task cụ thể** — 1-2 câu mô tả rõ scope + acceptance criteria
2. **Files to modify** — list path cụ thể (không "look at codebase")
3. **Files to read for context** — list path để sub-agent đọc nền
4. **Constraints** — webclaw-specific (WASM-safe core, primp isolation, crate direction)
5. **Reports Path** — nếu sub-agent cần output report: `plans/reports/` hoặc task-specific path

**Rule:** Nếu CWD khác với work context, PHẢI pass absolute paths (`D:\webclaw\crates\...`).

## Prompt Template cho sub-agent

```
Task: [specific task mô tả 1-2 câu]

Files to modify:
- [absolute path 1]
- [absolute path 2]

Files to read for context:
- [absolute path 1]
- [absolute path 2]

Acceptance criteria:
- [criterion 1 — observable, testable]
- [criterion 2 — observable]

Constraints (webclaw-specific):
- Core WASM-safe: KHÔNG import tokio/reqwest/wreq/std::fs/std::net vào crates/webclaw-core/
- primp patches ở workspace root, không crate-level
- webclaw-llm plain reqwest (không wreq)
- Dependency direction: cli → mcp → {fetch, llm, pdf} → core
- [rule khác liên quan]

Plan reference (nếu có): [path tới phase-XX.md]

Report output (nếu sub-agent cần write report): [path]
```

## Context Isolation Principle

**Sub-agents không nhận session history.** Prompt PHẢI self-contained.

### Lý do
- Sub-agent có fresh context → nhanh + rẻ
- Pass full history = waste token + sub-agent lạc scope
- Forcing explicit brief → main agent phải nghĩ rõ hơn task

### Anti-pattern ↔ Good pattern

| BAD | GOOD |
|-----|------|
| "Continue from where we left off" | "Implement extraction caching per spec in `plans/phase-02.md`, L45-92" |
| "Fix the issues we discussed" | "Fix panic in `crates/webclaw-core/src/extractor.rs:234`. Root cause: `unwrap()` on empty selector match" |
| "Look at the codebase and figure out" | "Read `crates/webclaw-fetch/src/crawler.rs` `Crawler::crawl()` L80-150, then add `max_depth` parameter" |
| Passing 50+ lines of conversation | 5-line task summary với absolute file paths + acceptance criteria |
| "Do the usual review" | "Run wc-review-v2 3-stage on files: `crates/webclaw-mcp/src/server.rs`. Focus R3 (error propagation) và R6 (MCP schema stability)" |

## Khi nào SHOULD spawn sub-agent

### ✅ Dùng sub-agent khi
- **Independent research** — agent làm xong, report lại, main tiếp tục
- **Parallel exploration** — 3 agents song song tìm ở 3 area khác nhau (crates/core, crates/fetch, crates/mcp)
- **Heavy file scan** — grep + read hàng trăm file → giữ output khỏi main context
- **Specialized task** — wc-rust-expert / wc-mcp-expert / wc-bench-runner cho domain chuyên biệt
- **Context protection** — task lớn không muốn bloat main context

### ❌ KHÔNG dùng sub-agent khi
- Task đơn giản main có thể làm trong 1-2 tool call (over-engineering)
- Cần main agent maintain state across task (sub-agent có fresh context)
- User muốn theo dõi từng bước (sub-agent chỉ return 1 message)
- Task cần interactive confirmation với user (sub-agent không hỏi user được)

## Sequential vs Parallel

### Sequential (default)
Task B cần output của Task A.

```
Agent A: research crate layout option 1 vs 2 → report findings
  ↓
Main: đọc report, decide chọn option
  ↓
Agent B: implement option đã chọn
```

### Parallel
Tasks độc lập, output riêng.

```
Main: spawn agent A + B + C cùng 1 message (multi tool_use blocks)

Agent A: scan crates/webclaw-core/       → report
Agent B: scan crates/webclaw-fetch/      → report   ← parallel
Agent C: scan crates/webclaw-mcp/        → report

Main: synthesize 3 reports
```

**Rule:** Nếu spawn >1 agent, `run_in_background=true` cho agents không block để user không chờ.

## Status protocol cho sub-agents

Khi agent complete, main kỳ vọng 1 trong 4 status:

| Status | Nghĩa | Main action |
|--------|-------|-------------|
| **DONE** | Xong, output attached | Proceed next step |
| **DONE_WITH_CONCERNS** | Xong nhưng flag doubts | Read concerns → address nếu correctness/scope → proceed nếu observational |
| **BLOCKED** | Không complete được | Assess blocker → provide context / break task / escalate user |
| **NEEDS_CONTEXT** | Thiếu info | Provide missing context → re-dispatch |

### Handling rules

- **Never** ignore BLOCKED hoặc NEEDS_CONTEXT
- **Never** force same approach sau BLOCKED (đổi: more context → simpler task → escalate)
- **DONE_WITH_CONCERNS** về file growth / tech debt → note, proceed
- **DONE_WITH_CONCERNS** về correctness → address trước review
- Agent fail 3+ lần → DỪNG, escalate user

## webclaw-specific agent selection

| Task type | Agent đề xuất | Model |
|-----------|---------------|-------|
| Plan feature lớn đa-crate | wc-rust-expert | sonnet |
| Add MCP tool / modify rmcp schema | wc-mcp-expert | sonnet |
| Chạy benchmark corpus, so sánh ground-truth | wc-bench-runner | haiku |
| Debug bug unknown root cause | wc-debug-map (skill) | (caller model) |
| Review code sau implement | wc-review-v2 (skill) | (caller model) |
| Predict risk 5 personas | wc-predict (skill) | (caller model) |
| Research external repo / port code | wc-github-ref (skill) + Explore agent | haiku |
| Hỏi Claude Code / rmcp / MCP spec | claude-code-guide | haiku |

## Delegation example — add MCP tool

```
Agent(
    subagent_type="wc-mcp-expert",
    prompt="""
Task: Thêm MCP tool `diff_markdown` — so sánh 2 markdown snapshot, trả về change set.

Files to modify:
- D:\webclaw\crates\webclaw-mcp\src\server.rs (register tool)
- D:\webclaw\crates\webclaw-core\src\diff.rs (already exists, wire to MCP)

Files to read for context:
- D:\webclaw\crates\webclaw-mcp\src\server.rs (existing tool registration pattern)
- D:\webclaw\crates\webclaw-core\src\diff.rs (diff engine API)
- D:\webclaw\SKILL.md L200-280 (existing tool format)

Acceptance criteria:
- Tool name: `diff_markdown`
- Input schema: { before: string, after: string, mode?: "line" | "word" }
- Output: { additions: string[], deletions: string[], modifications: Array<...> }
- `cargo test -p webclaw-mcp` pass
- `cargo build --release` 6 crate pass

Constraints:
- rmcp 1.2 API (hiện tại), không update rmcp version
- Tool name trong MCP phải unique (check existing)
- JSON schema valid (verify bằng schemars derive)

Plan reference: plans/2026-04-25-diff-mcp-tool/plan.md
"""
)
```

## Không cần (skip cho webclaw)

- **Team Mode** — webclaw 1-maintainer, không có multi-session collaboration
- **SendMessage / TaskList cross-agent** — không có long-running agent team
- **File ownership claims** — single-dev không conflict
- **Shutdown protocol** — sub-agent là one-shot, tự kết thúc
