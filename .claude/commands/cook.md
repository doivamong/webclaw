---
description: "Invoke wc-cook — plan-first 7-step implement workflow (interactive/fast/auto mode)"
argument-hint: "[--fast | --auto] <task description>"
allowed-tools: Read, Grep, Glob, Bash, Edit, Write, Agent
---

Activate `wc-cook` skill workflow. Default mode: interactive.

**Mode flag:**
- (no flag) — interactive: approve every step
- `--fast` — skip research, still plan
- `--auto` — auto-approve nếu confidence ≥ 9.5

**Arguments**: `$ARGUMENTS`

Invoke wc-cook skill now. Task description được pass là context cho Step 1 (Scout).

Reference: `.claude/skills/wc-cook/SKILL.md`.
