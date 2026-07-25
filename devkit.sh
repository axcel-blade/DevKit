#!/usr/bin/env bash
# DevKit application launcher (macOS / Linux)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
exec python3 "$ROOT/main.py" "$@"
