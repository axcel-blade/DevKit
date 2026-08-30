#!/usr/bin/env bash
# DevKit application launcher (macOS / Linux).
#
# DevKit itself is built with Rust, so before anything else this makes sure
# a Rust toolchain is available: if `cargo` isn't found, it checks for
# internet access, then downloads and runs rustup-init the same way the
# built-in `rust` plugin does (same host-triple URL, `-y --default-toolchain
# stable --profile default`) — except into the *standard* ~/.cargo location,
# since this Rust install is what builds DevKit, not an SDK DevKit is
# managing for someone else.
#
# Once cargo is available, it builds the release binary on first run (or
# after source changes) and forwards all arguments to it. Running with no
# arguments launches the interactive plugin menu (see `devkit menu`).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"
BIN="$ROOT/target/release/devkit"

CARGO_EXE=""
if command -v cargo >/dev/null 2>&1; then
    CARGO_EXE="cargo"
elif [ -x "$HOME/.cargo/bin/cargo" ]; then
    CARGO_EXE="$HOME/.cargo/bin/cargo"
fi

if [ -z "$CARGO_EXE" ]; then
    echo "DevKit needs a Rust toolchain (cargo) to build itself — none found."
    echo "Checking internet connection..."

    # A raw TCP connect via bash's /dev/tcp — no extra tools needed. Wrapped
    # in an `if !` so `set -e` doesn't abort the script on a failed connect.
    if ! (exec 3<>"/dev/tcp/static.rust-lang.org/443") 2>/dev/null; then
        echo "No internet connection detected."
        echo "Install Rust manually from https://rustup.rs and re-run this script."
        exit 1
    fi
    exec 3<&- 3>&- 2>/dev/null || true

    case "$(uname -s)" in
        Darwin) os="apple-darwin" ;;
        Linux) os="unknown-linux-gnu" ;;
        *)
            echo "Unsupported OS for automatic Rust install: $(uname -s)"
            echo "Install Rust manually from https://rustup.rs and re-run this script."
            exit 1
            ;;
    esac
    case "$(uname -m)" in
        arm64 | aarch64) arch="aarch64" ;;
        *) arch="x86_64" ;;
    esac
    triple="${arch}-${os}"

    echo "Installing Rust (rustup, stable) for $triple ..."
    tmp_init="$(mktemp)"
    curl --proto '=https' --tlsv1.2 -sSf \
        "https://static.rust-lang.org/rustup/dist/${triple}/rustup-init" \
        -o "$tmp_init"
    chmod +x "$tmp_init"
    "$tmp_init" -y --default-toolchain stable --profile default
    rm -f "$tmp_init"

    CARGO_EXE="$HOME/.cargo/bin/cargo"
    if [ ! -x "$CARGO_EXE" ]; then
        echo "Rust install finished but cargo was not found at $CARGO_EXE."
        exit 1
    fi
    echo "Rust installed. Open a new terminal for PATH to include cargo automatically."
fi

if [ ! -x "$BIN" ]; then
    "$CARGO_EXE" build --release --manifest-path "$ROOT/Cargo.toml"
fi
exec "$BIN" "$@"
