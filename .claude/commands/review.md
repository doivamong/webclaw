---
description: "Invoke wc-review-v2 — 3-stage review (spec → quality R1-R8 → adversarial)"
argument-hint: "[file path | 'last changes']"
allowed-tools: Read, Grep, Glob, Bash
---

Activate `wc-review-v2` skill for code review.

**Target**: `$ARGUMENTS` (file path, commit hash, hoặc "last changes" cho git diff HEAD~1)

Run 3-stage review protocol:
1. Spec compliance vs plan/requirement
2. Code quality — R1-R8 Rust/webclaw checklist
3. Adversarial red-team (scope gate: skip if ≤2 file ≤30 dòng AND not touching core/mcp)

Output theo SKILL.md format với severity labels (blocking/important/nit/suggestion).

Reference: `.claude/skills/wc-review-v2/SKILL.md`.
