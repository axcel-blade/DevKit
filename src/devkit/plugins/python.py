"""Python plugin — portable CPython from python-build-standalone (all platforms)."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status
from devkit.progress import download_progress

# Pin a known-good CPython + python-build-standalone release pair.
PYTHON_VERSION = "3.12.13"
PBS_TAG = "20260728"
MARKER = ".devkit-python"


def _pbs_triple() -> str:
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        cpu = "aarch64" if arch == "aarch64" else "x86_64"
        return f"{cpu}-pc-windows-msvc"
    if host == HostOS.LINUX:
        cpu = "aarch64" if arch == "aarch64" else "x86_64"
        return f"{cpu}-unknown-linux-gnu"
    if host == HostOS.MACOS:
        cpu = "aarch64" if arch == "aarch64" else "x86_64"
        return f"{cpu}-apple-darwin"
    raise RuntimeError(f"Python is not supported on this OS: {host.value}")


def resolve_python_download() -> tuple[str, str]:
    """Return ``(download_url, version)`` for this OS/arch."""
    if cpu_arch() not in {"x64", "aarch64"}:
        raise RuntimeError(f"Unsupported arch for Python: {cpu_arch()}")
    triple = _pbs_triple()
    # '+' must be URL-encoded in the asset name.
    asset = f"cpython-{PYTHON_VERSION}%2B{PBS_TAG}-{triple}-install_only.tar.gz"
    url = (
        "https://github.com/astral-sh/python-build-standalone/releases/download/"
        f"{PBS_TAG}/{asset}"
    )
    return url, PYTHON_VERSION


def _python_bin(ctx: InstallContext) -> Path:
    if is_windows():
        for candidate in (
            ctx.install_dir / "python.exe",
            ctx.install_dir / "bin" / "python.exe",
        ):
            if candidate.is_file():
                return candidate
        return ctx.install_dir / "python.exe"
    for candidate in (
        ctx.install_dir / "bin" / "python3",
        ctx.install_dir / "bin" / "python",
    ):
        if candidate.is_file():
            return candidate
    return ctx.install_dir / "bin" / "python3"


class PythonPlugin(Plugin):
    id = "python"
    name = "Python"
    description = (
        f"Download portable Python {PYTHON_VERSION} (python-build-standalone) "
        "and set PYTHON_HOME / PATH."
    )

    def status(self, ctx: InstallContext):
        return binary_status(
            _python_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but python binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_python_download()
        print(f"Python {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading Python")
        # Archive root is python/; strip so bin/ lands in install_dir.
        install_archive_from_url(url, ctx.install_dir, strip_top_level=True, progress=progress)
        progress.done()
        if not _python_bin(ctx).is_file():
            raise RuntimeError(f"Python extracted but binary not found at {_python_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            ctx.install_dir,
            message=f"Python {version} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        home = str(ctx.install_dir.resolve())
        bin_dir = _python_bin(ctx).parent if _python_bin(ctx).is_file() else ctx.install_dir / "bin"
        return EnvSpec(paths=[bin_dir], vars={"PYTHON_HOME": home})
