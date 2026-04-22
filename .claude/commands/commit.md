---
description: "Run wc-pre-commit checklist + conventional commit with ITF attribution"
argument-hint: "[commit message override]"
allowed-tools: Read, Grep, Glob, Bash
---

Hai bước:

## Step 1: Invoke wc-pre-commit

Chạy 10-item checklist:
- C1 crate boundary + WASM-safe
- C2 dependency direction
- C3 patch isolation
- C4 cargo fmt + clippy
- C5 cargo test --workspace
- C6 cargo audit + deny
- C7 MCP schema stability
- C8 CHANGELOG updated
- C9 no truncated code
- C10 benchmark regression

Nếu BẤT KỲ check FAIL → STOP, report issue. User fix trước re-invoke.

## Step 2: Create commit

Nếu all checks PASS:

1. `git status` + `git diff --staged` xem changes
2. `git log -3` xem commit style
3. Draft commit message conventional format:
   ```
   <type>(<scope>): <summary VN có dấu>

   <body — WHY, không WHAT>
   ```
4. Attribution theo `.claude/settings.json` (Co-Authored-By: webclaw-dev)
5. Commit bằng HEREDOC format (preserve newlines)

**Message override**: `$ARGUMENTS` (nếu user cung cấp — skip draft step)

Reference: `.claude/skills/wc-pre-commit/SKILL.md`, `.claude/rules/development-rules.md` (commit format section).
