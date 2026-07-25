"""Host OS detection for Windows, macOS, and Linux."""

from __future__ import annotations

import os
import platform
from enum import Enum
from pathlib import Path


class HostOS(str, Enum):
    """Supported operating systems."""

    WINDOWS = "windows"
    MACOS = "macos"
    LINUX = "linux"
    OTHER = "other"


def current_os() -> HostOS:
    """Return the current host OS."""
    system = platform.system()
    if system == "Windows":
        return HostOS.WINDOWS
    if system == "Darwin":
        return HostOS.MACOS
    if system == "Linux":
        return HostOS.LINUX
    return HostOS.OTHER


def is_windows() -> bool:
    return current_os() == HostOS.WINDOWS


def is_macos() -> bool:
    return current_os() == HostOS.MACOS


def is_linux() -> bool:
    return current_os() == HostOS.LINUX


def is_unix() -> bool:
    return current_os() in {HostOS.MACOS, HostOS.LINUX, HostOS.OTHER}


def cpu_arch() -> str:
    """Return a normalized CPU arch label: ``x64`` or ``aarch64``."""
    machine = platform.machine().lower()
    if machine in {"arm64", "aarch64"}:
        return "aarch64"
    if machine in {"x86_64", "amd64", "x64"}:
        return "x64"
    if machine in {"i386", "i686", "x86"}:
        return "x86"
    return machine


def adoptium_os() -> str:
    """OS slug for the Adoptium / Temurin API."""
    host = current_os()
    if host == HostOS.WINDOWS:
        return "windows"
    if host == HostOS.MACOS:
        return "mac"
    if host == HostOS.LINUX:
        return "linux"
    raise RuntimeError(f"Unsupported OS for Temurin JDK: {host.value}")


def os_label() -> str:
    """Human-readable OS name for CLI output."""
    labels = {
        HostOS.WINDOWS: "Windows",
        HostOS.MACOS: "macOS",
        HostOS.LINUX: "Linux",
        HostOS.OTHER: platform.system() or "Unknown",
    }
    return labels[current_os()]


def pick_for_os(mapping: dict[str, str], *, default: str | None = None) -> str:
    """Pick a value from a platform map.

    Keys may be ``windows``, ``macos``/``darwin``, ``linux``, or ``*``/``default``.
    """
    os_key = current_os().value
    aliases = {
        HostOS.MACOS: ("macos", "darwin", "osx"),
        HostOS.WINDOWS: ("windows", "win", "win32"),
        HostOS.LINUX: ("linux",),
        HostOS.OTHER: ("other",),
    }
    lowered = {k.lower(): v for k, v in mapping.items()}
    for alias in aliases.get(current_os(), (os_key,)):
        if alias in lowered:
            return lowered[alias]
    if "default" in lowered:
        return lowered["default"]
    if "*" in lowered:
        return lowered["*"]
    if default is not None:
        return default
    known = ", ".join(sorted(mapping))
    raise KeyError(f"No URL/value for OS '{os_key}'. Available keys: {known}")


def shell_profile_candidates() -> list[Path]:
    """Return likely shell profile paths for the current OS (macOS/Linux)."""
    home = Path.home()
    shell = os.environ.get("SHELL", "")
    profiles: list[Path] = []

    if "zsh" in shell or current_os() == HostOS.MACOS:
        profiles.extend([home / ".zshrc", home / ".zprofile"])
    if "bash" in shell or current_os() == HostOS.LINUX:
        profiles.extend([home / ".bashrc", home / ".bash_profile", home / ".profile"])
    # Always include common fallbacks without duplicates
    for extra in (home / ".zshrc", home / ".bashrc", home / ".profile"):
        if extra not in profiles:
            profiles.append(extra)
    return profiles


def primary_shell_profile() -> Path:
    """Best profile file to source ``~/.devkit/env.sh`` from."""
    shell = os.environ.get("SHELL", "")
    home = Path.home()
    if "zsh" in shell or current_os() == HostOS.MACOS:
        return home / ".zshrc"
    if "bash" in shell:
        # Prefer .bashrc when it exists (typical Linux interactive shells)
        bashrc = home / ".bashrc"
        if bashrc.exists() or current_os() == HostOS.LINUX:
            return bashrc
        return home / ".bash_profile"
    return home / ".profile"
