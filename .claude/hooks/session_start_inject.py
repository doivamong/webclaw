#!/usr/bin/env python3
"""SessionStart hook — inject using-skills meta-skill summary into context.

Event: SessionStart
Behavior: emit additionalContext with condensed using-skills trigger table,
so Claude knows which skill to invoke without having to read the full SKILL.md.

Output format: Claude Code hook spec expects JSON with key `hookSpecificOutput.additionalContext`.

Cached per-session via env var `WEBCLAW_SESSION_ID` if set, else no cache.
"""

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from lib.hook_lib import hook_main, log_hook_event


# Condensed trigger table — inject vào context mỗi session start.
# KEEP TERSE. Full detail in .claude/skills/using-skills/SKILL.md.
_CONTEXT = """# webclaw skill quick reference (auto-injected)

You are in the webclaw codebase (Rust workspace, 6 crate, CLI + MCP server).
20 skills live in .claude/skills/. Invoke by announcing "Đang dùng <skill> để <purpose>."

Tier 0 — WORKFLOW (plan + review orchestrators):
- wc-cook         → "cook", "plan rồi code", "implement step-by-step"
- wc-review-v2    → "review code mới", "adversarial", "3-stage review"

Tier 1 — GUARD (block wrong before commit):
- wc-pre-commit   → "xong", "done", "commit", "hoàn thành"
- wc-output-guard → viết file >200 dòng, "generate", "tạo file mới"
- wc-arch-guard   → edit crate boundary, WASM-safe, dep direction, patch isolation
- wc-config-guard → edit Cargo.toml, RUSTFLAGS, env var
- wc-session-save → "save session", "handover", "hết quota"

Tier 2 — FEATURE / GUIDE:
- wc-research-guide → "nghiên cứu", "so sánh", "đánh giá"
- wc-github-ref     → "tham khảo repo", "github.com", "port từ"
- wc-predict        → "5 personas", "phân tích rủi ro", "nên làm không"
- wc-scenario       → "edge case", "kịch bản", "test case"
- wc-debug-map      → "lỗi", "panic", "crash", "bug"
- wc-optimize       → "tối ưu", "chậm", "performance"
- wc-graph          → "cấu trúc", "ai gọi", "module tree"

Tier 3 — AUDIT:
- wc-code-audit     → "audit code", "dead code", "code thừa", "clippy pedantic"
- wc-deps-audit     → "dependency", "cargo audit", "vulnerability", "license"

Tier 4 — WEBCLAW-SPECIFIC:
- wc-mcp-guard              → edit crates/webclaw-mcp/, "thêm MCP tool", "rmcp schema"
- wc-extraction-bench       → "benchmark", "corpus", "ground-truth", "recall"
- wc-bot-detection-audit    → edit crates/webclaw-mcp/src/cloud.rs, "Cloudflare", "Turnstile"
- wc-release                → "release", "publish", "bump version", "tag"

Rules (always-load from .claude/rules/):
- primary-workflow.md        — 7 task-type → skill chain
- development-rules.md       — YAGNI/KISS/DRY + Rust idiom
- orchestration-protocol.md  — sub-agent delegation context
- crate-boundaries.md        — WASM-safe core + dep direction + patch isolation

Hooks (auto-fire):
- PostToolUse Edit|Write on *.rs           → cargo_fmt_check.py (warn only)
- PreToolUse Write|Edit                     → secret_scanner.py (block API key)
- PostToolUse Edit|Write on webclaw-core/** → wasm_boundary_check.py (block tokio/reqwest)

Conflict resolution (from using-skills §Conflict Resolution):
1. Tier 0 > 1 > 2 > 3
2. Specific domain (arch-guard, mcp-guard) beats generic (debug-map)
3. Object > Verb (e.g., "review benchmark" → extraction-bench, not review-v2)
"""


@hook_main("session-start-inject")
def main() -> None:
    output = {
        "hookSpecificOutput": {
            "hookEventName": "SessionStart",
            "additionalContext": _CONTEXT,
        }
    }
    sys.stdout.write(json.dumps(output))
    sys.stdout.flush()

    log_hook_event(
        "session-start-inject",
        "info",
        f"injected {len(_CONTEXT)} chars of skill context",
    )


if __name__ == "__main__":
    main()
