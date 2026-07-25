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
from devkit.progress import download_progress

# Default LTS line when ``install jdk`` is run without ``--version``.
JDK_FEATURE_VERSION = 21


def parse_jdk_feature(version: str | None) -> int:
    """Parse a feature version from ``--version`` (``21`` or ``21.0.2`` → 21)."""
    if version is None or not str(version).strip():
        return JDK_FEATURE_VERSION
    raw = str(version).strip().lower()
    if raw.startswith("jdk"):
        raw = raw[3:].lstrip("-")
    major = raw.split(".", 1)[0]
    if not major.isdigit():
        raise RuntimeError(
            f"Invalid JDK version '{version}'. Use a feature number like 17 or 21."
        )
    feature = int(major)
    if feature < 8:
        raise RuntimeError(f"Unsupported JDK feature version: {feature}")
    return feature


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


class JdkPlugin(Plugin):
    """Install Eclipse Temurin JDK into the machine ``dev`` folder."""

    id = "jdk"
    name = "JDK"
    description = (
        f"Download Eclipse Temurin JDK (default {JDK_FEATURE_VERSION} LTS; "
        "optional --version for feature line) and set JAVA_HOME / PATH."
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
        feature = parse_jdk_feature(ctx.version)
        url = temurin_download_url(feature)
        print(f"Temurin JDK {feature}")
        print(f"URL: {url}")
        progress = download_progress("Downloading JDK")
        install_archive_from_url(
            url,
            ctx.install_dir,
            strip_top_level=True,
            progress=progress,
        )
        progress.done()
        java = _java_bin(ctx)
        if not java.is_file():
            raise RuntimeError(f"JDK extracted but java not found at {java}")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Temurin JDK {feature} installed at {ctx.install_dir}",
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
