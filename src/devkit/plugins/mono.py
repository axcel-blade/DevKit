"""Mono runtime / MDK plugin."""

from __future__ import annotations

import shutil
import stat
from pathlib import Path

from devkit.download import download_file
from devkit.installers import extract_msi_admin, extract_pkg
from devkit.platform import HostOS, current_os, is_windows
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)

MONO_VERSION = "6.12.0.206"
MONO_WINDOWS_URL = (
    f"https://download.mono-project.com/archive/6.12.0/windows-installer/"
    f"mono-{MONO_VERSION}-x64-0.msi"
)
MONO_MACOS_URL = (
    f"https://download.mono-project.com/archive/6.12.0/macos-10-universal/"
    f"MonoFramework-MDK-{MONO_VERSION}.macos10.xamarin.universal.pkg"
)
MARKER = ".devkit-mono"


def _find_mono_bin_dir(root: Path) -> Path | None:
    """Locate a directory that contains the ``mono`` executable."""
    names = ("mono.exe", "mono")
    for name in names:
        matches = list(root.rglob(name))
        files = [m for m in matches if m.is_file()]
        if files:
            # Prefer .../bin/mono over deeper copies
            files.sort(key=lambda p: (len(p.parts), str(p)))
            return files[0].parent
    # Known Mono.framework layout on macOS
    commands = root / "Library" / "Frameworks" / "Mono.framework" / "Commands"
    if (commands / "mono").is_file():
        return commands
    return None


def _mono_binary(ctx: InstallContext) -> Path | None:
    bin_dir = _find_mono_bin_dir(ctx.install_dir)
    if not bin_dir:
        return None
    if is_windows():
        exe = bin_dir / "mono.exe"
        return exe if exe.is_file() else None
    mono = bin_dir / "mono"
    return mono if mono.is_file() else None


def _write_linux_wrappers(install_dir: Path, system_mono: Path) -> None:
    """Create thin wrappers in install_dir that call the system mono."""
    bin_dir = install_dir / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    for name in ("mono", "mcs", "csharp"):
        system = shutil.which(name)
        target = bin_dir / name
        if system:
            target.write_text(
                "#!/usr/bin/env bash\n"
                f'exec "{system}" "$@"\n',
                encoding="utf-8",
                newline="\n",
            )
        elif name == "mono":
            target.write_text(
                "#!/usr/bin/env bash\n"
                f'exec "{system_mono}" "$@"\n',
                encoding="utf-8",
                newline="\n",
            )
        else:
            continue
        mode = target.stat().st_mode
        target.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    (install_dir / MARKER).write_text(
        f"system-wrapper\n{system_mono}\n",
        encoding="utf-8",
    )


class MonoPlugin(Plugin):
    """Install Mono into the machine ``dev`` folder (Windows/macOS) or wrap system Mono (Linux)."""

    id = "mono"
    name = "Mono"
    description = (
        f"Install Mono {MONO_VERSION} (Windows MSI / macOS PKG). "
        "On Linux, registers system Mono if already installed via apt/dnf."
    )

    def status(self, ctx: InstallContext) -> PluginStatus:
        binary = _mono_binary(ctx)
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
                detail="system mono wrappers",
            )
        if ctx.install_dir.exists() and any(ctx.install_dir.iterdir()):
            return PluginStatus(
                state=InstallState.PARTIAL,
                install_dir=ctx.install_dir,
                detail="Install dir exists but mono binary is missing",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        host = current_os()
        if host == HostOS.WINDOWS:
            return self._install_windows(ctx)
        if host == HostOS.MACOS:
            return self._install_macos(ctx)
        if host == HostOS.LINUX:
            return self._install_linux(ctx)
        raise RuntimeError(f"Mono is not supported on this OS: {host.value}")

    def _install_windows(self, ctx: InstallContext) -> InstallResult:
        print(f"Downloading Mono {MONO_VERSION} (Windows x64 MSI) ...")
        msi = download_file(MONO_WINDOWS_URL, filename=f"mono-{MONO_VERSION}-x64.msi")
        print(f"Extracting MSI into {ctx.install_dir} ...")
        extract_msi_admin(msi, ctx.install_dir)
        binary = _mono_binary(ctx)
        if not binary:
            raise RuntimeError(
                "Mono MSI extracted but mono.exe was not found under "
                f"{ctx.install_dir}"
            )
        (ctx.install_dir / MARKER).write_text(MONO_VERSION + "\n", encoding="utf-8")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Mono {MONO_VERSION} installed at {ctx.install_dir}",
        )

    def _install_macos(self, ctx: InstallContext) -> InstallResult:
        print(f"Downloading Mono {MONO_VERSION} (macOS PKG) ...")
        pkg = download_file(
            MONO_MACOS_URL,
            filename=f"MonoFramework-MDK-{MONO_VERSION}.pkg",
        )
        print(f"Extracting PKG into {ctx.install_dir} ...")
        extract_pkg(pkg, ctx.install_dir)
        binary = _mono_binary(ctx)
        if not binary:
            raise RuntimeError(
                "Mono PKG extracted but mono was not found under "
                f"{ctx.install_dir}"
            )
        (ctx.install_dir / MARKER).write_text(MONO_VERSION + "\n", encoding="utf-8")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Mono {MONO_VERSION} installed at {ctx.install_dir}",
        )

    def _install_linux(self, ctx: InstallContext) -> InstallResult:
        system_mono = shutil.which("mono")
        if not system_mono:
            raise RuntimeError(
                "Mono does not ship a portable Linux archive for DevKit.\n"
                "Install it with your package manager, then re-run this command:\n"
                "  Debian/Ubuntu:  sudo apt install mono-complete\n"
                "  Fedora:         sudo dnf install mono-complete\n"
                "  Arch:           sudo pacman -S mono\n"
                "Then:  python main.py install mono"
            )
        ctx.install_dir.mkdir(parents=True, exist_ok=True)
        _write_linux_wrappers(ctx.install_dir, Path(system_mono))
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Registered system Mono at {system_mono}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        bin_dir = _find_mono_bin_dir(ctx.install_dir)
        paths = [bin_dir] if bin_dir else []
        # Fallback for Linux wrappers
        wrapper_bin = ctx.install_dir / "bin"
        if wrapper_bin.is_dir() and wrapper_bin not in paths:
            paths.append(wrapper_bin)
        return EnvSpec(
            paths=paths,
            vars={"MONO_HOME": str(ctx.install_dir.resolve())},
        )
