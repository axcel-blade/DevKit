#!/usr/bin/env bash
# DevKit application launcher (macOS / Linux).
#
# 1. Use Rust from the machine `dev` folder (`/opt/dev/rust` or `~/dev/rust`,
#    or `$DEVKIT_HOME/rust`). If cargo is missing there, require an internet
#    connection and install rustup stable into that folder (same rustup-init
#    flags as the rust plugin: -y --no-modify-path).
# 2. Rebuild the release binary (a no-op when it is already up to date), so
#    a stale binary from an older checkout never hides new features.
# 3. Run it. With no arguments the interactive plugin menu is shown.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
BIN="$ROOT/target/release/devkit"

# Step 1: same machine `dev` root as paths::home() (DEVKIT_HOME, /opt/dev, ~/dev).
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

    # Installing Rust needs rustup's CDN, so fail fast when it is
    # unreachable. An already-installed toolchain skips this check so the
    # menu still opens offline. A raw TCP connect via bash's /dev/tcp — no
    # extra tools needed; wrapped in `if !` so `set -e` doesn't abort.
    echo "Checking internet connection..."
    if ! (exec 3<>"/dev/tcp/static.rust-lang.org/443") 2>/dev/null; then
        echo "Error: no internet connection. Connect to the internet and try again."
        exit 1
    fi

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
    # rustup picks its mode from its own file name, so the installer must be
    # named exactly `rustup-init` (mktemp's random name fails with "unknown
    # proxy name"). Put it in a private temp directory instead.
    tmp_dir="$(mktemp -d)"
    tmp_init="${tmp_dir}/rustup-init"
    curl --proto '=https' --tlsv1.2 -sSf \
        "https://static.rust-lang.org/rustup/dist/${triple}/rustup-init" \
        -o "$tmp_init"
    chmod +x "$tmp_init"
    "$tmp_init" -y --no-modify-path --default-toolchain stable --profile default
    rm -rf "$tmp_dir"

    if [ ! -x "$CARGO_EXE" ]; then
        echo "Rust install finished but cargo was not found at $CARGO_EXE."
        exit 1
    fi
    echo "Rust installed at ${DEVROOT}/rust."
fi

# Step 2: always run cargo build. Cargo only recompiles when sources
# changed, so this is fast when up to date, and it guarantees the binary
# matches this checkout (an old binary may predate the plugin menu).
if ! "$CARGO_EXE" build --release --quiet --manifest-path "$ROOT/Cargo.toml"; then
    if [ ! -x "$BIN" ]; then
        echo "Build failed."
        exit 1
    fi
    echo "Warning: build failed, running the previously built binary."
fi

# Step 3: no arguments means the user just launched DevKit, so open the
# interactive menu explicitly.
if [ "$#" -eq 0 ]; then
    exec "$BIN" menu
fi
exec "$BIN" "$@"
