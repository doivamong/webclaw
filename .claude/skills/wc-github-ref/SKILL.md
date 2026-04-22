---
name: wc-github-ref
origin: adapted-from-itf
inspired-by: itf-github-ref (2026-04-22, paraphrased)
description: >
  USE WHEN user muốn tham khảo code từ repo GitHub khác — đọc snippet, study kiến trúc,
  hoặc port pattern sang webclaw. 3 modes: Lookup / Study / Adoption (guarded gate).
  Ví dụ trigger: "tham khảo repo", "xem source", "github.com", "raw github",
  "port từ", "borrow from", "áp dụng pattern từ", "study repo",
  "đọc kiến trúc repo", "grep repo", "reference implementation".
  Priority: Dùng khi user muốn học/port code từ external repo.
  DO NOT TRIGGER when: nghiên cứu nội bộ webclaw (→ wc-research-guide),
  review code đã viết (→ wc-review-v2).
triggers:
  - "tham khảo repo"
  - "tham khảo github"
  - "xem source"
  - "đọc code repo"
  - "lấy pattern từ"
  - "copy cách làm"
  - "implement giống repo"
  - "migrate từ repo"
  - "port từ"
  - "borrow from"
  - "áp dụng pattern từ"
  - "study repo"
  - "github.com"
  - "raw github"
  - "reference implementation"
---

Announce: "Đang dùng wc-github-ref — tham khảo external repo có cấu trúc."

# webclaw GitHub Reference Guide

Hỗ trợ đọc / study / port code từ repo GitHub khác vào webclaw.

## 1. Intent Classification

| Intent | Heuristic | Output gate |
|---|---|---|
| **Lookup** | "xem file X", URL trỏ thẳng 1 file, "snippet" | Lightweight |
| **Study** | "kiến trúc", "module", "đọc repo", "grep repo" | Lightweight |
| **Adoption** | "port", "adapt", "áp dụng", "implement giống", "migrate từ", "copy cách làm" | **Guarded** (Section 6) |

Còn lại → mặc định **Lookup**. Chỉ hỏi user nếu format ảnh hưởng lớn.

## 2. Fetch Strategy (dùng webclaw MCP tool)

| Số file cần đọc | Tool | Lệnh mẫu |
|---|---|---|
| 1 file / snippet | `webclaw.scrape` URL raw GitHub | `webclaw.scrape("https://raw.githubusercontent.com/owner/repo/<branch>/path/file.rs")` |
| 2-5 file cùng module | `webclaw.batch` | Batch nhiều raw URL |
| 1 topic, đa nguồn | `webclaw.research` | "compare Rust readability libraries 2026" |
| >5 file hoặc cần grep cross-file | `gh repo clone` vào workspace | Xem Section 3 |

**Branch handling:** raw URL PHẢI ghi đúng branch. Verify:

```bash
gh repo view owner/repo --json defaultBranchRef -q .defaultBranchRef.name
```

Hardcode `/main/` sẽ fail silent nếu repo dùng `/master/`.

**Clone theo host:**

```bash
# GitHub
gh repo clone owner/repo D:/webclaw/research/github_owner_repo
git clone --depth 1 https://github.com/owner/repo D:/webclaw/research/github_owner_repo

# GitLab
glab repo clone owner/repo D:/webclaw/research/gitlab_owner_repo
git clone --depth 1 https://gitlab.com/owner/repo D:/webclaw/research/gitlab_owner_repo
```

**Fallback:** chỉ dùng `WebFetch` nếu webclaw fail. KHÔNG dùng `gh repo view` cho file lớn — token-heavy.

## 3. Workspace Convention (BẮT BUỘC)

**Root:** `D:\webclaw\research\` (ngoài crate dir, không làm bẩn workspace webclaw).

**Naming:** `<host>_<owner>_<repo>[__<ref>]\`

```
D:\webclaw\research\
├── github_spider-rs_llm-readability\
├── github_spider-rs_spider\
├── github_letmutex_htmd__v0.1.0\
└── github_niklak_dom_smoothie\
```

`<ref>` thêm khi cần branch/tag/commit cụ thể (`__v0.1.0`, `__main`, `__abc123`).

**Metadata file BẮT BUỘC** — tạo `_wc_ref_meta.md` cạnh repo:

```markdown
# Reference Metadata

- Source URL: https://github.com/owner/repo
- Branch / Tag / Commit: main @ abc123 (cloned 2026-04-22)
- License: [MIT / Apache-2.0 / AGPL / Commercial — block nếu AGPL+ webclaw không compatible]
- Mục tiêu nghiên cứu: [e.g., học readability scoring cho extractor.rs]
- Liên quan webclaw: crates/webclaw-core/src/extractor.rs
- Verdict: lookup / study / adoption-approved / adoption-blocked
```

**Freshness check (BẮT BUỘC trước khi đọc lại repo cũ):**

- Đọc dòng `cloned` trong `_wc_ref_meta.md`
- Nếu >30 ngày → warn user "Repo clone X ngày trước, code có thể đã thay đổi. Refresh không?"
- Refresh: `cd D:/webclaw/research/<dir> && git pull --depth 1` hoặc `git fetch --all && git reset --hard origin/<branch>`
- Sau refresh, cập nhật dòng `cloned` trong metadata

## 4. Extract & Map (output 2 lớp khi adoption)

Sau khi đọc, output theo 2 lớp song song:

### Lớp 1 — Source pattern
Code/structure như repo nguồn viết, không sửa.

### Lớp 2 — webclaw adaptation (bảng map BẮT BUỘC)

| Khía cạnh | Source repo | webclaw convention | Action |
|---|---|---|---|
| Naming | snake_case / camelCase | snake_case (Rust idiom) | Rename |
| Crate structure | monolith lib | cli → mcp → fetch/llm/pdf → core | Re-layer |
| Error handling | `Box<dyn Error>` everywhere | `thiserror` + `Result<T, E>` specific | Wrap |
| Async runtime | tokio explicit | Chỉ `webclaw-mcp` dùng tokio, core không async | Refactor |
| HTTP client | reqwest / hyper | `wreq` trong fetch, `reqwest` trong llm | Replace |
| Deps license | MIT / Apache | AGPL-3.0 block, GPL block | License check |

## 5. Adoption Gate (CRITICAL — 3 checks trước port)

Chỉ khi intent = Adoption:

### Gate 1 — License compatibility

```bash
cat D:/webclaw/research/<repo>/LICENSE
```

| License source repo | webclaw (AGPL-3.0) adopt được? |
|---------------------|-------------------------------|
| MIT / Apache-2.0 / BSD | ✅ Yes — with attribution |
| AGPL-3.0 | ✅ Yes (same license) |
| GPL-3.0 | ⚠ Copyleft conflict — confirm maintainer |
| GPL-2.0 | ❌ Incompatible AGPL — block |
| Proprietary / Commercial | ❌ Block — paraphrase pattern only, no copy |
| Unlicensed | ❌ Block (no grant) |

### Gate 2 — Crate boundary compat

- Pattern có WASM-safe không (nếu port vào core)?
- Dùng dep nào? Compatible với webclaw stack không?
- Async/sync match không?

### Gate 3 — webclaw idiom

- Error type: có `thiserror` wrapper không?
- Module layout: match `crates/webclaw-<crate>/src/` convention?
- Test strategy: inline `#[cfg(test)]` hay `tests/*.rs`?

Nếu bất kỳ gate FAIL → **Verdict: adoption-blocked**. Document lý do trong `_wc_ref_meta.md`.

## 6. Adoption Workflow (guarded)

Khi cả 3 gate PASS:

```
wc-github-ref (adoption intent) → license + boundary + idiom check
  ↓
wc-arch-guard (verify H1-H5 không vi phạm)
  ↓
wc-research-guide (so sánh vs existing webclaw pattern)
  ↓
wc-predict (5 personas stress-test nếu structural)
  ↓
wc-cook --fast (implement, research đã xong)
  ↓
wc-review-v2 (3-stage với focus R1, R5)
  ↓
wc-pre-commit
```

**Attribution tracking:** thêm comment header file port:

```rust
// Adapted from github.com/owner/repo (MIT) — <function/pattern>
// Original: <URL to specific file/line>
// Modified for webclaw: <brief note>
```

## 7. Banned Behaviors

```
- Copy code mà chưa check license
- Port pattern mà skip Adoption Gate 1-3
- "Chỉ copy 1 đoạn nhỏ thôi" — license vẫn apply
- Adopt từ repo proprietary/unlicensed
- Adopt xong không ghi attribution
- Clone repo vào crates/ hoặc workspace root (phải ở research/)
- Skip freshness check cho repo clone >30 ngày
```

## 8. Output Format

```
## GitHub Reference: [owner/repo]

**Intent**: Lookup | Study | Adoption
**URL**: https://github.com/owner/repo
**License**: [type]
**Cloned**: D:/webclaw/research/github_owner_repo (YYYY-MM-DD)

### Findings
[Code read + pattern observed]

### Adoption Gate (nếu Adoption)
- [ ] Gate 1 License: PASS/FAIL ([license type vs AGPL-3.0])
- [ ] Gate 2 Boundary: PASS/FAIL
- [ ] Gate 3 Idiom: PASS/FAIL

Verdict: lookup / study / adoption-approved / adoption-blocked

### Next skill
- Adoption → wc-arch-guard → wc-cook --fast
- Study → document in research notes
- Lookup → done
```
