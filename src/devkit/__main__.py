"""Allow ``python -m devkit`` when ``src`` is on PYTHONPATH."""

from __future__ import annotations

from devkit.cli import main

if __name__ == "__main__":
    raise SystemExit(main())
