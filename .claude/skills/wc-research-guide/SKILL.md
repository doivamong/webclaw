---
name: wc-research-guide
origin: adapted-from-itf
inspired-by: itf-research-guide (2026-04-22, paraphrased)
description: >
  USE WHEN user cần nghiên cứu, so sánh phương án, hoặc đánh giá trước khi implement.
  Hiểu trước khi làm. Đọc code hiện tại trước khi đề xuất.
  Ví dụ trigger: "nghiên cứu", "tìm hiểu", "so sánh", "phương án",
  "đánh giá", "rà soát", "kiểm tra", "review", "khảo sát",
  "thiết kế logic", "research", "explore", "compare", "evaluate".
  Priority: Dùng cho research/analysis TRƯỚC implement.
  DO NOT TRIGGER when: "audit code"/"rà soát code"/"code thừa" (→ wc-code-audit),
  "review code mới"/"adversarial" (→ wc-review-v2),
  "tham khảo repo"/"port từ github" (→ wc-github-ref).
triggers:
  - "nghiên cứu"
  - "tìm hiểu"
  - "so sánh"
  - "phương án"
  - "đánh giá"
  - "rà soát"
  - "kiểm tra"
  - "review"
  - "khảo sát"
  - "research"
  - "explore"
  - "compare"
  - "evaluate"
---

Announce: "Đang dùng wc-research-guide — 4 bước research có cấu trúc."

# webclaw Research Guide

**Nguyên tắc:** Hiểu trước khi làm. Đọc code hiện tại trước khi đề xuất.

---

## Web Research Tools (dogfooding)

Khi cần thông tin web, **ưu tiên webclaw MCP tools**:

- `webclaw.research` — nghiên cứu sâu 1 chủ đề (search + fetch + synthesis, 1 call)
- `webclaw.search` — tìm kiếm nhanh (Google via SerpAPI, 8s, $0)
- `webclaw.scrape` — đọc nội dung 1 URL (readability extraction)
- `webclaw.batch` — đọc nhiều URL parallel

**KHÔNG dùng** WebSearch/WebFetch trừ khi webclaw tool fail hoặc hết quota.

## Quy trình 4 bước bắt buộc

### Bước 1 — Scope (xác định phạm vi)

- User muốn nghiên cứu gì? Crate nào? Toàn workspace hay 1 module?
- Liệt kê file/module liên quan (`cargo-modules generate tree -p <crate>`)
- Xác định output mong đợi: báo cáo / so sánh / thiết kế / recommendation?

### Bước 2 — Explore (đọc code hiện tại)

**BẮT BUỘC đọc trước khi đề xuất:**

```
CLAUDE.md                         → Architecture rules, hard rules
.claude/rules/crate-boundaries.md → WASM-safe, dep direction, patch isolation
.claude/rules/development-rules.md → YAGNI/KISS/DRY + Rust idiom
Cargo.toml (workspace root)       → Dependency inventory
crates/<crate>/Cargo.toml         → Crate deps, features
```

**Đọc code thực tế:**

- File liên quan module đang research
- Pattern tương tự đã implement trong codebase (vd: tương tự `extractor.rs` cho parser mới)
- Test files cho module (`#[cfg(test)] mod tests` hoặc `tests/*.rs`)
- Git log: `git log --oneline -10 -- crates/<crate>/` để hiểu evolution

### Bước 3 — Compare (so sánh khi có nhiều phương án)

Khi có >1 phương án → **BẮT BUỘC lập bảng so sánh**:

```markdown
| Tiêu chí | Phương án A | Phương án B |
|----------|-------------|-------------|
| Phù hợp crate boundary (WASM-safe?) | ? | ? |
| Complexity (thêm module / bỏ module) | ? | ? |
| Performance (latency, alloc) | ? | ? |
| Maintainability (readability, testability) | ? | ? |
| Số file cần sửa | ? | ? |
| Dep thêm (size, license) | ? | ? |
| Breaking change public API | ? | ? |
```

### Bước 4 — Recommend (đề xuất)

- Chọn phương án, giải thích lý do dựa trên tradeoff
- Liệt kê file cần sửa (absolute path)
- Cảnh báo nếu vi phạm [crate-boundaries.md](../../rules/crate-boundaries.md) hoặc [development-rules.md](../../rules/development-rules.md)
- Link tới rule/doc liên quan

---

## webclaw-specific checks khi research

| Khi research về... | Luôn kiểm tra |
|-------------------|---------------|
| Module mới trong core | WASM-safe (không tokio/reqwest/fs/net) |
| Deps mới (crate từ crates.io) | License (GPL/AGPL block), size impact, maintained status |
| MCP tool mới | rmcp 1.2 API compat, JSON schema valid, tool name unique |
| Extractor/parser change | Benchmark corpus regression risk |
| Provider LLM mới | Plain reqwest only (không wreq), qwen3 think-tag strip |
| Bot detection threshold | Corpus fixture data, scrapfly/scrapeops reference |
| Refactor cross-crate | Dependency direction (cli→mcp→fetch/llm/pdf→core) |

## Khi "thiết kế" không liên quan UI

Webclaw không có UI. "Thiết kế" luôn là logic/kiến trúc → skill này.

Ví dụ:
- "thiết kế provider chain mới" → wc-research-guide
- "thiết kế tool MCP retry logic" → wc-research-guide + wc-predict
- "thiết kế benchmark harness mới" → wc-research-guide + wc-extraction-bench

## Banned behaviors khi research

```
- Đề xuất mà chưa đọc code hiện tại
- So sánh phương án mà không có bảng
- Bỏ qua crate boundary / WASM-safe khi đánh giá
- Đề xuất thêm crate từ crates.io mà chưa check license + maintained status
- "Theo kinh nghiệm thì..." mà không có evidence từ codebase/docs
- Copy blindly từ repo khác — luôn check license + adapt Rust idiom
```

## Output Format

```
## Research Report: [topic]

### Scope
[Context: crate, module, output mong đợi]

### Existing code analysis
- File A (`crates/<crate>/src/<file>.rs`): [1-line summary]
- Pattern similar: [existing module for reference]

### Comparison (nếu có >1 phương án)
[Table theo format trên]

### Recommendation
**Choice**: [Option X]
**Reason**: [2-3 câu tradeoff]
**Files to modify**: [list]
**Risks**: [list]
**Next skill**: wc-predict (nếu risky) → wc-cook (implement)
```

## Kết hợp

| Sau research | Skill tiếp | Khi nào |
|-------------|-----------|---------|
| Recommendation có risk | wc-predict | Feature lớn / structural change |
| Recommendation có edge case | wc-scenario | Stateful / multi-input |
| Recommendation OK proceed | wc-cook --fast | Research đã xong, implement |
