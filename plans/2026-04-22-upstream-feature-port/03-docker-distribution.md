# 03 — Docker + Distribution Surface

**Date**: 2026-04-22
**Type**: Packaging + distribution infrastructure
**Status**: Blocked on `02-webclaw-server.md` (need 3 binary ready) + fork branding decisions
**Crate(s) affected**: None (config + doc files)
**Context**: Upstream có Docker stack + registry manifests (SKILL.md, smithery.yaml, glama.json) + npm package `create-webclaw`. Port selective + adapt cho fork branding.

## Executive Summary

Port 4 group asset distribution: (a) Docker stack, (b) top-level SKILL.md (Claude/Anthropic MCP skill marketplace), (c) Smithery/Glama MCP registry configs, (d) npm package `create-webclaw`. Mỗi group có adaptation decision (brand/domain/org name).

## Decision points (blockers)

**Phải trả lời trước khi execute**:

1. **Fork domain**: Fork có dùng `webclaw.io` (upstream's domain) không?
   - Không có → SKILL.md + smithery.yaml adapt: remove `webclaw.io` references, point to fork repo
   - Có (user own) → config với domain mới
2. **npm package name**: `create-webclaw` (upstream) hay tên khác?
   - Recommend: `create-webclaw-fork` or `@doivamong/create-webclaw` hoặc SKIP npm publish cho fork (nếu chỉ là personal fork không distribute)
3. **GHCR image name**: Current release.yml dùng `ghcr.io/0xmassi/webclaw` → fork cần rename `ghcr.io/doivamong/webclaw`
4. **Registry listing** (Smithery + Glama): List riêng fork? Hay không list, ride trên upstream?

## Commits

### Commit 1 — Dockerfile + Dockerfile.ci (port)

**Source**:
- `research/github_0xMassi_webclaw/Dockerfile`
- `research/github_0xMassi_webclaw/Dockerfile.ci`

**Target**:
- `D:/webclaw/Dockerfile`
- `D:/webclaw/Dockerfile.ci`

**Adaptations**:
1. `FROM ghcr.io/0xmassi/webclaw:latest` (nếu reference upstream image) → fork equivalent hoặc rebuild fresh
2. `COPY target/release/webclaw target/release/webclaw-mcp target/release/webclaw-server /usr/local/bin/` — 3 binary (upstream đã update qua commit `ccdb6d3`)
3. Image labels (`LABEL org.opencontainers.image.source=...`) → update fork repo URL
4. CMD + ENTRYPOINT per upstream (smart entrypoint pattern — xem docker-entrypoint.sh)

**Acceptance**:
- [ ] `docker build -t webclaw-fork:test -f Dockerfile .` succeed
- [ ] `docker run webclaw-fork:test --help` work
- [ ] `docker run webclaw-fork:test https://example.com` work

**Commit message**: `feat(docker): port Dockerfile + Dockerfile.ci covering 3 binary`

### Commit 2 — docker-compose.yml + docker-entrypoint.sh

**Source**:
- `research/github_0xMassi_webclaw/docker-compose.yml`
- `research/github_0xMassi_webclaw/docker-entrypoint.sh`

**Target**:
- `D:/webclaw/docker-compose.yml`
- `D:/webclaw/docker-entrypoint.sh`

**Adaptations**:
1. `docker-entrypoint.sh`: Smart routing shim (preserve upstream logic). Review first 30 dòng đã đọc — pattern handle URL/flag vs bash/command distinction. Copy verbatim, `chmod +x`.
2. `docker-compose.yml`: Review cho upstream-specific:
   - Image name (`ghcr.io/0xmassi/webclaw`) → `ghcr.io/doivamong/webclaw` hoặc local `webclaw-fork:latest`
   - Volume mounts (if reference upstream path) → adapt
   - Service name (`webclaw` or `webclaw-server`) — preserve
   - Port mapping for webclaw-server REST API (usually `3000:3000`)

**Acceptance**:
- [ ] `docker-compose up` chạy trên fork
- [ ] docker-entrypoint.sh tests (từ upstream comment block):
  ```
  docker run IMAGE https://example.com          → webclaw https://example.com
  docker run IMAGE --help                       → webclaw --help
  docker run IMAGE bash                         → bash
  docker run IMAGE                              → webclaw --help (default CMD)
  ```

**Commit message**: `feat(docker): port docker-compose.yml + smart entrypoint`

### Commit 3 — top-level SKILL.md (fork branding)

**Source**: `research/github_0xMassi_webclaw/SKILL.md`

**Target**: `D:/webclaw/SKILL.md`

**Background**: Upstream's top-level SKILL.md là **Claude/Anthropic agent-facing skill manifest**. Khác `.claude/skills/` (IDE-internal skills). Format:
```yaml
---
name: webclaw
description: ...
homepage: https://webclaw.io
user-invocable: true
metadata: {...}
---

# webclaw
...
## API base
All requests go to `https://api.webclaw.io/v1/`.
```

**Adaptations critical**:
1. Nếu fork KHÔNG có hosted API: Entire "API base" section cần rewrite:
   - Option A: Point to local `webclaw-server` (self-host): `http://localhost:3000/v1/`
   - Option B: SKIP API base section — fork chỉ có CLI + MCP (skill không claim REST)
   - **Recommended**: Option B. Fork không chạy hosted API commercial.
2. Endpoints documentation: Keep structure but note "via webclaw-mcp MCP tool hoặc local self-host webclaw-server"
3. `homepage:` field — fork repo URL hoặc remove
4. `metadata` → update install pointing to fork's npm package (nếu có) hoặc binary release

**Acceptance**:
- [ ] SKILL.md exists at root
- [ ] KHÔNG reference `api.webclaw.io` (fork không own)
- [ ] Install instructions đúng fork's binary release hoặc local build
- [ ] YAML frontmatter valid

**Commit message**: `feat: add top-level SKILL.md (fork-branded Claude skill manifest)`

### Commit 4 — smithery.yaml (fork-branded)

**Source**: `research/github_0xMassi_webclaw/smithery.yaml`

**Target**: `D:/webclaw/smithery.yaml`

**Adaptations**:
1. `description` field: adapt (remove "webclaw.io" reference, focus fork's value prop)
2. `apiKey` config schema: Nếu fork không có hosted API, either:
   - Remove apiKey schema (simpler)
   - Keep nhưng comment nó là optional for local usage
3. `commandFunction` keep as is — `webclaw-mcp` binary name unchanged

**Option B (defer entirely)**: Nếu fork không muốn list Smithery registry, SKIP file. Revisit khi có demand.

**Acceptance**:
- [ ] smithery.yaml valid (test at https://smithery.ai/ playground)
- [ ] Fork listing reflects real capabilities, không claim upstream features

**Commit message**: `feat: add smithery.yaml MCP registry config`

### Commit 5 — glama.json (optional)

**Source**: `research/github_0xMassi_webclaw/glama.json`

**Content upstream** (very minimal):
```json
{
  "$schema": "https://glama.ai/mcp/schemas/server.json",
  "maintainers": ["0xMassi"]
}
```

**Decision**: Fork SKIP hoặc create own với `"maintainers": ["doivamong"]`.

**Recommendation**: **SKIP** — fork không submit Glama registry. Upstream owns that listing. Fork users install via binary release hoặc source build.

**No commit**.

### Commit 6 — packages/create-webclaw npm installer (optional)

**Source**: `research/github_0xMassi_webclaw/packages/create-webclaw/`

**Investigation needed**:
- Read `packages/create-webclaw/package.json` để hiểu full scope
- Determine if npm package downloads upstream binary hoặc fork's
- Fork probably wants SKIP (không publish npm) HOẶC rename với scope (`@doivamong/create-webclaw`)

**Recommendation for fork**: **SKIP initially**. Fork users who want npx installer — revisit sau khi release workflow + GHCR image stable.

**Defer this commit** unless user explicitly request.

## Updated release.yml (cross-reference)

Study-followup `04-release-pipeline.md` F2 (Windows build) + F3 (UPX) assume **2 binary**. Post-`02-webclaw-server.md` merge:
- Update F2 packaging step cover 3 binary
- Update F3 UPX step cover 3 binary
- Update Docker job `COPY` step cover 3 binary (qua this plan's commit 1-2)
- Update Homebrew formula action — auto-update via `bump-homebrew-formula-action` — verify formula template có `bin.install "webclaw-server"` (upstream v0.4.0 formula đã có, nhưng fork's tap nếu có, phải update manually lần đầu)

## Acceptance (overall)

- [ ] `docker build` succeed cho Dockerfile + Dockerfile.ci
- [ ] `docker run` smart entrypoint tests pass (6 cases từ upstream comment)
- [ ] SKILL.md reflect fork's capability (không claim upstream hosted API)
- [ ] smithery.yaml valid (nếu ship)
- [ ] glama.json + create-webclaw SKIP documented trong CLAUDE.md

## Risk Assessment

| Risk | Impact | Mitigation |
|---|---|---|
| Docker build 3 binary time lâu | Low | CI cache, don't run per-PR |
| SKILL.md confuse user thinking fork = upstream | High | Explicit "fork" language, clear API endpoint description |
| smithery.yaml schema version drift | Low | Pin config schema, follow Smithery docs |
| Port npm package pollute fork with JS tooling | Med | Defer commit 6 |

## Quick Reference

```bash
# After commit 1-2 (Docker)
docker build -t webclaw-fork:test .
docker run webclaw-fork:test --help
docker-compose up

# After commit 3 (SKILL.md)
# Verify YAML frontmatter
head -10 SKILL.md

# After commit 4 (smithery.yaml)
# Submit to Smithery (if listing)
curl -F "yaml=@smithery.yaml" https://smithery.ai/api/submit
```

## Next

Sau commit 1-4, plan này done. Update fork's README (separate task) để reflect new Docker + REST API capability.
