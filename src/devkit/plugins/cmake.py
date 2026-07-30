"""CMake plugin — latest official binary from Kitware GitHub releases."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status, github_latest_release, pick_release_asset
from devkit.progress import download_progress

MARKER = ".devkit-cmake"


def resolve_cmake_download() -> tuple[str, str]:
    release = github_latest_release("Kitware", "CMake")
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        if arch != "x64":
            raise RuntimeError(f"Unsupported Windows arch for CMake ZIP: {arch}")
        return pick_release_asset(release, "windows-x86_64.zip")
    if host == HostOS.LINUX:
        if arch == "aarch64":
            return pick_release_asset(release, "linux-aarch64.tar.gz")
        return pick_release_asset(release, "linux-x86_64.tar.gz")
    if host == HostOS.MACOS:
        return pick_release_asset(release, "macos-universal.tar.gz")
    raise RuntimeError(f"CMake is not supported on this OS: {host.value}")


def _cmake_bin(ctx: InstallContext) -> Path:
    # Kitware archives nest under cmake-<ver>-<plat>/bin after strip, or CMake.app on macOS.
    if is_windows():
        return ctx.install_dir / "bin" / "cmake.exe"
    app = ctx.install_dir / "CMake.app" / "Contents" / "bin" / "cmake"
    if app.is_file():
        return app
    return ctx.install_dir / "bin" / "cmake"


class CmakePlugin(Plugin):
    id = "cmake"
    name = "CMake"
    description = "Download latest CMake binary and set CMAKE_HOME / PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _cmake_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but cmake binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_cmake_download()
        print(f"CMake {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading CMake")
        install_archive_from_url(url, ctx.install_dir, strip_top_level=True, progress=progress)
        progress.done()
        if not _cmake_bin(ctx).is_file():
            raise RuntimeError(f"CMake extracted but binary not found at {_cmake_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(ctx.install_dir, message=f"CMake {version} installed at {ctx.install_dir}")

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        binary = _cmake_bin(ctx)
        return EnvSpec(
            paths=[binary.parent],
            vars={"CMAKE_HOME": str(ctx.install_dir.resolve())},
        )
