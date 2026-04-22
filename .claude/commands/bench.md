---
description: "Invoke wc-extraction-bench + wc-bench-runner agent — corpus regression check"
argument-hint: "[--save <name> | --compare <baseline>]"
allowed-tools: Read, Grep, Glob, Bash, Agent
---

Run extraction benchmark corpus — verify quality/speed/bot-bypass delta.

**Mode:**
- (no flag) — run current bench, save /tmp/current.json, compare với `benchmarks/baseline.json`
- `--save <name>` — run bench, save as `benchmarks/<name>.json` (commit khi stable)
- `--compare <baseline>` — compare two saved results

**Arguments**: `$ARGUMENTS`

Delegate execution tới `wc-bench-runner` agent (haiku model, read-only). Agent report:
- Overall quality delta
- Per-category (news/docs/blogs/spa/ecommerce/edge_cases)
- Per-page regressions >5%
- Speed p50/p95
- Bot bypass rate (CF, DataDome)

Main agent decide action:
- STABLE → proceed
- WATCH (1-5% regression) → document, proceed
- BLOCK (>5% regression) → invoke wc-debug-map để root cause

Reference:
- `.claude/skills/wc-extraction-bench/SKILL.md` — tolerance + regression sources
- `.claude/agents/wc-bench-runner.md` — runner delegation spec
