"""Eclipse Temurin JDK plugin (Adoptium)."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import adoptium_os, cpu_arch, is_windows
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)

# Current LTS line. Override later if DevKit grows a --version flag.
JDK_FEATURE_VERSION = 21


def temurin_download_url(feature: int = JDK_FEATURE_VERSION) -> str:
    """Stable Adoptium URL for the latest GA Temurin JDK on this OS/arch."""
    os_slug = adoptium_os()
    arch = cpu_arch()
    if arch not in {"x64", "aarch64", "x86"}:
        raise RuntimeError(f"Unsupported CPU arch for Temurin JDK: {arch}")
    return (
        f"https://api.adoptium.net/v3/binary/latest/{feature}/ga/"
        f"{os_slug}/{arch}/jdk/hotspot/normal/eclipse"
    )


def _java_bin(ctx: InstallContext) -> Path:
    if is_windows():
        return ctx.install_dir / "bin" / "java.exe"
    return ctx.install_dir / "bin" / "java"


def _print_progress(downloaded: int, total: int | None) -> None:
    if total and total > 0:
        pct = min(100, downloaded * 100 // total)
        mb = downloaded / (1024 * 1024)
        total_mb = total / (1024 * 1024)
        print(f"\rDownloading JDK... {pct}% ({mb:.1f}/{total_mb:.1f} MiB)", end="", flush=True)
    else:
        mb = downloaded / (1024 * 1024)
        print(f"\rDownloading JDK... {mb:.1f} MiB", end="", flush=True)


class JdkPlugin(Plugin):
    """Install Eclipse Temurin JDK into the machine ``dev`` folder."""

    id = "jdk"
    name = "JDK"
    description = (
        f"Download Eclipse Temurin JDK {JDK_FEATURE_VERSION} (LTS) "
        "and set JAVA_HOME / PATH."
    )

    def status(self, ctx: InstallContext) -> PluginStatus:
        java = _java_bin(ctx)
        if java.is_file():
            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail=str(java),
            )
        if ctx.install_dir.exists() and any(ctx.install_dir.iterdir()):
            return PluginStatus(
                state=InstallState.PARTIAL,
                install_dir=ctx.install_dir,
                detail="Install dir exists but java binary is missing",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        url = temurin_download_url()
        print(f"Temurin JDK {JDK_FEATURE_VERSION}")
        print(f"URL: {url}")
        install_archive_from_url(
            url,
            ctx.install_dir,
            strip_top_level=True,
            progress=_print_progress,
        )
        print()
        java = _java_bin(ctx)
        if not java.is_file():
            raise RuntimeError(f"JDK extracted but java not found at {java}")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Temurin JDK {JDK_FEATURE_VERSION} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        java_home = str(ctx.install_dir.resolve())
        return EnvSpec(
            paths=[ctx.install_dir / "bin"],
            vars={"JAVA_HOME": java_home},
        )
