#!/usr/bin/env bash
# DevKit application launcher (macOS / Linux) — builds the release binary on
# first run (or after source changes), then forwards to it.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
BIN="$ROOT/target/release/devkit"
if [ ! -x "$BIN" ]; then
    cargo build --release --manifest-path "$ROOT/Cargo.toml"
fi
exec "$BIN" "$@"
