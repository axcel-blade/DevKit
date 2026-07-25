"""Android SDK command-line tools plugin (helper for Flutter / Android builds).

Downloads Google's cmdline-tools ZIP into the machine ``dev`` folder, lays out
``cmdline-tools/latest/`` as required by ``sdkmanager``, and sets
``ANDROID_HOME`` / ``ANDROID_SDK_ROOT``. Does not accept licenses or install
platform-tools — run ``sdkmanager`` yourself after install.
"""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, current_os, is_windows
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)
from devkit.progress import download_progress

# Pin a known cmdline-tools build; bump when cutting a release that tracks newer tools.
CMDLINE_TOOLS_BUILD = "14742923"
_REPO = "https://dl.google.com/android/repository"
MARKER = ".devkit-android"


def resolve_cmdline_tools_url() -> tuple[str, str]:
    """Return ``(download_url, build_id)`` for this OS."""
    host = current_os()
    if host == HostOS.WINDOWS:
        slug = "win"
    elif host == HostOS.MACOS:
        slug = "mac"
    elif host == HostOS.LINUX:
        slug = "linux"
    else:
        raise RuntimeError(f"Android cmdline-tools are not supported on: {host.value}")
    name = f"commandlinetools-{slug}-{CMDLINE_TOOLS_BUILD}_latest.zip"
    return f"{_REPO}/{name}", CMDLINE_TOOLS_BUILD


def _sdkmanager(ctx: InstallContext) -> Path:
    base = ctx.install_dir / "cmdline-tools" / "latest" / "bin"
    if is_windows():
        return base / "sdkmanager.bat"
    return base / "sdkmanager"


def _layout_cmdline_tools(ctx: InstallContext) -> None:
    """Move extracted ``cmdline-tools/`` contents under ``cmdline-tools/latest/``.

    Google's ZIP unpacks as ``cmdline-tools/{bin,lib,...}``. ``sdkmanager``
    expects ``cmdline-tools/latest/bin/sdkmanager`` (or a versioned sibling).
    """
    root = ctx.install_dir
    extracted = root / "cmdline-tools"
    latest = extracted / "latest"
    if _sdkmanager(ctx).is_file():
        return
    if not extracted.is_dir():
        raise RuntimeError(f"Expected cmdline-tools folder under {root}")
    # Zip unpacks as install_dir/cmdline-tools/{bin,lib,...}; nest under latest/.
    if latest.exists():
        shutil.rmtree(latest)
    latest.mkdir(parents=True)
    for child in list(extracted.iterdir()):
        if child.name == "latest":
            continue
        shutil.move(str(child), str(latest / child.name))


class AndroidPlugin(Plugin):
    """Install Android SDK command-line tools for Flutter and Android CLI workflows."""

    id = "android"
    name = "Android SDK"
    description = (
        "Download Android SDK cmdline-tools and set ANDROID_HOME / ANDROID_SDK_ROOT."
    )

    def status(self, ctx: InstallContext) -> PluginStatus:
        binary = _sdkmanager(ctx)
        if binary.is_file():
            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail=str(binary),
            )
        if ctx.install_dir.exists() and any(ctx.install_dir.iterdir()):
            return PluginStatus(
                state=InstallState.PARTIAL,
                install_dir=ctx.install_dir,
                detail="Install dir exists but sdkmanager is missing",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        url, build = resolve_cmdline_tools_url()
        print(f"Android cmdline-tools {build}")
        print(f"URL: {url}")
        # Keep top-level cmdline-tools/ then nest under latest/.
        progress = download_progress("Downloading Android cmdline-tools")
        install_archive_from_url(
            url,
            ctx.install_dir,
            strip_top_level=False,
            progress=progress,
        )
        progress.done()
        _layout_cmdline_tools(ctx)
        binary = _sdkmanager(ctx)
        if not binary.is_file():
            raise RuntimeError(f"cmdline-tools extracted but sdkmanager not found at {binary}")
        (ctx.install_dir / MARKER).write_text(build + "\n", encoding="utf-8")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=(
                f"Android cmdline-tools {build} installed at {ctx.install_dir}. "
                "Accept licenses and install packages with sdkmanager "
                "(e.g. platform-tools) as needed for Flutter."
            ),
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        home = str(ctx.install_dir.resolve())
        paths = [
            ctx.install_dir / "cmdline-tools" / "latest" / "bin",
            ctx.install_dir / "platform-tools",
        ]
        return EnvSpec(
            paths=paths,
            vars={
                "ANDROID_HOME": home,
                "ANDROID_SDK_ROOT": home,
            },
        )
