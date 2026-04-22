#!/usr/bin/env python3
"""PostToolUse hook — block Edit/Write that violates WASM-safe invariant.

Matcher: Edit | Write on crates/webclaw-core/src/**.rs
Behavior: BLOCK edit (exit 2) if file introduces banned import.

Banned imports trong webclaw-core (reference .claude/rules/crate-boundaries.md H1):
- tokio (runtime not in WASM)
- reqwest, wreq, primp, hyper (network not in WASM)
- std::fs, std::net, std::process, std::thread (sandbox)
- std::time::SystemTime (no stable wall clock)

Escape hatch: line có comment `// WASM-BOUNDARY-EXCEPTION: <reason>` được skip.

Kill switch (emergency):
  WEBCLAW_WASM_CHECK=0 → disable block (still logs)
"""

import os
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from lib.hook_lib import hook_main, read_hook_input, log_hook_event


# Banned import patterns (match `use X...` statements)
_BANNED = [
    (r"^\s*use\s+tokio(\b|::)", "tokio (async runtime not WASM-compatible)"),
    (r"^\s*use\s+reqwest(\b|::)", "reqwest (network not in WASM)"),
    (r"^\s*use\s+wreq(\b|::)", "wreq (network, wreq is for fetch crate only)"),
    (r"^\s*use\s+primp(\b|::)", "primp (TLS impersonation, fetch only)"),
    (r"^\s*use\s+hyper(\b|::)", "hyper (low-level HTTP, fetch only)"),
    (r"^\s*use\s+std::fs(\b|::)", "std::fs (no filesystem in WASM)"),
    (r"^\s*use\s+std::net(\b|::)", "std::net (no network in WASM)"),
    (r"^\s*use\s+std::process(\b|::)", "std::process (no subprocess in WASM)"),
    (r"^\s*use\s+std::thread(\b|::)", "std::thread (WASM single-threaded)"),
    # std::time::SystemTime allowed in some WASM targets, warn only via soft check
]

_EXCEPTION_MARKER = "WASM-BOUNDARY-EXCEPTION"


def _is_in_core(file_path: str) -> bool:
    """True if file is inside crates/webclaw-core/src/."""
    try:
        parts = Path(file_path).resolve().parts
        # Match: .../crates/webclaw-core/src/...
        for i, p in enumerate(parts):
            if p == "crates" and i + 2 < len(parts):
                if parts[i + 1] == "webclaw-core" and parts[i + 2] == "src":
                    return True
    except Exception:
        pass
    return False


def _scan_content(text: str) -> list[tuple[int, str, str]]:
    """Return list of (line_num, matched_line, reason) for banned imports."""
    findings = []
    for line_num, line in enumerate(text.splitlines(), start=1):
        if _EXCEPTION_MARKER in line:
            continue  # escape hatch documented on same line
        for pattern, reason in _BANNED:
            if re.match(pattern, line):
                findings.append((line_num, line.strip(), reason))
                break
    return findings


@hook_main("wasm-boundary-check")
def main() -> None:
    if os.environ.get("WEBCLAW_WASM_CHECK", "1") == "0":
        return

    payload = read_hook_input()
    tool_name = payload.get("tool_name", "")
    tool_input = payload.get("tool_input", {})
    file_path = tool_input.get("file_path", "")

    if not file_path.endswith(".rs"):
        return
    if not _is_in_core(file_path):
        return

    # Collect content to scan
    # For Edit: scan new_string (post-edit content at that location)
    # For Write: scan content (full new file)
    content = ""
    if tool_name == "Write":
        content = tool_input.get("content", "")
    elif tool_name == "Edit":
        content = tool_input.get("new_string", "")
    else:
        return

    findings = _scan_content(content)
    if not findings:
        return

    # BLOCK
    error_lines = [
        "@@WASM_BOUNDARY_VIOLATION@@",
        f"File: {file_path}",
        "",
        "webclaw-core must stay WASM-safe. Banned import detected:",
    ]
    for line_num, line, reason in findings:
        error_lines.append(f"  L{line_num}: {line}")
        error_lines.append(f"    → {reason}")
    error_lines.append("")
    error_lines.append("See .claude/rules/crate-boundaries.md H1 for full rules.")
    error_lines.append("")
    error_lines.append("Escape hatch: add comment // WASM-BOUNDARY-EXCEPTION: <reason>")
    error_lines.append("             on the same line as the use statement,")
    error_lines.append("             AND add #[cfg(feature = \"wasm-unsafe\")] gate.")
    error_lines.append("Bypass (emergency only): WEBCLAW_WASM_CHECK=0")

    msg = "\n".join(error_lines)
    print(msg, file=sys.stderr)

    log_hook_event(
        "wasm-boundary-check",
        "block",
        f"WASM violation in {file_path}",
        {
            "file_path": file_path,
            "tool": tool_name,
            "findings": [{"line": ln, "reason": r} for ln, _, r in findings],
        },
    )

    sys.exit(2)


if __name__ == "__main__":
    main()
