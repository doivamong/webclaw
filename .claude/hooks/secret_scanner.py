#!/usr/bin/env python3
"""PreToolUse hook — block Write/Edit containing API key literals.

Matcher: Write | Edit
Behavior: BLOCK write (exit 2 → Claude Code aborts tool call) if content
matches known API key regex. Logs violation.

Scans both:
- Write tool: `content` parameter
- Edit tool: `new_string` parameter

Kill switch (emergency):
  WEBCLAW_SECRET_SCAN=0  → disable block (still logs)

Patterns detected:
- OpenAI (sk-...)
- Anthropic (sk-ant-...)
- Serper / SerpAPI (64-hex)
- Generic Bearer tokens
- AWS access key (AKIA...)
- GitHub PAT (ghp_..., ghs_..., gho_...)
"""

import json
import os
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from lib.hook_lib import hook_main, read_hook_input, log_hook_event


# Regex patterns — order matters (more specific first)
_PATTERNS = [
    (r"sk-ant-api\d{2}-[A-Za-z0-9_\-]{80,}", "Anthropic API key"),
    (r"sk-[A-Za-z0-9]{32,}", "OpenAI API key"),
    (r"ghp_[A-Za-z0-9]{36}", "GitHub personal access token"),
    (r"gh[ous]_[A-Za-z0-9]{36}", "GitHub OAuth/app token"),
    (r"AKIA[A-Z0-9]{16}", "AWS access key ID"),
    (r"Bearer\s+[A-Za-z0-9+/=_\-]{32,}", "Bearer token"),
    (r"(?<![a-f0-9])[a-f0-9]{64}(?![a-f0-9])", "Hex secret (possible Serper/SHA256)"),
]

# Allowlist — strings that LOOK LIKE secrets but aren't
_ALLOWLIST = {
    # Example keys used in docs/tests (user should use env var in real code)
    "sk-ant-api03-abc123...",
    "sk-ant-api03-xxxx",
    # Git commit hashes (40 hex, but pattern above is 64 hex so won't match)
}

# File patterns where secret-like strings are OK
_ALLOWED_FILES = {
    ".env.example",
    "README.md",          # docs showing secret format
    "SECURITY.md",
    "SKILL.md",           # skill doc may reference secret format
}


def _is_allowed_file(file_path: str) -> bool:
    name = Path(file_path).name
    if name in _ALLOWED_FILES:
        return True
    # Docs folder
    if "/docs/" in file_path.replace("\\", "/"):
        return True
    return False


def _scan(text: str, file_path: str) -> list[tuple[str, str]]:
    """Return list of (matched_substring, reason)."""
    findings = []
    for pattern, reason in _PATTERNS:
        for m in re.finditer(pattern, text):
            matched = m.group(0)
            if matched in _ALLOWLIST:
                continue
            findings.append((matched[:20] + "...", reason))
    return findings


@hook_main("secret-scanner")
def main() -> None:
    # Emergency kill switch
    if os.environ.get("WEBCLAW_SECRET_SCAN", "1") == "0":
        return

    payload = read_hook_input()
    tool_name = payload.get("tool_name", "")
    tool_input = payload.get("tool_input", {})
    file_path = tool_input.get("file_path", "")

    # Allowlist: docs file OK to contain secret-format strings
    if _is_allowed_file(file_path):
        return

    # Collect content to scan based on tool
    content = ""
    if tool_name == "Write":
        content = tool_input.get("content", "")
    elif tool_name == "Edit":
        content = tool_input.get("new_string", "")
    else:
        return  # other tools not scanned

    findings = _scan(content, file_path)
    if not findings:
        return

    # BLOCK: print structured error then exit 2 (Claude Code interprets as tool rejection)
    error_lines = [
        "@@SECRET_DETECTED@@",
        f"File: {file_path}",
        "Detected:",
    ]
    for matched, reason in findings:
        error_lines.append(f"  - {reason}: {matched}")
    error_lines.append("")
    error_lines.append("Fix: use env var / config file, don't hardcode secrets.")
    error_lines.append("Bypass (emergency only): set WEBCLAW_SECRET_SCAN=0")

    msg = "\n".join(error_lines)
    print(msg, file=sys.stderr)

    log_hook_event(
        "secret-scanner",
        "block",
        f"secret detected in {file_path}",
        {"file_path": file_path, "tool": tool_name, "findings": [r for _, r in findings]},
    )

    sys.exit(2)  # block the tool call


if __name__ == "__main__":
    main()
