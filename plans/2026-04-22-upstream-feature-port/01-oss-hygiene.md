# 01 — OSS Hygiene File Ports

**Date**: 2026-04-22
**Type**: File copy/adapt từ upstream
**Status**: Ready — no decision blockers
**Crate(s) affected**: Workspace root (no code change)
**Context**: 7 file copy/minor-adapt từ upstream. Zero code risk, immediate OSS project hygiene.

## Executive Summary

Port 7 nhóm file/directory từ `research/github_0xMassi_webclaw/`. Không code change, chủ yếu doc + config. Ship trong 1-2 session. Mỗi port = 1 commit độc lập.

## Requirements

- [ ] Mỗi commit = 1 file/group độc lập, không bundle
- [ ] Adapt nội dung nếu có upstream-specific reference (domain, org)
- [ ] Không phá vỡ fork's existing files (plans/, research/, .claude/skills/)

## Commits

### Commit 1 — `rustfmt.toml`

**Source**: `research/github_0xMassi_webclaw/rustfmt.toml`
**Target**: `D:/webclaw/rustfmt.toml` (new)

**Action**: Copy verbatim. Rust formatter config không có upstream-specific content.

**Acceptance**:
- [ ] `cargo fmt --check` chạy với config mới
- [ ] `cargo fmt` không change bulk code (hoặc change expected style)

**Commit message**: `chore: add rustfmt.toml from upstream`

### Commit 2 — `env.example` + `proxies.example.txt`

**Source**:
- `research/github_0xMassi_webclaw/env.example`
- `research/github_0xMassi_webclaw/proxies.example.txt`

**Target**:
- `D:/webclaw/env.example`
- `D:/webclaw/proxies.example.txt`

**Action**:
1. Copy từng file
2. Strip upstream-specific values (API key endpoints, webclaw.io URLs) — replace bằng placeholder comment

**Adaptations needed**:
- Env var references `WEBCLAW_API_KEY=wc_...` → keep as is (generic pattern)
- Nếu có mention `https://api.webclaw.io` → comment explain là upstream hosted API, optional

**Acceptance**:
- [ ] Files tồn tại
- [ ] Không chứa real API key hoặc secret

**Commit message**: `docs: add env.example + proxies.example from upstream`

### Commit 3 — `setup.sh`

**Source**: `research/github_0xMassi_webclaw/setup.sh`
**Target**: `D:/webclaw/setup.sh`

**Action**:
1. Copy
2. Review cho upstream-specific references (git clone URL, binary names)
3. Update git clone URL upstream → fork (hoặc giữ upstream cho contributor workflow, depending user preference)
4. `chmod +x setup.sh`

**Acceptance**:
- [ ] `bash setup.sh --help` chạy không lỗi
- [ ] Shebang correct
- [ ] File executable

**Commit message**: `chore: add setup.sh bootstrap from upstream`

### Commit 4 — `CHANGELOG.md`

**Source**: `research/github_0xMassi_webclaw/CHANGELOG.md`
**Target**: `D:/webclaw/CHANGELOG.md` (new)

**Action**:
1. Read upstream CHANGELOG để hiểu format
2. **Option A**: Copy verbatim, add fork entry on top
3. **Option B**: Fresh fork CHANGELOG starting từ fork divergence point (HEAD = skill infra commit)
4. **Recommendation**: Option B — fork có history khác, track từ fork's own commits

**Template** (Option B):
```markdown
# Changelog

All notable changes to this fork. This fork diverged from `0xMassi/webclaw` at upstream v0.3.4 baseline plus skill infrastructure commit (`ea975a3`). Upstream original CHANGELOG at `research/github_0xMassi_webclaw/CHANGELOG.md` for v0.3.4 history.

## [Unreleased]

### Added
- Skill system infrastructure: 20 skills + 4 rules + 4 hooks + 3 agents + 5 commands (ea975a3)
- Research workflow: `research/` dir với 8 external repo studies

## Ported from upstream v0.4.0

_Pending `02-webclaw-server.md` + `04-benchmark-corpus.md` execution._

- Will add: webclaw-server REST API crate (axum, 914 LOC)
- Will add: 1000-URL benchmark corpus (targets_1000.txt)
```

**Acceptance**:
- [ ] CHANGELOG.md exists at root
- [ ] Format phù hợp keep-a-changelog.com

**Commit message**: `docs: init fork CHANGELOG.md`

### Commit 5 — `CONTRIBUTING.md`

**Source**: `research/github_0xMassi_webclaw/CONTRIBUTING.md`
**Target**: `D:/webclaw/CONTRIBUTING.md` (new)

**Action**:
1. Copy
2. **Adapt**:
   - Replace upstream repo URL với fork URL (`0xMassi/webclaw` → `doivamong/webclaw`)
   - Update to reference fork's skills + plan workflow (`.claude/skills/`, `plans/`)
   - Mention `wc-pre-commit` + `wc-review-v2` + `wc-cook` workflow (fork's specialty)
   - If upstream requires CLA or DCO, preserve if fork adopts same policy

**Acceptance**:
- [ ] CONTRIBUTING.md exists
- [ ] References fork's tooling (skills), not just generic cargo workflow
- [ ] Repo URL đúng fork

**Commit message**: `docs: add CONTRIBUTING.md adapted for fork`

### Commit 6 — `CODE_OF_CONDUCT.md`

**Source**: `research/github_0xMassi_webclaw/CODE_OF_CONDUCT.md`
**Target**: `D:/webclaw/CODE_OF_CONDUCT.md` (new)

**Action**: Copy verbatim. Standard Contributor Covenant — no adaptation needed (chỉ contact email nếu upstream hardcode, replace với fork maintainer email).

**Acceptance**:
- [ ] File exists
- [ ] Contact email không còn reference upstream maintainer

**Commit message**: `docs: add CODE_OF_CONDUCT.md`

### Commit 7 — `examples/`

**Source**: `research/github_0xMassi_webclaw/examples/` (chỉ có `README.md` minimal trong upstream)
**Target**: `D:/webclaw/examples/` (new dir)

**Action**:
1. Copy upstream examples/README.md
2. **Expand** với fork's added value:
   - CLI usage examples (từ CLAUDE.md command section)
   - MCP client config examples
   - Per-skill usage examples (tận dụng fork's skill system)
3. Optional: Add `examples/cli/`, `examples/mcp/`, `examples/skills/` subdirs

**Recommendation**: Start minimal, expand organic khi user hỏi/issue request.

**Acceptance**:
- [ ] `examples/README.md` exists
- [ ] Có ít nhất 3 example (scrape, crawl, batch)

**Commit message**: `docs: init examples/ directory with CLI examples`

## Execution sequence

```bash
# Commit 1 (rustfmt)
cp research/github_0xMassi_webclaw/rustfmt.toml .
cargo fmt --check  # verify
git add rustfmt.toml && git commit -m "chore: add rustfmt.toml from upstream"

# Commit 2 (env examples)
cp research/github_0xMassi_webclaw/env.example .
cp research/github_0xMassi_webclaw/proxies.example.txt .
# Review & adapt if needed
git add env.example proxies.example.txt && git commit -m "docs: add env.example + proxies.example from upstream"

# Commit 3 (setup.sh)
cp research/github_0xMassi_webclaw/setup.sh .
chmod +x setup.sh
# Review & adapt
git add setup.sh && git commit -m "chore: add setup.sh bootstrap from upstream"

# Commit 4 (CHANGELOG)
# Write new fork CHANGELOG per Option B
git add CHANGELOG.md && git commit -m "docs: init fork CHANGELOG.md"

# Commit 5 (CONTRIBUTING)
# Copy + adapt
git add CONTRIBUTING.md && git commit -m "docs: add CONTRIBUTING.md adapted for fork"

# Commit 6 (CODE_OF_CONDUCT)
cp research/github_0xMassi_webclaw/CODE_OF_CONDUCT.md .
# Update contact email
git add CODE_OF_CONDUCT.md && git commit -m "docs: add CODE_OF_CONDUCT.md"

# Commit 7 (examples)
mkdir -p examples
# Write README + expand
git add examples/ && git commit -m "docs: init examples/ directory with CLI examples"
```

## Risk Assessment

| Risk | Impact | Mitigation |
|---|---|---|
| Upstream content vi phạm fork branding | Low | Review mỗi file, adapt references |
| License header mismatch | Low | Upstream AGPL-3.0 = fork AGPL-3.0, same license |
| setup.sh Windows compat | Med | Fork user trên Windows — verify bash script chạy WSL/Git Bash |
| CHANGELOG Option A/B choice | Med | Default Option B (fork-fresh), user override if prefer continuity |

## Quick Reference

```bash
# Verify fork state after ports
cargo fmt --check
ls D:/webclaw | sort   # Confirm new files present
git log --oneline -10  # Confirm 7 distinct commits
```

## Next plan

`02-webclaw-server.md` — port REST API crate.
