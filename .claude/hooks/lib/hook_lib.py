"""Hook crash logger + @hook_main decorator for fail-open semantics.

Paraphrased from ITF_APP hook_logger.py (user's own work) — Rust/webclaw adaptation.

Provides:
- log_hook_crash(name, exc, payload): append JSONL to .claude/.logs/hook-log.jsonl
- @hook_main(name): decorator wrapping main() with fail-open semantics (catch all → exit 0)

Kill switch: env WEBCLAW_HOOK_LOGGING=0 → log_hook_crash no-op (still fail-open).

Usage:
    from lib.hook_lib import hook_main

    @hook_main("cargo-fmt-check")
    def main():
        # ... hook body ...
        pass

    if __name__ == "__main__":
        main()
"""

import json
import os
import sys
import traceback
from datetime import datetime
from pathlib import Path
from typing import Any, Callable, Dict, Optional

_MAX_LOG_BYTES = 5 * 1024 * 1024  # 5 MB rotate threshold
_KEEP_ROTATED = 3                  # keep .1, .2, .3


def _log_dir() -> Path:
    """Resolve .claude/.logs directory from project root.

    Project root = parent of .claude/hooks/lib/ (3 levels up from this file).
    """
    here = Path(__file__).resolve()
    return here.parent.parent.parent / ".logs"


def _log_file() -> Path:
    return _log_dir() / "hook-log.jsonl"


def _rotate_if_needed(path: Path) -> None:
    """Rotate log file if over size threshold. Keep last 3 rotations. Silent on failure."""
    try:
        if not path.exists() or path.stat().st_size <= _MAX_LOG_BYTES:
            return
        # Shift .2 → .3, .1 → .2, current → .1
        for i in range(_KEEP_ROTATED, 0, -1):
            src = path.with_suffix(f".jsonl.{i}")
            if i == _KEEP_ROTATED and src.exists():
                src.unlink()
            elif src.exists():
                src.rename(path.with_suffix(f".jsonl.{i + 1}"))
        path.rename(path.with_suffix(".jsonl.1"))
    except Exception:
        pass  # rotate failure is non-fatal


def log_hook_crash(
    name: str,
    exc: BaseException,
    payload: Optional[Dict[str, Any]] = None,
) -> None:
    """Append a JSONL entry recording a hook crash.

    Silent best-effort — if logging itself fails, we swallow to preserve fail-open.
    Respects WEBCLAW_HOOK_LOGGING=0 kill switch (no-op).
    """
    if os.environ.get("WEBCLAW_HOOK_LOGGING", "1") == "0":
        return

    try:
        path = _log_file()
        path.parent.mkdir(parents=True, exist_ok=True)
        _rotate_if_needed(path)

        entry = {
            "ts": datetime.now().isoformat(timespec="seconds"),
            "hook": name,
            "status": "crash",
            "error": f"{type(exc).__name__}: {exc}",
            "traceback": traceback.format_exception(type(exc), exc, exc.__traceback__),
        }
        if payload:
            entry["payload"] = payload

        with path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except Exception:
        pass  # never block on logging failure


def log_hook_event(
    name: str,
    status: str,
    message: str,
    payload: Optional[Dict[str, Any]] = None,
) -> None:
    """Log a non-crash event (info/warn/block). Best-effort, no-op if logging disabled."""
    if os.environ.get("WEBCLAW_HOOK_LOGGING", "1") == "0":
        return

    try:
        path = _log_file()
        path.parent.mkdir(parents=True, exist_ok=True)
        _rotate_if_needed(path)

        entry = {
            "ts": datetime.now().isoformat(timespec="seconds"),
            "hook": name,
            "status": status,
            "message": message,
        }
        if payload:
            entry["payload"] = payload

        with path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(entry, ensure_ascii=False) + "\n")
    except Exception:
        pass


def read_hook_input() -> Dict[str, Any]:
    """Parse JSON payload from stdin (Claude Code hook input format).

    Returns empty dict if stdin is empty or invalid JSON.
    """
    try:
        raw = sys.stdin.read()
        if not raw.strip():
            return {}
        return json.loads(raw)
    except Exception:
        return {}


def hook_main(name: str) -> Callable[[Callable[..., Any]], Callable[..., None]]:
    """Decorator: wrap hook main() with fail-open + crash logging.

    - Catches all exceptions except SystemExit
    - Logs crash to JSONL
    - Exits 0 (fail-open) so Claude Code session never blocks on hook error
    - Preserves SystemExit code if main() called sys.exit(N)

    Args:
        name: Hook identifier for logs (e.g., "cargo-fmt-check", "secret-scanner").

    Example:
        @hook_main("cargo-fmt-check")
        def main():
            # hook logic
            pass
    """
    def decorator(fn: Callable[..., Any]) -> Callable[..., None]:
        def runner(*args: Any, **kwargs: Any) -> None:
            try:
                fn(*args, **kwargs)
            except SystemExit:
                # Preserve intentional sys.exit() codes
                raise
            except Exception as exc:
                log_hook_crash(name, exc)
                sys.exit(0)  # fail-open
        runner.__name__ = fn.__name__
        runner.__doc__ = fn.__doc__
        return runner
    return decorator
