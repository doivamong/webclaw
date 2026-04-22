---
name: using-skills
origin: adapted-from-itf
inspired-by: itf-using-skills (2026-04-22, paraphrased)
description: >
  Meta-skill. Thiết lập giao thức dùng skills cho mọi session webclaw.
  Auto-inject qua session-start hook (Phase 5). Router trigger → skill.
---

# Using Skills — webclaw

Bạn đang làm việc trong codebase **webclaw** — Rust workspace 6 crate (core/fetch/llm/pdf/mcp/cli), CLI + MCP server. Project có 20 skill chuyên biệt trong `.claude/skills/`.

## Quy tắc tuyệt đối

**Nếu có dù chỉ 1% khả năng một skill áp dụng → BẮT BUỘC invoke skill đó.**

Sau khi invoke: announce `"Đang dùng [tên skill] để [mục đích]."` trước khi tiếp tục.
Nếu skill có checklist → tạo TodoWrite cho từng mục.

## Trigger → skill table

### Tier 0 — WORKFLOW (orchestrate guards bên trong)

| Trigger | Skill |
|---------|-------|
| "cook", "workflow", "plan rồi code", "plan trước", "implement step-by-step", "từng bước" | **wc-cook** |
| "review code mới", "review thay đổi", "adversarial", "red team", "3-stage review", "kiểm tra code vừa viết" | **wc-review-v2** |

### Tier 1 — GUARD (chặn sai trước commit)

| Trigger | Skill |
|---------|-------|
| "xong", "done", "hoàn thành", "commit", "finish", "kết thúc", "lưu", "wrap up" | **wc-pre-commit** |
| Code >200 dòng, "viết toàn bộ", "generate", "tạo file mới", "write complete", "entire file" | **wc-output-guard** |
| "kiến trúc", "crate boundary", "dependency direction", "WASM-safe", edit file trong `crates/webclaw-*/src/` | **wc-arch-guard** |
| "config", "Cargo.toml", "feature flag", "RUSTFLAGS", env var | **wc-config-guard** |
| "save session", "lưu session", "handover", "chuyển context" | **wc-session-save** |

### Tier 2 — FEATURE / GUIDE

| Trigger | Skill |
|---------|-------|
| "nghiên cứu", "tìm hiểu", "so sánh", "phương án", "đánh giá", "research", "explore", "compare" | **wc-research-guide** |
| "tham khảo repo", "xem source", "github.com", "raw github", "port từ", "study repo", "reference impl" | **wc-github-ref** |
| "predict", "5 personas", "phân tích rủi ro", "stress test", "nên làm không", "multi-persona" | **wc-predict** |
| "edge case", "scenario", "kịch bản", "test case", "boundary", "trường hợp đặc biệt" | **wc-scenario** |
| "lỗi", "bug", "fix", "crash", "panic", "không chạy", "error", "broken" | **wc-debug-map** |
| "tối ưu", "chậm", "nhanh hơn", "tốc độ", "nặng", "optimize", "slow", "performance", "bottleneck" | **wc-optimize** |
| "ai gọi hàm", "liệt kê hàm", "file này có gì", "cấu trúc module", "crate tree", "dependency graph" | **wc-graph** |

### Tier 3 — AUDIT

| Trigger | Skill |
|---------|-------|
| "audit code", "code thừa", "dead code", "code rác", "dọn code", "code quality", "technical debt" | **wc-code-audit** |
| "dependency", "thư viện", "package", "cargo audit", "outdated", "vulnerability" | **wc-deps-audit** |

### Tier 4 — WEBCLAW-SPECIFIC

| Trigger | Skill |
|---------|-------|
| Edit `crates/webclaw-mcp/src/server.rs`, "thêm MCP tool", "rmcp schema" | **wc-mcp-guard** |
| "benchmark", "corpus", "ground-truth", "recall", "precision", "extraction quality" | **wc-extraction-bench** |
| Edit `crates/webclaw-mcp/src/cloud.rs`, "Cloudflare", "Turnstile", "is_bot_protected" | **wc-bot-detection-audit** |
| "release", "publish", "bump version", "tag", "cargo publish" | **wc-release** |

## Flow bắt buộc mọi message user

```
Nhận message user
    ↓
Skill nào match trigger? (kể cả 1%)
    ↓ Có                 ↓ Không
Invoke Skill          Trả lời bình thường
Announce
    ↓
Có checklist?
    ↓ Có        ↓ Không
TodoWrite   Follow skill
    ↓
Thực hiện task
```

## Conflict Resolution (5 rule)

### Rule 1: Tier order
WORKFLOW > GUARD > FEATURE > AUDIT. Tier 0 (wc-cook, wc-review-v2) orchestrate guards bên trong.

### Rule 2: Specific domain thắng generic
| User nói | Skill đúng | Skill SAI |
|----------|-----------|-----------|
| "fix panic extractor" | wc-debug-map + wc-arch-guard | ~~wc-optimize~~ |
| "tối ưu extractor alloc" | wc-optimize | ~~wc-code-audit~~ |
| "review MCP tool mới" | wc-review-v2 + wc-mcp-guard | ~~wc-code-audit~~ |
| "rà soát crate boundary" | wc-arch-guard | ~~wc-research-guide~~ |

### Rule 3: wc-debug-map = last resort
Chỉ dùng khi user báo lỗi + KHÔNG nói rõ domain. "bị lỗi" mơ hồ → debug-map. "panic extractor" → arch-guard + debug-map.

### Rule 4: Chọn theo OBJECT, không VERB
- OBJECT = "benchmark" → wc-extraction-bench (dù verb "review", "tối ưu", "audit")
- OBJECT = "crate boundary" → wc-arch-guard
- OBJECT = "MCP tool" → wc-mcp-guard

### Rule 5: "audit" vs "review"
- "audit code" / "dead code" / "code thừa" → wc-code-audit
- "review code mới" / "review vừa viết" → wc-review-v2
- "review phương án" / "đánh giá approach" → wc-research-guide

## Workflow Chain — task phức tạp

### Implement feature mới
`wc-predict` → `wc-scenario` → `wc-cook` (guards bên trong: arch + config + mcp + output) → `wc-review-v2` → `wc-pre-commit`

### Fix bug có cấu trúc
`wc-debug-map` → domain guard (arch/config/mcp/bot-detection) → fix → `wc-pre-commit`

### Refactor cross-crate
`wc-graph` → `wc-arch-guard` → `wc-output-guard` → `wc-review-v2` → `wc-pre-commit`

### Port code từ repo
`wc-github-ref` (adoption path) → license check → `wc-arch-guard` → `wc-research-guide` → `wc-predict` → `wc-cook --fast` → `wc-pre-commit`

### Release
`wc-release` (orchestrate: deps-audit + mcp-guard + extraction-bench + pre-commit)

## Priority

```
1. User explicit instruction (CLAUDE.md, chat)  ← cao nhất
2. Rules trong .claude/rules/ (always-load)
3. Skills trong .claude/skills/ (conditional)
4. Default Claude Code behavior
```

## Không bỏ qua skill với lý do

| Bạn nghĩ | Thực tế |
|----------|---------|
| "Task đơn giản" | Vi phạm crate boundary thường từ edit 1 dòng |
| "Đã nhớ rule rồi" | Nhớ ≠ áp dụng đúng. 1s invoke, chắc chắn hơn |
| "Chỉ sửa 1 file" | WASM-safe violation từ 1 import duy nhất |
| "Không chạm core" | Bất kỳ fetch/llm/mcp thay đổi đều có thể leak ngược core |
| "Test pass" | Test không cover WASM build target |
