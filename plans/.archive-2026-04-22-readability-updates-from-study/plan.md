# Plan — Readability Updates từ 5-Repo Study (2026-04-22)

## Context

Audit 5 external Rust/AGPL repo study đã hoàn tất:
- `spider-rs/readability` (aka `llm_readability`)
- `niklak/dom_smoothie`
- `dreampuf/readability-rust`
- `firecrawl/firecrawl`
- `kreuzberg-dev/html-to-markdown`

Metadata: `D:/webclaw/research/github_<owner>_<repo>/_wc_ref_meta.md`.

Audit báo cáo (trước plan này) rank action items thành 4 tier. Plan này scope cho:
- **Tier 1 quick wins** — ship ngay, không cần research
- **Tier 2 items cần research** — 3 item A/B/C dưới đây

Plan KHÔNG cover Tier 3/4 (defer hoặc không recommend).

## Blocking prerequisite — Phase 0

### Phase 0 — Build benchmark corpus (MANDATORY first)

**Status hiện tại**: `D:/webclaw/benchmarks/corpus/` empty. Chỉ có `benchmarks/README.md` root.

**Problem**: Cả 3 item A/B/C đều cần corpus để đo delta extraction quality. Không có corpus = không có evidence để quyết định. Design choice hiện tại của webclaw (`find-best + recovery walks` thay vì Mozilla propagation) chỉ validate được qua benchmark.

**Phase 0 scope:**

1. **Structure corpus folder**:
   ```
   benchmarks/
   ├── corpus/
   │   ├── en/          (English — BBC, NYT, Wikipedia, Medium, tech blog)
   │   ├── cjk/         (Japanese Asahi, Chinese Zhihu, Korean Naver)
   │   ├── docs/        (docs.rs, MDN, Rust book — TOC-heavy)
   │   └── edge/        (login page, error 404, redirect, empty — không-phải-article)
   ├── ground-truth/
   │   └── <same layout as corpus>/<name>.expected.json
   └── harness.rs       (runner)
   ```

2. **Minimum fixtures**:
   - 5 English article (BBC, NYT, Medium, Wikipedia, blog)
   - 3 CJK (1 Nhật, 1 Trung, 1 Hàn — để validate Tier 1 CJK regex)
   - 2 docs (TOC-heavy, validate link density penalty tuning)
   - 3 edge (login, 404, empty — validate `is_probably_readable` threshold)
   - Total: **13 fixtures**

3. **Ground-truth format** (per fixture):
   ```json
   {
     "title": "...",
     "text_contains": ["key phrase 1", "key phrase 2"],
     "text_not_contains": ["sidebar garbage", "footer copyright"],
     "min_score": 50.0,
     "is_readable": true,
     "lang": "en"
   }
   ```

4. **Harness**:
   - Read each fixture HTML
   - Run `webclaw-core::extract_content`
   - Compare against `*.expected.json`
   - Report precision/recall/F1 per fixture + aggregate
   - Output baseline snapshot

**Output Phase 0**: `benchmarks/baseline-2026-04-22.json` + 13 fixtures committed.

**Acceptance**:
- `cargo run -p webclaw-core --example bench_corpus -- --baseline` chạy xong
- Baseline JSON lưu được aggregate metrics
- `wc-extraction-bench` skill có thể run regression check

**Effort**: 1-2 session. Lấy fixtures từ web archive (Wayback Machine) để stable.

**Risk**: Fixture selection bias — chọn trang quá clean sẽ miss real-world noise. Mitigation: 2/5 English fixtures từ trang có nhiều modal/popup/ad (news site thường).

---

## Phase A — `is_probably_readable` fast-path

**Goal**: Filter non-article pages sớm trong batch/crawl workflow, tiết kiệm CPU.

**Reference**: `D:/webclaw/research/github_niklak_dom_smoothie/src/readable.rs` (63 LOC), `config.rs:52-55`.

**License**: MIT → AGPL-3.0 OK với attribution.

**Design decision**: Option A2 + A3 combo (từ Compare table)
- Add `pub fn is_probably_readable(html: &str, opts: &ReadabilityOpts) -> bool` vào `webclaw-core`
- Add `ReadabilityOpts { min_score: f64, min_content_length: usize }` với default từ benchmark
- `ExtractionOptions` gain field optional `readability_threshold: Option<ReadabilityOpts>` (None = skip check, Some = apply)

**Phase A1 — Threshold calibration (depends on Phase 0)**

- Run Phase 0 harness trên corpus, log score distribution:
  - `edge/` fixtures (login, 404, empty) — expect score ~0-10
  - `en/` fixtures — expect score 100-500+ (webclaw scoring scale ≠ Mozilla 20)
  - `docs/` — expect 80-300 (link-dense, penalized)
- Determine threshold sao cho: all `edge/` fail, all `en/` + `docs/` + `cjk/` pass.
- Likely default: `min_score: 30.0`, `min_content_length: 140` (Mozilla default content length giữ, score scale điều chỉnh).

**Phase A2 — Implement `is_probably_readable`**

Files to modify:
- `D:/webclaw/crates/webclaw-core/src/lib.rs` — export `is_probably_readable` + `ReadabilityOpts`
- `D:/webclaw/crates/webclaw-core/src/extractor.rs` — new `is_probably_readable()` reusing `find_best_node` internals

Signature (draft):
```rust
// crates/webclaw-core/src/extractor.rs
pub struct ReadabilityOpts {
    pub min_score: f64,
    pub min_content_length: usize,
}

impl Default for ReadabilityOpts {
    fn default() -> Self {
        Self { min_score: 30.0, min_content_length: 140 }
    }
}

pub fn is_probably_readable(doc: &Html, opts: &ReadabilityOpts) -> bool {
    let Some(best) = find_best_node(doc) else { return false };
    let score = score_node(best);
    if score < opts.min_score { return false }
    let text_len = best.text().collect::<String>().len();
    text_len >= opts.min_content_length
}
```

Constraint: WASM-safe (no tokio/reqwest/fs/net — tất cả đạt). Pure function.

**Phase A3 — Integration point optional**

- `D:/webclaw/crates/webclaw-fetch/src/crawler.rs` — sau khi fetch HTML, nếu `ReadabilityOpts` set, call `is_probably_readable` và skip page nếu false. Default: không apply (zero-cost cho user không opt-in).
- `D:/webclaw/crates/webclaw-mcp/src/server.rs` — optional tool input `skip_non_readable: bool` cho `scrape` tool.

**Phase A Acceptance**:
- Unit test: 3 `edge/` fixtures return false, 10 khác return true
- Regression: `cargo run bench_corpus` cho thấy precision/recall không đổi (is_probably_readable chỉ là gate, không thay đổi extraction)
- Doc comment với reference: `// Inspired by github.com/niklak/dom_smoothie (MIT) — is_probably_readable pattern`

**Phase A Risk**:
- Threshold tuning thấp → lọt false negative (skip article hợp lệ). Mitigation: conservative default (30.0 < typical article 100+).
- Threshold cao → false positive (keep junk). Less critical (junk sẽ noise trong output).

**Effort**: 1 session sau Phase 0.

---

## Phase B — Score propagation design comparison (PIVOT)

**Initial premise (wrong)**: Audit webclaw extractor có parent/grandparent propagation chưa.

**Actual finding**: webclaw **INTENTIONALLY không dùng** Mozilla candidate-propagation. Dùng pattern khác:
1. Single-pass score tất cả candidates (`article, main, div, section, td`)
2. Pick highest scoring standalone
3. Walk sideways để "recover" context: `recover_announcements`, `recover_hero_paragraph`, `recover_section_headings`, `recover_footer_cta`, `collect_sibling_links`

Đây là **DESIGN CHOICE** khác triết lý Mozilla. Không phải bug/gap.

**Phase B scope** (rescoped):

### Phase B1 — Corpus benchmark gap analysis

- Chạy Phase 0 corpus với webclaw baseline
- Identify fixtures score thấp hoặc miss text_contains keyword
- Với mỗi miss case, run same fixture qua dom_smoothie (build CLI binary trong `D:/webclaw/research/github_niklak_dom_smoothie/crates/cli/`)
- So sánh: Mozilla propagation có catch case mà webclaw miss không?

### Phase B2 — Decision tree

```
Miss case analysis:
├── Webclaw miss < 10% corpus → STAY CURRENT (B1), document design trade-off in extractor.rs docstring
├── 10-25% miss → PORT propagation as OPT-IN feature flag
│   └── Add extractor mode `ScoringMode::{FindBest, Mozilla}` trong ExtractionOptions
└── >25% miss → REWRITE find_best_node với propagation as default
    └── MAJOR refactor, yêu cầu wc-predict 5-persona stress test trước
```

**Phase B Acceptance**:
- Quyết định document hóa trong `D:/webclaw/crates/webclaw-core/src/extractor.rs` top-of-file comment (block comment giải thích design choice + trade-off)
- Nếu B2 path: file comparison report `plans/2026-04-22-readability-updates-from-study/score-propagation-gap.md`

**Phase B Risk**:
- CẦN build dom_smoothie CLI trước (workspace member `crates/cli/`). Effort ~30 min.
- Miss-case classification subjective — nên có checklist rõ ("text_contains match", "title correct", "junk excluded") mỗi fixture.

**Effort**: 2-3 session (depends on miss rate).

---

## Phase C — LLM pipeline regex benchmark (RESCOPED)

**Initial premise (wrong)**: `noise.rs` regex hot path.

**Actual finding**: `noise.rs` + `extractor.rs` đã 0 regex (byte-level + token-based). Regex concentrated ở:

| File | Regex count |
|---|---|
| `crates/webclaw-core/src/brand.rs` | 24 |
| `crates/webclaw-core/src/llm/cleanup.rs` | 14 |
| `crates/webclaw-core/src/llm/images.rs` | 12 |
| `crates/webclaw-core/src/markdown.rs` | 10 |
| `crates/webclaw-core/src/llm/body.rs` | 5 |
| `crates/webclaw-core/src/llm/links.rs` | 5 |
| `crates/webclaw-core/src/youtube.rs` | 5 |
| `crates/webclaw-core/src/js_eval.rs` | 2 |

Total: **77 regex occurrences trong webclaw-core**.

### Phase C1 — Flamegraph realistic batch profiling

- Setup: `cargo install flamegraph` (với `perf` trên Linux hoặc DTrace trên macOS — trên Windows cần WSL)
- Profile target: `cargo run --release -p webclaw-cli -- $(cat urls.txt) --urls-file` với 20-30 URL thực
- Capture flamegraph SVG
- Identify top 10 hot functions by wall time

### Phase C2 — Decision

Only refactor regex→byte-level cho functions xuất hiện trong top 10 hot và gọi regex intensively.

Nếu top 10 dominated by:
- `scraper`/`html5ever` DOM parse → network/parse bound, regex không phải bottleneck → **SKIP refactor**
- `llm/cleanup::*` hay `markdown::*` regex → **targeted refactor** (port pattern từ dom_smoothie `matching.rs`)
- Network I/O (tokio, reqwest) → không liên quan extraction logic

### Phase C3 — Refactor (conditional)

Only if C2 identifies regex hot path:
- Port `dom_smoothie/matching.rs` pattern (byte-level functions, không regex)
- Per-function unit test coverage TĂNG CƯỜNG (byte-match catches fewer edge cases than regex)
- Benchmark before/after với criterion

**Phase C Acceptance**:
- Flamegraph SVG committed vào `plans/2026-04-22-readability-updates-from-study/flamegraph-baseline.svg`
- Decision document: `plans/.../regex-bottleneck-analysis.md`
- Nếu C3: criterion bench show ≥15% improvement per targeted function

**Phase C Risk**:
- Flamegraph trên Windows khó setup (cần WSL hoặc port cho DTrace). Mitigation: test trên Linux CI container.
- "Premature optimization" — nếu C2 cho thấy DOM parse dominant, C3 không worth.

**Effort**: 2 session (1 setup + profile, 1 decision/refactor).

---

## Phase D — TLS Impersonation & Upstream Tracking (SCOPE EXPANSION)

**Trigger**: Study `lwthiker/curl-impersonate` (2026-04-22) exposed 3 gap không liên quan readability nhưng bị cùng audit scope.

**Reference**: `D:/webclaw/research/github_lwthiker_curl-impersonate/_wc_ref_meta.md`

### Phase D1 — Docs drift fix (QUICK WIN, independent)

**Problem**: CLAUDE.md + `.claude/rules/crate-boundaries.md` đề cập "primp" nhưng `crates/webclaw-fetch/Cargo.toml:15` thực tế dùng **`wreq = "6.0.0-rc.28"`**.

Cụ thể drift:
- `CLAUDE.md` "Hard Rules": *"primp requires `[patch.crates-io]` for patched rustls/h2 forks at workspace level"* — sai (hiện dùng wreq, workspace Cargo.toml KHÔNG có `[patch.crates-io]`)
- `CLAUDE.md` "Architecture": *"webclaw-fetch/ HTTP client via primp"* — outdated
- `.claude/rules/crate-boundaries.md` H2 section "Patch isolation" — reference primp cần update
- `.claude/rules/crate-boundaries.md` "webclaw-llm: plain reqwest only" check: *"grep 'wreq\|primp' crates/webclaw-llm/"* — vẫn đúng concept nhưng nên nêu wreq trước

**Files to modify:**
- `D:/webclaw/CLAUDE.md` — replace `primp` references with `wreq`, update patch rule description
- `D:/webclaw/.claude/rules/crate-boundaries.md` — update H2 patch isolation section

**Acceptance**:
- `grep -rn 'primp' D:/webclaw/` returns 0 matches (trừ research/ meta files)
- Workspace `Cargo.toml` có `[patch.crates-io]` section hoặc comment giải thích tại sao KHÔNG cần (nếu wreq 6.0 không yêu cầu patched forks nữa)

**Effort**: 30 phút. Doc-only, zero code change. Ship độc lập, không block phase khác.

### Phase D2 — Browser profile coverage audit

**Source of truth shift**: curl-impersonate STALE (max Chrome 116, Firefox 109, released 2024-03). webclaw browser.rs claim Chrome 142, Firefox 144 → đến từ wreq.

**Gap questions**:
1. `BrowserVariant::ChromeMacos` — wreq 6.0.0-rc.28 có actually support Chrome on macOS profile không, hay chỉ enum placeholder?
2. `BrowserVariant::Safari` — wreq có Safari profile hiện đại (18, 17) không?
3. `BrowserVariant::Edge` — wreq có Edge profile nào?

**Approach**:
- Read `crates/webclaw-fetch/src/client.rs` xem `BrowserVariant::X` map đến wreq impersonation enum nào
- Compare với wreq v6.0.0-rc.28 upstream (docs.rs/wreq/6.0.0-rc.28 hoặc `penumbra-x/wreq` repo)
- Liệt kê profile thực sự implemented vs placeholder enum variants

**Files to review (no change):**
- `crates/webclaw-fetch/src/client.rs`
- `crates/webclaw-fetch/src/browser.rs:45-51` — `latest_chrome`, `latest_firefox`

**Output**: `plans/2026-04-22-readability-updates-from-study/browser-profile-matrix.md` — bảng webclaw variant × wreq support × real browser version.

**Acceptance**:
- Matrix commit
- Xác định có enum variant nào là "dead code" (chưa implement thực sự) → flag cho `wc-code-audit` skill xử lý riêng

**Effort**: 1 session.

### Phase D3 — Upstream tracking doc

**Problem**: Skill `wc-bot-detection-audit` reference "curl-impersonate upstream" — implicit assumption curl-impersonate là nguồn track. Thực tế không phải.

**Action**:
- Update `.claude/skills/wc-bot-detection-audit/SKILL.md` (nếu có reference curl-impersonate) → clarify upstream ladder:
  1. **Primary**: `penumbra-x/wreq` (Rust crate webclaw depend trực tiếp) — track release cycle
  2. **Secondary**: `lwthiker/curl-impersonate` — historical reference, **STALE upstream**, không track cho fresh profile
  3. **Tertiary**: scrapfly.io / scrapeops.io blogs — taxonomy & new challenge signature (Cloudflare Turnstile evolution)

**Files to modify:**
- `D:/webclaw/.claude/skills/wc-bot-detection-audit/SKILL.md` (verify tồn tại trước khi edit)

**Acceptance**: Doc rõ rằng curl-impersonate commit history stopped 2024-07, KHÔNG phải nguồn để pull latest Chrome profile.

**Effort**: 20 phút.

### Phase D4 — Refresh cadence process

**Define**:
- Mỗi 4-6 tuần: check `crates.io/crates/wreq` versions, nếu có new release → smoke test webclaw với profile mới
- Mỗi quarter: review scrapfly/scrapeops blog for new Cloudflare challenge types (Turnstile evolution, JA4 changes)
- Monitor Cloudflare public announcements (Cloudflare blog "turnstile", "bot management")

**Action**:
- Add section "Upstream cadence" vào `wc-bot-detection-audit` SKILL.md hoặc new file `.claude/rules/tls-profile-cadence.md`

**Output**: Doc file with checklist + calendar reminder template.

**Effort**: 30 phút.

---

## Phase F — Release Pipeline Hardening (SCOPE EXPANSION)

**Trigger**: Study `Govcraft/rust-docs-mcp-server` (2026-04-22) phát hiện 5 gap trong `D:/webclaw/.github/workflows/release.yml`.

**Reference**: `D:/webclaw/research/github_Govcraft_rust-docs-mcp-server/_wc_ref_meta.md`

**Baseline audit**:

| Item | Webclaw hiện tại | Govcraft pattern |
|---|---|---|
| Windows build | ❌ (gap) | ✅ windows-latest matrix |
| UPX compression | ❌ | ✅ Linux + Windows |
| Release profile | ❌ default | ✅ `opt-level="z" + panic="abort" + lto + strip` |
| CHANGELOG auto-gen | ⚠ `generate_release_notes: true` (thưa) | ✅ `git-chglog` |
| Manual re-trigger | ❌ chỉ tag push | ✅ `workflow_dispatch` |
| Docker multi-arch | ✅ | ❌ |
| Homebrew tap | ✅ | ❌ |
| SHA256 checksums | ✅ | ❌ |

Webclaw AHEAD ở 3 feature (Docker multi-arch, Homebrew, checksums). Govcraft AHEAD ở 5 feature → port selectively.

### Phase F1 — Release profile tuning (IMMEDIATE — consolidate with Tier 1)

**Goal**: Reduce binary size cho both `webclaw` + `webclaw-mcp`.

**Files to modify:**
- `D:/webclaw/Cargo.toml` — add `[profile.release]` section

**Design decision** (speed vs size tradeoff):

**Option F1a — Unified size-optimized** (Govcraft pattern):
```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
panic = "abort"
strip = true
```
Pros: Smallest binaries. Cons: Runtime ~5-10% slower; `panic="abort"` mất stack trace trong panic.

**Option F1b — Unified speed-optimized** (kreuzberg pattern):
```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
strip = true
debug = 1
```
Pros: Fastest. Cons: Larger binary.

**Option F1c — Per-package split** (recommended):
```toml
[profile.release]
opt-level = 3           # base for CLI
lto = "thin"
codegen-units = 1
strip = true

[profile.release.package.webclaw-mcp]
opt-level = "z"         # MCP server prioritizes size
```

**Constraint**: Per-package profile cần Rust 1.57+ (webclaw Rust 2024 edition OK).

**Recommendation**: **F1c** — `webclaw-mcp` user ship qua MCP client cần download nhanh, size ưu tiên. `webclaw` CLI user chạy batch scrape cần speed. Tradeoff balance đúng scope.

**Acceptance**:
- `cargo build --release` cho 2 binary
- `ls -lh target/release/webclaw*` so sánh size trước/sau
- Baseline sample run: `time webclaw https://example.com` không regression >5% wall time

**Effort**: 30 phút.

### Phase F2 — Windows build target (HIGH priority gap)

**Problem**: webclaw user chạy Windows hiện phải build từ source (cần Rust toolchain, cmake, BoringSSL). No pre-built Windows binary released.

**Files to modify:**
- `D:/webclaw/.github/workflows/release.yml` — add Windows matrix entry

**Matrix entry**:
```yaml
- target: x86_64-pc-windows-msvc
  os: windows-latest
```

**BoringSSL build on Windows**: wreq require BoringSSL. Windows-msvc cần MSVC toolchain (included trong windows-latest runner) + cmake. Windows `cmake` có thể install qua `choco install cmake` hoặc `actions-rust-cross` đã bundle.

**Packaging**:
- Binary suffix: `webclaw.exe` + `webclaw-mcp.exe`
- Archive format: `.zip` thay `.tar.gz` (Windows convention)
- Artifact name: `webclaw-${tag}-x86_64-pc-windows-msvc.zip`

**Integration với existing steps**:
- Bash step `Package` cần fallback cho Windows — PowerShell hoặc bash-on-windows:
  ```yaml
  - name: Package (Windows)
    if: runner.os == 'Windows'
    shell: bash
    run: |
      tag="${GITHUB_REF#refs/tags/}"
      staging="webclaw-${tag}-${{ matrix.target }}"
      mkdir "$staging"
      cp "target/${{ matrix.target }}/release/webclaw.exe" "$staging/" 2>/dev/null || true
      cp "target/${{ matrix.target }}/release/webclaw-mcp.exe" "$staging/" 2>/dev/null || true
      cp README.md LICENSE "$staging/"
      7z a "$staging.zip" "$staging"
      echo "ASSET=$staging.zip" >> $GITHUB_ENV
  ```

**Homebrew tap step**: skip cho Windows (chỉ macOS aarch64). Hiện tại đúng rồi.

**Docker step**: Windows không tham gia Docker flow (Linux-only base image).

**Risk**:
- BoringSSL Windows build có thể fail trên runner (đã biết pain point). Mitigation: test qua `workflow_dispatch` trước khi commit vào release.yml main.
- Windows binary signing: không ký, user sẽ thấy SmartScreen warning. Future work: self-signed certificate hoặc user accept risk.

**Acceptance**:
- Test run `workflow_dispatch` trên tag giả (v0.0.0-test)
- Windows binary download được + chạy `webclaw.exe --help` trên Windows 10/11
- Release artifact visible trong `gh release view <tag>`

**Effort**: 1-2 session (test cycle vì runner-bound debugging).

### Phase F3 — UPX compression (MEDIUM priority)

**Goal**: Giảm binary size thêm ~50-70% (sau khi đã `strip=true`).

**Files to modify:**
- `D:/webclaw/.github/workflows/release.yml` — add UPX install + compress steps

**Steps (per platform)**:
```yaml
- name: Install UPX
  if: matrix.target != 'x86_64-apple-darwin' && matrix.target != 'aarch64-apple-darwin'
  shell: bash
  run: |
    if [[ "${{ runner.os }}" == "Linux" ]]; then
      sudo apt-get update && sudo apt-get install -y upx-ucl
    elif [[ "${{ runner.os }}" == "Windows" ]]; then
      choco install upx --no-progress --yes
    fi

- name: Compress with UPX
  if: matrix.target != 'x86_64-apple-darwin' && matrix.target != 'aarch64-apple-darwin'
  shell: bash
  run: |
    BIN_DIR="target/${{ matrix.target }}/release"
    upx --best --lzma "$BIN_DIR/webclaw" 2>/dev/null || upx --best --lzma "$BIN_DIR/webclaw.exe" 2>/dev/null || true
    upx --best --lzma "$BIN_DIR/webclaw-mcp" 2>/dev/null || upx --best --lzma "$BIN_DIR/webclaw-mcp.exe" 2>/dev/null || true
```

**Skip on macOS**: UPX break code signing, mặc dù webclaw không sign binaries, user macOS Gatekeeper có thể reject binary đã UPX. Safer: skip.

**Risk**:
- Anti-virus on Windows có thể false-positive UPX-packed binary. Mitigation: document rõ trong README + link release.
- UPX decompression overhead khi launch (~50-100ms first start). Cho CLI binary acceptable, cho MCP server ít quan trọng (long-running process).

**Acceptance**:
- Binary size giảm ≥40% post-UPX (measurable baseline trước/sau)
- `webclaw --help` chạy OK post-UPX
- `webclaw-mcp` handshake MCP client OK post-UPX

**Effort**: 30 phút (sau khi F2 merge, run cùng test cycle).

### Phase F4 — CHANGELOG auto-generate (LOW priority)

**Goal**: Thay `generate_release_notes: true` (GitHub auto) bằng `git-chglog` (conventional commits → cấu trúc rõ hơn).

**Files to modify:**
- `D:/webclaw/.github/workflows/release.yml` — add git-chglog step trước release create
- `D:/webclaw/.chglog/config.yml` (new) — chglog template
- `D:/webclaw/.chglog/CHANGELOG.tpl.md` (new) — template

**Integration**:
- `wc-release` skill mention CHANGELOG — align với auto-gen step
- Per-version manual CHANGELOG.md commit → auto commit back như Govcraft pattern (line 138-180 release.yml của họ)

**Risk**:
- `git-chglog` require conventional commit format strict. Webclaw `development-rules.md` đã mandate format này.
- Auto-commit back vào main branch cần `GITHUB_TOKEN` permission write (đã có).

**Acceptance**:
- Release note có section Features / Fixes / Docs / Refactor rõ ràng
- CHANGELOG.md committed tự động, không cần manual edit

**Effort**: 1 session.

### Phase F5 — workflow_dispatch manual trigger (LOW priority, quick)

**Files to modify:**
- `D:/webclaw/.github/workflows/release.yml:3-5` — add `workflow_dispatch`

**Snippet**:
```yaml
on:
  push:
    tags: ["v*"]
  workflow_dispatch:
    inputs:
      version:
        description: 'Version tag (e.g., v0.3.5)'
        required: true
        type: string
```

**Use case**: Tag pushed nhưng workflow fail giữa chừng → re-run manual với cùng version.

**Effort**: 10 phút.

---

## Phase E — Bot Protection Corpus Fixtures (extension to Phase 0)

**Trigger**: Phase 0 corpus structure có `edge/` folder — logical expansion include bot-protected pages vì `is_probably_readable` (Phase A) nên phân biệt:
- Empty page (return false — no article)
- 404 page (return false)
- Cloudflare challenge HTML (return false — gated, không phải article)
- Turnstile widget page (return false)
- DataDome challenge (return false)

Hiện tại `wc-bot-detection-audit` skill mention corpus fixture nhưng chưa tồn tại.

### Phase E1 — Fixture collection

**Fixtures cần thêm vào `benchmarks/corpus/edge/bot-protected/`:**

| Fixture | Source | Expected `is_probably_readable` | Expected `is_bot_protected` |
|---|---|---|---|
| `cloudflare-challenge.html` | Real Cloudflare "Checking your browser" page capture | false | true |
| `cloudflare-turnstile.html` | Page gated by Turnstile widget | false | true |
| `datadome-challenge.html` | DataDome interstitial | false | true |
| `akamai-bot-manager.html` | Akamai bot challenge | false | true |
| `perimeter-x-block.html` | PerimeterX (HumanSecurity) block | false | true |

**Ground truth** (`benchmarks/ground-truth/edge/bot-protected/<name>.expected.json`):
```json
{
  "is_probably_readable": false,
  "is_bot_protected": true,
  "provider": "cloudflare",
  "challenge_type": "turnstile",
  "text_contains": ["Checking", "browser", "verify"]
}
```

### Phase E2 — Cross-skill integration

- Phase 0 harness extends: per fixture, chạy cả `extract_content` + `is_bot_protected` (từ `crates/webclaw-mcp/src/cloud.rs`)
- `wc-bot-detection-audit` skill giờ có corpus thật để verify threshold tuning

**Acceptance**:
- 5 bot-protected fixtures commit
- Harness output includes bot-protected detection metrics
- `is_probably_readable` trên bot-protected fixtures = false (cùng behavior như empty/404)

**Effort**: 1 session.

### Phase E Risk

- Capture bot-challenge HTML đòi hỏi reproduce real website trigger. Alternative: synthetic fixtures dựa trên signature đã biết (Cloudflare HTML structure public).
- Challenge HTML có thể outdated nhanh (Cloudflare update signature Q1 2026 → fixture không còn representative). Mitigation: refresh quarterly.

---

## Suggested Phase execution order (UPDATED v2)

```
IMMEDIATE QUICK WINS (ship độc lập, không block):
├── Phase D1 — Docs drift fix (wreq vs primp)
├── Phase F1 — Release profile tuning (size + speed)
├── Phase F5 — workflow_dispatch manual trigger
└── Tier 1 items (CJK regex, workspace lints, ATTRIBUTIONS.md)
   ↓
HIGH-VALUE INFRA:
├── Phase F2 — Windows build target
├── Phase F3 — UPX compression (sau F2 merge)
└── Phase F4 — CHANGELOG git-chglog (độc lập F2/F3)
   ↓
CORPUS (MANDATORY blocks A/B/E):
├── Phase 0 — Readability corpus (13 fixtures)
├── Phase E1 — Bot-protected fixtures (5 fixtures, parallel với 0)
└── Phase D2 — Browser profile audit (parallel, read-only)
   ↓
RESEARCH PHASE (parallel):
├── Phase A1 — is_probably_readable threshold calibration
├── Phase B1 — Readability gap analysis
├── Phase C1 — Flamegraph profile
├── Phase E2 — Cross-skill harness integration
├── Phase D3 — Upstream tracking doc
└── Phase D4 — Refresh cadence process
   ↓
DECISION + IMPLEMENTATION:
├── Phase A2 — Implement is_probably_readable
├── Phase B2 — Readability design decision (likely stay current)
└── Phase C2 — Regex bottleneck decision (likely skip refactor)
   ↓
CONDITIONAL (only if research signals):
├── Phase A3 — Integration point (MCP tool input, crawler filter)
├── Phase B3 — find_best_node rewrite (if miss rate >25%)
└── Phase C3 — regex→byte-level refactor (if flamegraph shows regex hot path)
```

## Linked to Tier 1 quick wins (ship separately, không block plan này)

Các item Tier 1 từ audit (có thể ship parallel với Phase 0):

1. **CJK punctuation regex port** (`llm_readability` MIT attribution) → edit `crates/webclaw-core/src/extractor.rs` `score_node`
2. **Workspace config hardening** (unsafe_code, clippy lints) → edit `D:/webclaw/Cargo.toml`
3. **ATTRIBUTIONS.md scaffold** → new file `D:/webclaw/ATTRIBUTIONS.md`
4. **Phase D1 docs drift fix** (`primp` → `wreq`) → edit `CLAUDE.md` + `.claude/rules/crate-boundaries.md`
5. **Phase F1 release profile** (size+speed per-package) → edit `D:/webclaw/Cargo.toml`
6. **Phase F5 workflow_dispatch** → edit `.github/workflows/release.yml`

Items này không cần corpus — ship ngay khi user approve (6 patches độc lập, mỗi cái <1 session).

## File manifest

| Phase | File sẽ tạo/sửa | Purpose |
|---|---|---|
| 0 | `benchmarks/corpus/<lang>/*.html` (13 files) | Fixtures readability |
| 0 | `benchmarks/ground-truth/<lang>/*.expected.json` | Expected output |
| 0 | `benchmarks/harness.rs` hoặc example | Runner |
| 0 | `benchmarks/baseline-2026-04-22.json` | Baseline snapshot |
| A | `crates/webclaw-core/src/extractor.rs` | `is_probably_readable` fn + `ReadabilityOpts` |
| A | `crates/webclaw-core/src/lib.rs` | Export |
| A | `crates/webclaw-core/src/types.rs` | Optional field trong ExtractionOptions |
| A | `crates/webclaw-fetch/src/crawler.rs` | Integration point |
| A | `crates/webclaw-mcp/src/server.rs` | MCP tool input |
| B | `crates/webclaw-core/src/extractor.rs` | Docstring explaining design choice |
| B | `plans/2026-04-22-readability-updates-from-study/score-propagation-gap.md` | Analysis report |
| C | `plans/2026-04-22-readability-updates-from-study/flamegraph-baseline.svg` | Profile output |
| C | `plans/2026-04-22-readability-updates-from-study/regex-bottleneck-analysis.md` | Decision doc |
| D1 | `CLAUDE.md` + `.claude/rules/crate-boundaries.md` | `primp` → `wreq` docs drift fix |
| D2 | `plans/2026-04-22-readability-updates-from-study/browser-profile-matrix.md` | webclaw variant × wreq support |
| D3 | `.claude/skills/wc-bot-detection-audit/SKILL.md` | Upstream ladder clarify (wreq primary, curl-impersonate stale) |
| D4 | `.claude/rules/tls-profile-cadence.md` hoặc section mở rộng trong `wc-bot-detection-audit` | Refresh cadence checklist |
| E1 | `benchmarks/corpus/edge/bot-protected/*.html` (5 fixtures) | Cloudflare/Turnstile/DataDome/Akamai/PerimeterX |
| E1 | `benchmarks/ground-truth/edge/bot-protected/*.expected.json` | Expected dual flags |
| E2 | `benchmarks/harness.rs` (extend) | Cross-check extract + bot detection |
| F1 | `Cargo.toml` | `[profile.release]` + `[profile.release.package.webclaw-mcp]` size override |
| F2 | `.github/workflows/release.yml` | Add Windows x86_64-pc-windows-msvc matrix + `.zip` packaging |
| F3 | `.github/workflows/release.yml` | UPX install + compress (Linux + Windows, skip macOS) |
| F4 | `.github/workflows/release.yml` + `.chglog/config.yml` + `.chglog/CHANGELOG.tpl.md` | git-chglog auto CHANGELOG |
| F5 | `.github/workflows/release.yml` | `workflow_dispatch` manual trigger |

## Constraints (webclaw hard rules — recap)

- Core WASM-safe: `is_probably_readable` trong webclaw-core phải pure, no tokio/reqwest/fs/net
- Dependency direction: không reverse (core không depend fetch/mcp)
- `[patch.crates-io]` only workspace root
- License: attribution mandatory khi port dom_smoothie/llm_readability pattern
- Pre-commit: `cargo fmt --check` + `cargo clippy -D warnings` + `cargo test --workspace` + `wc-extraction-bench` regression <5%

## Next skill

- Phase 0 (corpus build) — khi user approve: `wc-cook --fast` hoặc `wc-extraction-bench` skill để add fixtures
- Phase A1/B1/C1 (parallel research) — có thể execute sequentially trong cùng session nếu compact
- Phase A2 implementation — `wc-cook --fast` vì research (Phase A1) đã done
- Nếu B2 decision path = rewrite propagation — MANDATORY `wc-predict` 5-persona trước implement (structural change extractor.rs)

## Risks (top-level)

1. **Corpus fixture bias** — chọn sai trang → baseline méo → threshold cả 3 phase sai. Mitigation: 2/5 English từ messy news site.
2. **Phase B rewrite path** — nếu corpus show webclaw miss >25%, rewrite `find_best_node` là structural change lớn. Mitigation: wc-predict gate, alternative = feature-flag opt-in.
3. **Phase C flamegraph Windows** — perf/DTrace không native. Mitigation: WSL hoặc Linux CI.
4. **Scope creep** — plan đã expand từ 3 item (A/B/C) sang 5 (thêm D/E). Mitigation: Phase D1 ship đầu tiên (docs fix, zero code risk), D2-D4 + E độc lập lịch trình với A/B/C.
5. **Phase D2 may reveal dead code** — nếu `ChromeMacos`/`Safari`/`Edge` variants chỉ là placeholder enum → `wc-code-audit` skill xử lý riêng, không phải scope plan này.
6. **Phase E fixture staleness** — bot-challenge HTML có thể outdated (Cloudflare update signatures). Mitigation: quarterly refresh cadence trong Phase D4.
7. **Phase F2 Windows build flaky** — BoringSSL Windows-msvc có known pain points. Mitigation: test qua `workflow_dispatch` trên tag giả trước merge vào release.yml main. Có fallback: skip Windows nếu build consistently fail, document build-from-source instructions.
8. **Phase F3 UPX anti-virus false positive** — Windows Defender có thể flag UPX-packed binary. Mitigation: document rõ trong README + release notes; user có thể download non-UPX binary từ source build.
9. **Scope bloat total** — plan đã expand 3→5→6 phase group (0+A+B+C+D+E+F) + Tier 1 (6 items). Mitigation: Phase F có priority rõ (F1+F5 immediate quick win; F2 infra priority; F3/F4 nice-to-have defer).

## Không trong scope

- Tier 4 items (lol_html, dep swaps, multi-binding) — plan khác
- Firecrawl agent/interact parity — không recommend
- webclaw public API v0.4.0 bump — chờ Phase A3 decide có thêm field ExtractionOptions không
- curl-impersonate code port — **STALE upstream**, MIT compat nhưng KHÔNG giá trị adoption (2024-07 last push, wreq đã ahead)
- wreq fork/patch — nếu wreq upstream có bug cần workaround, xử lý riêng bằng `wc-arch-guard` + `[patch.crates-io]`
- rmcp version bump (Govcraft 0.1.5 vs webclaw 1.2) — webclaw đã AHEAD, no action needed
- Nix flake reproducible build (Govcraft pattern) — out of scope hiện tại
- Windows binary code signing — Phase F2 ship unsigned, SmartScreen warning acceptable cho v0.x. Ship certificate sign trong version 1.0.
- MCP registry submission (webclaw vào awesome-mcp-servers) — marketing task, out of technical plan
