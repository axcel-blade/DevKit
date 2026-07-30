"""Android platform-tools plugin (adb / fastboot) — companion to the android plugin."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status
from devkit.progress import download_progress

_REPO = "https://dl.google.com/android/repository"
MARKER = ".devkit-platform-tools"


def resolve_platform_tools_url() -> tuple[str, str]:
    host = current_os()
    if host == HostOS.WINDOWS:
        slug = "windows"
    elif host == HostOS.MACOS:
        slug = "darwin"
    elif host == HostOS.LINUX:
        slug = "linux"
    else:
        raise RuntimeError(f"platform-tools are not supported on: {host.value}")
    # Google publishes a rolling ``latest`` ZIP per OS.
    name = f"platform-tools-latest-{slug}.zip"
    return f"{_REPO}/{name}", "latest"


def _adb_bin(ctx: InstallContext) -> Path:
    base = ctx.install_dir
    # ZIP root is platform-tools/; may remain if strip_top_level False, or stripped.
    nested = base / "platform-tools" / ("adb.exe" if is_windows() else "adb")
    if nested.is_file():
        return nested
    return base / ("adb.exe" if is_windows() else "adb")


class PlatformToolsPlugin(Plugin):
    id = "platform-tools"
    name = "Android platform-tools"
    description = (
        "Download Android SDK platform-tools (adb, fastboot) and add them to PATH. "
        "Pairs with the android cmdline-tools plugin for Flutter."
    )

    def status(self, ctx: InstallContext):
        return binary_status(
            _adb_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but adb is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_platform_tools_url()
        print(f"Android platform-tools ({version})")
        print(f"URL: {url}")
        progress = download_progress("Downloading platform-tools")
        # Keep platform-tools/ folder as Google ships it.
        install_archive_from_url(url, ctx.install_dir, strip_top_level=False, progress=progress)
        progress.done()
        if not _adb_bin(ctx).is_file():
            raise RuntimeError(f"platform-tools extracted but adb not found at {_adb_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            ctx.install_dir,
            message=f"Android platform-tools installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        adb = _adb_bin(ctx)
        path_dir = adb.parent if adb.is_file() else ctx.install_dir / "platform-tools"
        return EnvSpec(
            paths=[path_dir],
            vars={"ANDROID_PLATFORM_TOOLS": str(path_dir.resolve())},
        )
