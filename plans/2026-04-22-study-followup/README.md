# Study Followup — Master Index (2026-04-22)

**Source**: 6-repo study audit (readability ports + Firecrawl positioning + release pattern + TLS tracking).

**Study metadata**: `D:/webclaw/research/github_*/_wc_ref_meta.md`

**Archive**: plan cũ v1 (722 dòng, mixed concerns) lưu tại `plans/.archive-2026-04-22-readability-updates-from-study/plan.md`.

## Plan routing

| File | Concern | Dependencies | Est |
|---|---|---|---|
| `01-quick-wins.md` | 6 commit độc lập, ship TODAY | None | 3-4h total |
| ~~`02-benchmark-harness.md`~~ | ~~Build `webclaw-bench` crate + corpus~~ | **SUPERSEDED → `../2026-04-22-upstream-feature-port/04-benchmark-corpus.md`** | — |
| `03-readability-research.md` | `is_probably_readable` + design audit + perf profile | blocks on upstream-feature-port/04 | 2-3 session |
| `04-release-pipeline.md` | Windows build + UPX + CHANGELOG auto | blocks on upstream-feature-port/02 (3 binary) | 1-2 session |
| `05-tls-tracking.md` | Browser profile audit + upstream cadence | blocks on `01` (docs fix) | 1 session |

## Cross-plan relationship

Sau khi phát hiện fork `doivamong/webclaw` vs upstream `0xMassi/webclaw` drift (v0.4.0 upstream có webclaw-server + targets_1000.txt), tách thêm plan mới `plans/2026-04-22-upstream-feature-port/` chuyên bóc tách upstream feature cho fork. Relationship:

- `upstream-feature-port/04-benchmark-corpus.md` **replaces** study-followup's `02-benchmark-harness.md` (targets_1000.txt tốt hơn manual 18 fixture)
- `upstream-feature-port/02-webclaw-server.md` thêm binary thứ 3 → study-followup `04-release-pipeline.md` F2 packaging update 3 binary
- `upstream-feature-port/01-oss-hygiene.md` đưa CHANGELOG.md → study-followup `04-release-pipeline.md` F4 (git-chglog) bổ trợ generate CHANGELOG tự động
- Study-followup `01-quick-wins.md` commit 4 (docs drift `primp→wreq`) bổ sung: mention `webclaw-server` binary SAU khi upstream-feature-port/02 merge

## Decision matrix

| Item | Source study | Benefit | Cost | Ship timing |
|---|---|---|---|---|
| CJK regex port | llm_readability | HIGH (i18n) | 30 min | 01-quick-wins |
| workspace lints + profile | kreuzberg + Govcraft | HIGH (safety, size) | 30 min | 01-quick-wins |
| ATTRIBUTIONS.md scaffold | kreuzberg | MEDIUM (AGPL hygiene) | 5 min | 01-quick-wins |
| Docs drift `primp`→`wreq` | curl-impersonate study | HIGH (correctness) | 30 min | 01-quick-wins |
| `workflow_dispatch` | Govcraft | LOW (ops flex) | 10 min | 01-quick-wins |
| `[profile.release]` | Govcraft per-package | HIGH (binary size) | 15 min | 01-quick-wins |
| is_probably_readable | dom_smoothie | MEDIUM (batch perf) | 2 session | 03-readability-research |
| Score propagation audit | llm_readability | LOW (design doc only) | 1-2 session | 03-readability-research |
| Regex bench hot-path | dom_smoothie matching.rs | LOW unless bench shows bottleneck | 2 session | 03-readability-research |
| Windows build target | Govcraft | HIGH (user reach) | 1-2 session | 04-release-pipeline |
| UPX compression | Govcraft | MEDIUM (size) | 30 min | 04-release-pipeline |
| CHANGELOG git-chglog | Govcraft | LOW | 1 session | 04-release-pipeline |
| Browser profile audit | curl-impersonate study | MEDIUM | 1 session | 05-tls-tracking |
| Upstream cadence doc | curl-impersonate study | MEDIUM | 30 min | 05-tls-tracking |

## Not in scope (deferred)

- curl-impersonate code port — upstream STALE 2024-07
- Firecrawl agent/interact parity — requires new `webclaw-browser` crate, too big
- rmcp version change — webclaw 1.2 AHEAD of Govcraft 0.1.5
- Nix flake — out of scope
- lol_html streaming, dep swaps (`tl` vs `html5ever`), multi-language bindings — Tier 4

## Execution order (band-based)

```
BAND A — Immediate (parallel possible, no blocker):
├── 01-quick-wins.md  (6 commits, each <1h)
├── 04-release-pipeline.md F1 profile (first commit) ← OVERLAPS with 01 item #6
└── 05-tls-tracking.md D1 docs fix (first commit) ← OVERLAPS with 01 item #4

BAND B — Enablement (blocks Band C):
└── 02-benchmark-harness.md  (1 MANDATORY prerequisite)

BAND C — Research + decision (parallel):
├── 03-readability-research.md A1/B1/C1 (blocks on Band B)
├── 04-release-pipeline.md F2/F3/F4 (independent)
└── 05-tls-tracking.md D2/D3/D4 (independent)

BAND D — Conditional implementation:
└── 03-readability-research.md A2/A3/B2/C2 (decisions after Band C)
```

## Claude Code execution notes

- Mỗi file plan ≤200 dòng, ≤1 concern domain.
- Acceptance criteria **observable + testable** (cargo command chạy pass, file exists, grep count).
- Phase = 1 commit khi có thể. Nếu cần split: phase chia thành sub-step với commit tag rõ.
- Tất cả path absolute (`D:/webclaw/...`).
- Trước implement: `wc-arch-guard` nếu chạm crate boundary, `wc-config-guard` nếu chạm Cargo.toml/config.
- Sau implement: `wc-pre-commit` MANDATORY.
- `wc-extraction-bench` MANDATORY khi chạm `crates/webclaw-core/src/{extractor,markdown,brand,noise,data_island,domain,metadata}.rs` — nhưng chỉ enable SAU khi `02-benchmark-harness.md` ship xong.
