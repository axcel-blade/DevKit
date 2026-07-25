"""MySQL Community Server plugin (8.4 LTS portable archives).

Downloads official noinstall / tar archives into the machine ``dev`` folder and
sets ``MYSQL_HOME`` + PATH. Does not initialize a data directory or start
``mysqld`` — run server setup separately after install.
"""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)
from devkit.progress import download_progress

# Pin MySQL 8.4 LTS; bump when DevKit cuts a release that tracks a newer LTS.
MYSQL_VERSION = "8.4.10"
MYSQL_SERIES = "8.4"
_GET_BASE = f"https://dev.mysql.com/get/Downloads/MySQL-{MYSQL_SERIES}/"
MARKER = ".devkit-mysql"


def resolve_mysql_url() -> tuple[str, str]:
    """Return ``(download_url, version)`` for this OS/arch."""
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        if arch not in {"x64", "aarch64"}:
            raise RuntimeError(f"Unsupported Windows arch for MySQL: {arch}")
        # Official Windows community ZIP is x64 only.
        name = f"mysql-{MYSQL_VERSION}-winx64.zip"
    elif host == HostOS.LINUX:
        arch_slug = "aarch64" if arch == "aarch64" else "x86_64"
        name = f"mysql-{MYSQL_VERSION}-linux-glibc2.28-{arch_slug}.tar.xz"
    elif host == HostOS.MACOS:
        arch_slug = "arm64" if arch == "aarch64" else "x86_64"
        name = f"mysql-{MYSQL_VERSION}-macos15-{arch_slug}.tar.gz"
    else:
        raise RuntimeError(f"MySQL is not supported on this OS: {host.value}")
    return f"{_GET_BASE}{name}", MYSQL_VERSION


def _mysql_client(ctx: InstallContext) -> Path | None:
    if is_windows():
        exe = ctx.install_dir / "bin" / "mysql.exe"
    else:
        exe = ctx.install_dir / "bin" / "mysql"
    return exe if exe.is_file() else None


class MysqlPlugin(Plugin):
    """Install MySQL Community Server binaries into the machine ``dev`` folder."""

    id = "mysql"
    name = "MySQL"
    description = (
        f"Download MySQL Community Server {MYSQL_VERSION} (LTS) portable archive "
        "and set MYSQL_HOME / PATH."
    )

    def status(self, ctx: InstallContext) -> PluginStatus:
        client = _mysql_client(ctx)
        if client:
            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail=str(client),
            )
        if ctx.install_dir.exists() and any(ctx.install_dir.iterdir()):
            return PluginStatus(
                state=InstallState.PARTIAL,
                install_dir=ctx.install_dir,
                detail="Install dir exists but mysql client is missing",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_mysql_url()
        print(f"MySQL Community Server {version}")
        print(f"URL: {url}")
        print(
            "Note: this installs binaries only. Initialize/start the server separately "
            "(e.g. mysqld --initialize-insecure)."
        )
        progress = download_progress("Downloading MySQL")
        install_archive_from_url(
            url,
            ctx.install_dir,
            strip_top_level=True,
            progress=progress,
        )
        progress.done()
        client = _mysql_client(ctx)
        if not client:
            raise RuntimeError(
                f"MySQL archive extracted but mysql client missing under {ctx.install_dir}"
            )
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"MySQL {version} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(
            paths=[ctx.install_dir / "bin"],
            vars={"MYSQL_HOME": str(ctx.install_dir.resolve())},
        )
