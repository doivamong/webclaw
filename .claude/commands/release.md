---
description: "Invoke wc-release — 12-step release workflow (version bump → cargo publish → gh release)"
argument-hint: "[major|minor|patch | --dry-run]"
allowed-tools: Read, Grep, Glob, Bash, Edit
---

Orchestrate release workflow cho webclaw.

**Mode:**
- `patch` — bug fix only, 0.x.y → 0.x.(y+1)
- `minor` — new feature non-breaking, 0.x.y → 0.(x+1).0 (pre-1.0: also breaking bump here)
- `major` — stable ≥1.0 breaking change, x.y.z → (x+1).0.0
- `--dry-run` — run Steps 1-6 only, skip commit/tag/publish

**Arguments**: `$ARGUMENTS`

## 12 Steps (reference wc-release SKILL.md)

1. Semver decide (analyze `git log v<last>...HEAD`)
2. Pre-flight audits (cargo audit, clippy, test, fmt, deny, corpus bench)
3. Version bump workspace Cargo.toml
4. CHANGELOG finalize ([Unreleased] → [X.Y.Z])
5. Corpus bench vs released baseline
6. `cargo publish --dry-run` topo order (core → fetch/llm/pdf → mcp → cli)
7. Commit release chore
8. Git tag + push
9. CI multi-target binary build (macOS x86_64/arm64, Linux x86_64/arm64, Windows x86_64)
10. `gh release create` + attach binary
11. `cargo publish` live (same topo order, 30s pause between)
12. MCP registry submit (modelcontextprotocol/servers, mcpservers.org, awesome-mcp-servers)

**Blocking gates:**
- Step 2: any audit fail → BLOCK
- Step 5: regression >5% → BLOCK
- Step 6: any crate dry-run fail → BLOCK

**Rollback:** patch release preferred. Yank CHỈ cho security/data-corruption.

Reference: `.claude/skills/wc-release/SKILL.md`.
