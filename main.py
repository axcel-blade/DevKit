#!/usr/bin/env python3
"""DevKit application entrypoint.

Run from the repo root (no library install needed):

    python main.py doctor
    python main.py list
    python main.py install hello
"""

from __future__ import annotations

import sys
from pathlib import Path

_ROOT = Path(__file__).resolve().parent
_SRC = _ROOT / "src"
if _SRC.is_dir() and str(_SRC) not in sys.path:
    sys.path.insert(0, str(_SRC))


def _run() -> int:
    from devkit.cli import main

    return int(main())


if __name__ == "__main__":
    raise SystemExit(_run())
