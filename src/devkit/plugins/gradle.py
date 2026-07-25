"""Gradle plugin — download the latest binary distribution into the machine ``dev`` folder.

Uses the official Gradle current-version API (same ZIP for Windows, macOS, and Linux).
Sets ``GRADLE_HOME`` and prepends ``bin`` to PATH.
"""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import download_json, install_archive_from_url
from devkit.platform import is_windows
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)
from devkit.progress import download_progress

_CURRENT_API = "https://services.gradle.org/versions/current"
MARKER = ".devkit-gradle"


def resolve_gradle_download() -> tuple[str, str]:
    """Return ``(download_url, version)`` for the current Gradle release."""
    meta = download_json(_CURRENT_API)
    version = str(meta.get("version") or "").strip()
    url = str(meta.get("downloadUrl") or "").strip()
    if not version or not url:
        raise RuntimeError("Unexpected response from Gradle versions API")
    if meta.get("broken"):
        raise RuntimeError(f"Gradle marks version {version} as broken; refusing to install")
    return url, version


def _gradle_bin(ctx: InstallContext) -> Path:
    if is_windows():
        return ctx.install_dir / "bin" / "gradle.bat"
    return ctx.install_dir / "bin" / "gradle"


class GradlePlugin(Plugin):
    """Install the Gradle binary distribution for all platforms."""

    id = "gradle"
    name = "Gradle"
    description = "Download latest Gradle binary ZIP and set GRADLE_HOME / PATH."

    def status(self, ctx: InstallContext) -> PluginStatus:
        binary = _gradle_bin(ctx)
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
                detail="Install dir exists but gradle binary is missing",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_gradle_download()
        print(f"Gradle {version}")
        print(f"URL: {url}")
        # Archive root is gradle-<version>/; strip so bin/ lands in install_dir.
        progress = download_progress("Downloading Gradle")
        install_archive_from_url(
            url,
            ctx.install_dir,
            strip_top_level=True,
            progress=progress,
        )
        progress.done()
        binary = _gradle_bin(ctx)
        if not binary.is_file():
            raise RuntimeError(f"Gradle extracted but binary not found at {binary}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Gradle {version} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(
            paths=[ctx.install_dir / "bin"],
            vars={"GRADLE_HOME": str(ctx.install_dir.resolve())},
        )
