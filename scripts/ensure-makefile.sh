#!/bin/sh
# Check that this repo can run the Makefile. Install GNU make, cargo, a C
# compiler, or bash when they are missing. Invoked by the Makefile before
# other targets. Run this file directly when `make` itself is not installed.
set -eu

ROOT="$(CDPATH= cd "$(dirname "$0")/.." && pwd)"
FROM_MAKE=0
NEED_BASH=0
for arg in "$@"; do
    if [ "$arg" = "--from-make" ]; then
        FROM_MAKE=1
    fi
    if [ "$arg" = "sh" ]; then
        NEED_BASH=1
    fi
done

install_packages() {
    # $1 is a short package description. Remaining words are the command.
    desc="$1"
    shift
    echo "$desc"
    if ! sh -c "$*"; then
        echo "Error: could not install automatically."
        echo "Run this yourself, then start make again:"
        echo "  $*"
        exit 1
    fi
}

sudo_prefix() {
    if [ "$(id -u)" -ne 0 ] && command -v sudo >/dev/null 2>&1; then
        echo sudo
    fi
}

if ! command -v make >/dev/null 2>&1 && ! command -v gmake >/dev/null 2>&1; then
    SUDO="$(sudo_prefix)"
    if command -v apt-get >/dev/null 2>&1; then
        install_packages "GNU make is not installed. Installing it..." "$SUDO apt-get update && $SUDO apt-get install -y make"
    elif command -v dnf >/dev/null 2>&1; then
        install_packages "GNU make is not installed. Installing it..." "$SUDO dnf install -y make"
    elif command -v yum >/dev/null 2>&1; then
        install_packages "GNU make is not installed. Installing it..." "$SUDO yum install -y make"
    elif command -v pacman >/dev/null 2>&1; then
        install_packages "GNU make is not installed. Installing it..." "$SUDO pacman -S --needed --noconfirm make"
    elif command -v zypper >/dev/null 2>&1; then
        install_packages "GNU make is not installed. Installing it..." "$SUDO zypper install -y make"
    elif command -v apk >/dev/null 2>&1; then
        install_packages "GNU make is not installed. Installing it..." "$SUDO apk add make"
    elif [ "$(uname -s)" = "Darwin" ]; then
        install_packages "GNU make is not installed. Installing Apple command line tools..." "xcode-select --install"
    else
        echo "GNU make is not installed. Install make, then run this script again."
        exit 1
    fi
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo is not installed. Installing Rust (rustup, stable)..."
    if ! command -v curl >/dev/null 2>&1; then
        echo "curl is required to install Rust. Install curl and run make again."
        exit 1
    fi
    curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs | sh -s -- -y
    echo "Rust was installed. Open a new terminal so cargo is on PATH, then run make again."
    exit 1
fi

if ! command -v cc >/dev/null 2>&1; then
    echo "C compiler (cc) not found. Rust needs it to build DevKit."
    SUDO="$(sudo_prefix)"
    if command -v apt-get >/dev/null 2>&1; then
        install_packages "Installing the C toolchain..." "$SUDO apt-get update && $SUDO apt-get install -y build-essential"
    elif command -v dnf >/dev/null 2>&1; then
        install_packages "Installing the C toolchain..." "$SUDO dnf install -y gcc gcc-c++ make"
    elif command -v yum >/dev/null 2>&1; then
        install_packages "Installing the C toolchain..." "$SUDO yum install -y gcc gcc-c++ make"
    elif command -v pacman >/dev/null 2>&1; then
        install_packages "Installing the C toolchain..." "$SUDO pacman -S --needed --noconfirm base-devel"
    elif command -v zypper >/dev/null 2>&1; then
        install_packages "Installing the C toolchain..." "$SUDO zypper install -y gcc gcc-c++ make"
    elif command -v apk >/dev/null 2>&1; then
        install_packages "Installing the C toolchain..." "$SUDO apk add build-base"
    elif [ "$(uname -s)" = "Darwin" ]; then
        install_packages "Installing Apple command line tools..." "xcode-select --install"
    else
        echo "Install a C compiler (gcc or clang) providing cc, then run make again."
        exit 1
    fi
fi

if [ "$NEED_BASH" -eq 1 ] && ! command -v bash >/dev/null 2>&1; then
    SUDO="$(sudo_prefix)"
    if command -v apt-get >/dev/null 2>&1; then
        install_packages "bash is not installed. Installing it..." "$SUDO apt-get update && $SUDO apt-get install -y bash"
    elif command -v dnf >/dev/null 2>&1; then
        install_packages "bash is not installed. Installing it..." "$SUDO dnf install -y bash"
    elif command -v yum >/dev/null 2>&1; then
        install_packages "bash is not installed. Installing it..." "$SUDO yum install -y bash"
    elif command -v pacman >/dev/null 2>&1; then
        install_packages "bash is not installed. Installing it..." "$SUDO pacman -S --needed --noconfirm bash"
    elif command -v zypper >/dev/null 2>&1; then
        install_packages "bash is not installed. Installing it..." "$SUDO zypper install -y bash"
    elif command -v apk >/dev/null 2>&1; then
        install_packages "bash is not installed. Installing it..." "$SUDO apk add bash"
    else
        echo "bash is not installed. Install bash, then run make sh again."
        exit 1
    fi
fi

# Direct launch (not from make): hand off to make once it is available.
if [ "$FROM_MAKE" -eq 0 ]; then
    if command -v make >/dev/null 2>&1; then
        exec make -C "$ROOT" "$@"
    elif command -v gmake >/dev/null 2>&1; then
        exec gmake -C "$ROOT" "$@"
    fi
fi
