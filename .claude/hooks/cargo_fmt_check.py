#!/usr/bin/env python3
"""PostToolUse hook — warn if edited *.rs file fails cargo fmt --check.

Matcher: Edit | Write on *.rs files
Behavior: fail-open (warn via stderr, exit 0). Does NOT block edit.

Runs `cargo fmt --check -p <crate>` on the touched crate only (fast).
If fail → emit warning to stderr suggesting `cargo fmt -p <crate>`.

Kill switch: WEBCLAW_HOOK_LOGGING=0 disables logging (hook still runs).
"""

import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from lib.hook_lib import hook_main, read_hook_input, log_hook_event


def _crate_of(file_path: str) -> str | None:
    """Resolve crate name from file path.

    Expected: D:\\webclaw\\crates\\webclaw-<crate>\\src\\...
    Returns: 'webclaw-<crate>' or None if not in a crate.
    """
    try:
        path = Path(file_path).resolve()
        parts = path.parts
        # find 'crates' segment, next segment is crate name
        for i, part in enumerate(parts):
            if part == "crates" and i + 1 < len(parts):
                return parts[i + 1]
    except Exception:
        pass
    return None


@hook_main("cargo-fmt-check")
def main() -> None:
    payload = read_hook_input()
    tool_input = payload.get("tool_input", {})
    file_path = tool_input.get("file_path", "")

    # Only care about .rs files
    if not file_path.endswith(".rs"):
        return

    crate = _crate_of(file_path)
    if not crate:
        # Not in a crate subdir — skip silently
        return

    # Run cargo fmt --check on this crate
    project_root = Path(__file__).resolve().parent.parent.parent
    try:
        result = subprocess.run(
            ["cargo", "fmt", "--check", "-p", crate],
            cwd=project_root,
            capture_output=True,
            text=True,
            timeout=20,
        )
    except subprocess.TimeoutExpired:
        log_hook_event("cargo-fmt-check", "warn", f"timeout on {crate}")
        return
    except FileNotFoundError:
        # cargo not installed / not in PATH
        log_hook_event("cargo-fmt-check", "warn", "cargo not found in PATH")
        return

    if result.returncode != 0:
        # Format diff present — warn user
        msg = (
            f"⚠ cargo fmt diff detected in {crate} (file: {file_path}).\n"
            f"  Fix: cargo fmt -p {crate}\n"
        )
        print(msg, file=sys.stderr)
        log_hook_event(
            "cargo-fmt-check",
            "warn",
            f"fmt diff in {crate}",
            {"file_path": file_path, "crate": crate},
        )
        # fail-open: do NOT exit non-zero


if __name__ == "__main__":
    main()
