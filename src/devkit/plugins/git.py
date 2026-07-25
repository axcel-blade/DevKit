"""Git plugin — MinGit on Windows; system Git wrappers on macOS/Linux.

Windows: resolve the latest MinGit ZIP from git-for-windows GitHub releases,
extract under ``<dev>/git``, and put ``cmd``/``bin`` on PATH.

macOS/Linux: Git has no official portable archive here, so DevKit registers
thin wrappers around an already-installed system ``git``.
"""

from __future__ import annotations

import re
import shutil
import stat
from pathlib import Path

from devkit.download import download_json, install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)
from devkit.progress import download_progress

# Git for Windows publishes portable MinGit ZIP assets (easy to extract).
_GFW_LATEST = "https://api.github.com/repos/git-for-windows/git/releases/latest"
MARKER = ".devkit-git"


def _git_binary(ctx: InstallContext) -> Path | None:
    """Return the installed git executable if present."""
    candidates = [
        ctx.install_dir / "cmd" / ("git.exe" if is_windows() else "git"),
        ctx.install_dir / "bin" / ("git.exe" if is_windows() else "git"),
        ctx.install_dir / "mingw64" / "bin" / "git.exe",
        ctx.install_dir / "mingw32" / "bin" / "git.exe",
    ]
    for path in candidates:
        if path.is_file():
            return path
    return None


def _path_dirs(ctx: InstallContext) -> list[Path]:
    """Directories to prepend so ``git`` resolves (prefer ``cmd`` on Windows)."""
    dirs: list[Path] = []
    for rel in ("cmd", "bin"):
        d = ctx.install_dir / rel
        if d.is_dir():
            dirs.append(d)
    return dirs


def resolve_mingit_url() -> tuple[str, str]:
    """Pick the latest MinGit ZIP URL for this Windows CPU arch.

    Returns ``(url, version_label)``.
    """
    arch = cpu_arch()
    if arch == "aarch64":
        want = re.compile(r"^MinGit-.*-arm64\.zip$", re.IGNORECASE)
    elif arch == "x64":
        # Prefer standard 64-bit MinGit (not busybox, not 32-bit).
        want = re.compile(r"^MinGit-.*-64-bit\.zip$", re.IGNORECASE)
    else:
        want = re.compile(r"^MinGit-.*-32-bit\.zip$", re.IGNORECASE)

    meta = download_json(_GFW_LATEST)
    tag = str(meta.get("tag_name", "unknown"))
    assets = meta.get("assets") or []
    matches: list[dict] = []
    for asset in assets:
        if not isinstance(asset, dict):
            continue
        name = str(asset.get("name", ""))
        if "busybox" in name.lower():
            continue
        if want.match(name) and asset.get("browser_download_url"):
            matches.append(asset)
    if not matches:
        raise RuntimeError(
            f"No MinGit ZIP found for arch={arch} in git-for-windows latest release"
        )
    # Prefer the first match (API order is stable enough for a given release).
    chosen = matches[0]
    return str(chosen["browser_download_url"]), tag


def _write_unix_wrappers(install_dir: Path, system_git: Path) -> None:
    """Create thin wrappers under install_dir/bin that call system git."""
    bin_dir = install_dir / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    for name in ("git", "git-receive-pack", "git-upload-pack"):
        system = shutil.which(name)
        target = bin_dir / name
        if not system and name != "git":
            continue
        exe = system or str(system_git)
        target.write_text(
            "#!/usr/bin/env bash\n"
            f'exec "{exe}" "$@"\n',
            encoding="utf-8",
            newline="\n",
        )
        mode = target.stat().st_mode
        target.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    (install_dir / MARKER).write_text(f"system-wrapper\n{system_git}\n", encoding="utf-8")


class GitPlugin(Plugin):
    """Install Git into the machine ``dev`` folder."""

    id = "git"
    name = "Git"
    description = (
        "Install MinGit (Git for Windows portable ZIP). "
        "On macOS/Linux, registers system Git if already installed."
    )

    def status(self, ctx: InstallContext) -> PluginStatus:
        binary = _git_binary(ctx)
        if binary and binary.is_file():
            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail=str(binary),
            )
        if (ctx.install_dir / MARKER).is_file():
            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail="system git wrappers",
            )
        if ctx.install_dir.exists() and any(ctx.install_dir.iterdir()):
            return PluginStatus(
                state=InstallState.PARTIAL,
                install_dir=ctx.install_dir,
                detail="Install dir exists but git binary is missing",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        host = current_os()
        if host == HostOS.WINDOWS:
            return self._install_windows(ctx)
        if host in {HostOS.MACOS, HostOS.LINUX}:
            return self._install_unix(ctx)
        raise RuntimeError(f"Git is not supported on this OS: {host.value}")

    def _install_windows(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_mingit_url()
        print(f"Git for Windows MinGit {version}")
        print(f"URL: {url}")
        # MinGit ZIP lays out cmd/, mingw64/, etc. at the archive root.
        progress = download_progress("Downloading Git")
        install_archive_from_url(
            url,
            ctx.install_dir,
            strip_top_level=False,
            progress=progress,
        )
        progress.done()
        binary = _git_binary(ctx)
        if not binary:
            raise RuntimeError(
                f"MinGit extracted but git.exe was not found under {ctx.install_dir}"
            )
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Git {version} (MinGit) installed at {ctx.install_dir}",
        )

    def _install_unix(self, ctx: InstallContext) -> InstallResult:
        system_git = shutil.which("git")
        if not system_git:
            raise RuntimeError(
                "Git does not ship a portable macOS/Linux archive for DevKit.\n"
                "Install Git with your platform tools, then re-run this command:\n"
                "  macOS:          xcode-select --install\n"
                "                  (or: brew install git)\n"
                "  Debian/Ubuntu:  sudo apt install git\n"
                "  Fedora:         sudo dnf install git\n"
                "  Arch:           sudo pacman -S git\n"
                "Then:  python main.py install git"
            )
        ctx.install_dir.mkdir(parents=True, exist_ok=True)
        _write_unix_wrappers(ctx.install_dir, Path(system_git))
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Registered system Git at {system_git}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        paths = _path_dirs(ctx)
        return EnvSpec(
            paths=paths,
            vars={"GIT_HOME": str(ctx.install_dir.resolve())},
        )
