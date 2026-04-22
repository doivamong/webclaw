---
name: wc-session-save
origin: adapted-from-itf
inspired-by: itf-session-save (2026-04-22, paraphrased)
description: >
  Lưu structured session context trước khi chuyển account hoặc kết thúc session phức tạp.
  Tạo HANDOVER.md với decisions, WHY, failed approaches, next steps.
  USE WHEN: "save session", "lưu session", "backup session", "chuyển account",
  "hết quota", "switch account", "handover", "save context".
  DO NOT TRIGGER when: đang /checkpoint đơn giản, chưa có work đáng kể trong session.
triggers:
  - "save session"
  - "lưu session"
  - "backup session"
  - "chuyển account"
  - "hết quota"
  - "switch account"
  - "lưu context"
  - "handover"
  - "save context"
---

Announce: "Đang dùng wc-session-save — tạo handover context cho session tiếp theo."

# webclaw Session Save

Lưu session context có cấu trúc để khôi phục trong session mới (khác account, khác máy, hoặc sau compact).

## Khi nào dùng

- Trước khi chuyển account (hết quota)
- Trước khi kết thúc session phức tạp (nhiều decisions, nhiều files changed)
- Khi muốn handover cho session tiếp theo
- Sau multi-phase work (vd: Phase 0-5 skill implementation)

## File output

Tạo `D:\webclaw\HANDOVER.md` (overwrite OK — version cũ trong git). Gitignored theo `.gitignore`.

## Structure (CRITICAL)

```markdown
# HANDOVER — [session title]

**Date**: YYYY-MM-DD HH:MM
**Account**: [account transition reason nếu có]
**Session duration**: [X hours]

## Session Intent
[1-2 câu mô tả mục tiêu session này]

## Files Modified
- `crates/webclaw-<crate>/src/...rs` — [1-line what changed]
- `.claude/skills/<name>/SKILL.md` — [1-line]
(full list, không rút gọn)

## Decisions Made
1. **[Decision]** — WHY: [reason]. Alternative considered: [...]. Outcome: [...]
2. ...

## Failed Approaches (quan trọng — tránh lặp lại)
- **[Approach]** — đã thử, không work vì [reason]. Kết luận: [takeaway]

## Current State
- ✅ Completed: [list]
- 🚧 In-progress: [list with file:line pointer nếu có]
- ⏸ Blocked: [list + blocker]

## Next Steps
1. [Next action — cụ thể, có file path nếu có]
2. [Next action]

## Verification Commands
```bash
cd D:/webclaw
cargo check --workspace       # expected: pass
cargo test --workspace        # expected: X tests, 0 failures
[other commands to run]
```

## Context để session mới load
- Read these files first: [priority order]
- Skip these: [files không relevant]
- Recent commits: `git log --oneline -5`
```

## Anchored Iterative Summary (IMPORTANT)

Khi viết "Current State" + "Next Steps" — theo template này để session mới load nhanh:

- **Anchor 1**: Session started at `[commit hash]`
- **Anchor 2**: Session ended with file tree at `[state]`
- **Artifacts**: list file đã tạo/sửa (full path, không rút gọn)
- **Decisions log**: chronological order, mỗi decision có WHY

**Compression target**: ~500 từ total, preserve decision context.

## Không skip

- **Failed approaches** — session mới sẽ lặp lại mistake nếu không ghi
- **Why** cho mỗi decision — "what" code đã show, "why" chỉ có trong context
- **Verification commands** — session mới cần chạy gì để confirm resume đúng state

## Output Format

Sau khi tạo file, report:

```
✓ HANDOVER.md saved at D:\webclaw\HANDOVER.md
  - 12 files modified
  - 5 decisions logged
  - 2 failed approaches noted
  - 3 next steps queued

Session mới: mở Claude Code tại D:\webclaw, nói "đọc HANDOVER.md và tiếp tục"
```
