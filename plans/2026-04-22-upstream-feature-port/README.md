# Upstream Feature Port — Master Index (2026-04-22)

**Context**: Local `D:/webclaw/` là fork `doivamong/webclaw`. Upstream `0xMassi/webclaw` v0.4.0 (526 stars, released hôm nay) có feature/asset mà fork thiếu. Plan này bóc tách, đánh giá, áp dụng selective — giữ bản sắc fork (skill infrastructure ahead), bổ sung capability từ upstream.

## Fork identity (giữ nguyên)

Fork đã AHEAD upstream ở:
- **Skill infrastructure**: 20 skills (upstream 5), 4 rules, 4 hooks, 3 agents, plan templates
- **Research workflow**: `research/` dir với 8 repo study
- **Plan system**: `plans/` dir với templates (bug-fix, feature-impl, refactor)

KHÔNG port những thứ này từ upstream (fork AHEAD, upstream minimal).

## Upstream assets (đánh giá port)

| Asset | Upstream path | LOC/Size | Decision | Plan file |
|---|---|---|---|---|
| webclaw-server crate (REST API) | `crates/webclaw-server/` | 914 LOC, 14 files | **PORT** | `02-webclaw-server.md` |
| CHANGELOG.md | `CHANGELOG.md` | ~ | **PORT** | `01-oss-hygiene.md` |
| CONTRIBUTING.md | `CONTRIBUTING.md` | ~ | **PORT** | `01-oss-hygiene.md` |
| CODE_OF_CONDUCT.md | `CODE_OF_CONDUCT.md` | ~ | **PORT** | `01-oss-hygiene.md` |
| rustfmt.toml | `rustfmt.toml` | small | **PORT** | `01-oss-hygiene.md` |
| env.example + proxies.example.txt | root | small | **PORT** | `01-oss-hygiene.md` |
| setup.sh | `setup.sh` | small | **PORT** | `01-oss-hygiene.md` |
| examples/ | `examples/` | minimal | **PORT** (+ expand) | `01-oss-hygiene.md` |
| Dockerfile + Dockerfile.ci | root | ~ | **PORT + ADAPT** | `03-docker-distribution.md` |
| docker-compose.yml + docker-entrypoint.sh | root | ~ | **PORT** | `03-docker-distribution.md` |
| packages/create-webclaw (npm npx installer) | `packages/create-webclaw/` | ~ | **PORT + ADAPT** (rename) | `03-docker-distribution.md` |
| SKILL.md (top-level MCP skill manifest) | `SKILL.md` | ~ | **ADAPT** (fork branding) | `03-docker-distribution.md` |
| glama.json | `glama.json` | tiny | **SKIP** (upstream registry, fork không cần list riêng) | — |
| smithery.yaml | `smithery.yaml` | small | **ADAPT** (fork has own if wanted) | `03-docker-distribution.md` |
| assets/demo.gif + demo.mp4 | `assets/` | binary | **SKIP** (fork nên tạo demo riêng) | — |
| deploy/ | `deploy/` | ? | **EVALUATE** post-webclaw-server port | `03-docker-distribution.md` |
| targets_1000.txt (1000 URL labeled) | root | 1000 lines | **PORT → dùng làm corpus seed** | `04-benchmark-corpus.md` |

## Plan routing

| File | Concern | Est | Dependencies |
|---|---|---|---|
| `01-oss-hygiene.md` | 7 file copy/adapt | 2h | None — ship ngay |
| `02-webclaw-server.md` | Port REST API crate (914 LOC) | 1-2 session | `01-oss-hygiene.md` (rustfmt) optional |
| `03-docker-distribution.md` | Docker + SKILL.md + smithery + create-webclaw npm | 1-2 session | `02-webclaw-server.md` (3 binary) |
| `04-benchmark-corpus.md` | targets_1000.txt → fork's benchmark corpus | 1 session | None, parallel OK |

## Cross-plan implications

- **Study-followup plan `02-benchmark-harness.md`**: Được **SUPERSEDE** bởi plan mới `04-benchmark-corpus.md` (upstream đã có `targets_1000.txt` labeled — không cần build from scratch).
- **Study-followup plan `01-quick-wins.md` commit 4** (docs drift `primp→wreq`): Update để bao gồm remote URL fix (`0xMassi` → `doivamong`) + mention `webclaw-server` binary SAU khi plan `02-webclaw-server.md` port xong.
- **Study-followup plan `04-release-pipeline.md` F2** (Windows build): Phải package 3 binary (`webclaw`, `webclaw-mcp`, `webclaw-server`) thay vì 2, SAU khi plan `02-webclaw-server.md` merge.

## Fork decision points cần user xác nhận

1. **Domain/branding**: Upstream dùng `webclaw.io`. Fork có domain riêng? Hay không cần domain?
   - Nếu không: SKILL.md + smithery.yaml cần adapt (remove webclaw.io references)
   - Nếu có: config riêng
2. **npm package name**: Upstream `create-webclaw`. Fork dùng tên khác? (ví dụ `create-webclaw-fork`, `create-<user>-webclaw`, hoặc không publish npm)
3. **Registry listing** (Glama + Smithery): Fork muốn list riêng không? Hay share upstream listing?
4. **GitHub org/user**: Fork dưới `doivamong` — Dockerfile + GHCR image name dùng `doivamong/webclaw` hay name khác?

## Execution order

```
IMMEDIATE (không cần decision):
└── 01-oss-hygiene.md — 7 file copy/adapt ship today (non-branded content)

PARALLEL (không phụ thuộc lẫn nhau):
├── 02-webclaw-server.md — port REST API crate
└── 04-benchmark-corpus.md — port targets_1000.txt + build harness

BLOCKED ON DECISION (fork branding):
└── 03-docker-distribution.md — Docker + SKILL.md + create-webclaw
    │ Cần trả lời decision point 1-4 trước khi thực thi
    └── Có thể start với Dockerfile + compose (non-branded), defer SKILL.md + packages

FINAL:
└── Update CLAUDE.md architecture section: 3 binary (webclaw, webclaw-mcp, webclaw-server)
    + bump Cargo.toml version (proposal: 0.3.4 → 0.4.0-fork.1 hoặc user choose)
```

## Reference files

- Upstream study meta: `research/github_0xMassi_webclaw/_wc_ref_meta.md`
- Upstream Homebrew tap meta: `research/github_0xMassi_homebrew-webclaw/_wc_ref_meta.md`
- Upstream full source: `research/github_0xMassi_webclaw/` (depth-1 clone, freshness check quarterly)

## Out of scope plan này

- Wholesale merge upstream main (fork diverged from upstream — không bulk merge)
- Port fork-only skills up to upstream (khác hướng, upstream PR not fork's concern)
- Commercial closed-source parts của upstream (`api.webclaw.io` anti-bot, JS render, async jobs) — closed source, không accessible
