#!/usr/bin/env bash
# DevKit application launcher (macOS / Linux).
#
# 1. Require an internet connection (needed to fetch rustup / crates).
# 2. Use Rust from the machine `dev` folder (`/opt/dev/rust` or `~/dev/rust`,
#    or `$DEVKIT_HOME/rust`). If cargo is missing there, install rustup
#    stable into that folder (same rustup-init flags as the rust plugin:
#    -y --no-modify-path).
# 3. Build the release binary if needed, then run it.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
BIN="$ROOT/target/release/devkit"

# Step 1: fail fast if we cannot reach rustup's CDN (HTTPS/443).
echo "Checking internet connection..."
# A raw TCP connect via bash's /dev/tcp — no extra tools needed. Wrapped
# in an `if !` so `set -e` doesn't abort the script on a failed connect.
if ! (exec 3<>"/dev/tcp/static.rust-lang.org/443") 2>/dev/null; then
    echo "Error: no internet connection. Connect to the internet and try again."
    exit 1
fi
exec 3<&- 3>&- 2>/dev/null || true

# Step 2: same machine `dev` root as paths::home() (DEVKIT_HOME, /opt/dev, ~/dev).
if [ -n "${DEVKIT_HOME:-}" ]; then
    DEVROOT="$DEVKIT_HOME"
elif mkdir -p /opt/dev 2>/dev/null && [ -w /opt/dev ]; then
    DEVROOT="/opt/dev"
else
    DEVROOT="${HOME}/dev"
fi

export CARGO_HOME="${DEVROOT}/rust/cargo"
export RUSTUP_HOME="${DEVROOT}/rust/rustup"
CARGO_EXE="${CARGO_HOME}/bin/cargo"

if [ ! -x "$CARGO_EXE" ]; then
    echo "Rust is not installed in ${DEVROOT}/rust."
    echo "Installing Rust (rustup, stable) into the machine dev folder..."

    case "$(uname -s)" in
        Darwin) os="apple-darwin" ;;
        Linux) os="unknown-linux-gnu" ;;
        *)
            echo "Unsupported OS for automatic Rust install: $(uname -s)"
            echo "Connect to the internet and install Rust from https://rustup.rs, or use Windows / macOS / Linux."
            exit 1
            ;;
    esac
    case "$(uname -m)" in
        arm64 | aarch64) arch="aarch64" ;;
        *) arch="x86_64" ;;
    esac
    triple="${arch}-${os}"

    mkdir -p "${DEVROOT}/rust"
    tmp_init="$(mktemp)"
    curl --proto '=https' --tlsv1.2 -sSf \
        "https://static.rust-lang.org/rustup/dist/${triple}/rustup-init" \
        -o "$tmp_init"
    chmod +x "$tmp_init"
    "$tmp_init" -y --no-modify-path --default-toolchain stable --profile default
    rm -f "$tmp_init"

    if [ ! -x "$CARGO_EXE" ]; then
        echo "Rust install finished but cargo was not found at $CARGO_EXE."
        exit 1
    fi
    echo "Rust installed at ${DEVROOT}/rust."
fi

if [ ! -x "$BIN" ]; then
    "$CARGO_EXE" build --release --manifest-path "$ROOT/Cargo.toml"
fi
exec "$BIN" "$@"
