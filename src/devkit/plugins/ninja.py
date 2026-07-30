"""Ninja plugin — latest single-binary ZIP from ninja-build GitHub releases."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status, github_latest_release, pick_release_asset
from devkit.progress import download_progress

MARKER = ".devkit-ninja"


def resolve_ninja_download() -> tuple[str, str]:
    release = github_latest_release("ninja-build", "ninja")
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        if arch == "aarch64":
            return pick_release_asset(release, "ninja-winarm64.zip")
        return pick_release_asset(release, "ninja-win.zip")
    if host == HostOS.LINUX:
        if arch == "aarch64":
            return pick_release_asset(release, "ninja-linux-aarch64.zip")
        return pick_release_asset(release, "ninja-linux.zip")
    if host == HostOS.MACOS:
        return pick_release_asset(release, "ninja-mac.zip")
    raise RuntimeError(f"Ninja is not supported on this OS: {host.value}")


def _ninja_bin(ctx: InstallContext) -> Path:
    return ctx.install_dir / ("ninja.exe" if is_windows() else "ninja")


class NinjaPlugin(Plugin):
    id = "ninja"
    name = "Ninja"
    description = "Download latest Ninja build tool binary and add it to PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _ninja_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but ninja binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_ninja_download()
        print(f"Ninja {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading Ninja")
        # ZIP contains ninja(.exe) at the archive root.
        install_archive_from_url(url, ctx.install_dir, strip_top_level=False, progress=progress)
        progress.done()
        if not _ninja_bin(ctx).is_file():
            raise RuntimeError(f"Ninja extracted but binary not found at {_ninja_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(ctx.install_dir, message=f"Ninja {version} installed at {ctx.install_dir}")

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(paths=[ctx.install_dir], vars={"NINJA_HOME": str(ctx.install_dir.resolve())})
