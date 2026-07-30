"""SQLite tools plugin — official sqlite-tools ZIP from sqlite.org."""

from __future__ import annotations

import re
import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status, read_text_url
from devkit.progress import download_progress

_DOWNLOAD_PAGE = "https://www.sqlite.org/download.html"
MARKER = ".devkit-sqlite"


def resolve_sqlite_download() -> tuple[str, str]:
    """Parse sqlite.org download page for the matching sqlite-tools ZIP."""
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        key = "sqlite-tools-win-arm64" if arch == "aarch64" else "sqlite-tools-win-x64"
    elif host == HostOS.LINUX:
        key = "sqlite-tools-linux-x64"  # official page currently ships x64 only
        if arch == "aarch64":
            raise RuntimeError("sqlite.org does not publish linux-arm64 tools yet")
    elif host == HostOS.MACOS:
        key = "sqlite-tools-osx-arm64" if arch == "aarch64" else "sqlite-tools-osx-x64"
    else:
        raise RuntimeError(f"SQLite tools are not supported on this OS: {host.value}")

    html = read_text_url(_DOWNLOAD_PAGE)
    # Product rows look like: 2026/sqlite-tools-win-x64-3530400.zip,size,sha
    pattern = rf"(\d{{4}}/{re.escape(key)}-\d+\.zip)"
    match = re.search(pattern, html)
    if not match:
        raise RuntimeError(f"Could not find {key} on sqlite.org download page")
    rel = match.group(1)
    url = f"https://www.sqlite.org/{rel}"
    version_match = re.search(r"-(\d+)\.zip$", rel)
    version = version_match.group(1) if version_match else "unknown"
    return url, version


def _sqlite3_bin(ctx: InstallContext) -> Path:
    return ctx.install_dir / ("sqlite3.exe" if is_windows() else "sqlite3")


class SqlitePlugin(Plugin):
    id = "sqlite"
    name = "SQLite"
    description = "Download official sqlite-tools ZIP (sqlite3 CLI) and add it to PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _sqlite3_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but sqlite3 binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_sqlite_download()
        print(f"SQLite tools {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading SQLite tools")
        install_archive_from_url(url, ctx.install_dir, strip_top_level=False, progress=progress)
        progress.done()
        if not _sqlite3_bin(ctx).is_file():
            raise RuntimeError(f"SQLite tools extracted but sqlite3 not found at {_sqlite3_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            ctx.install_dir,
            message=f"SQLite tools {version} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(
            paths=[ctx.install_dir],
            vars={"SQLITE_HOME": str(ctx.install_dir.resolve())},
        )
