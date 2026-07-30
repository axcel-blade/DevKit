"""PostgreSQL plugin — EDB Windows binaries; system wrappers on Unix."""

from __future__ import annotations

import shutil
import stat
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status
from devkit.progress import download_progress

# Pin a known EDB Windows binaries ZIP; bump with DevKit releases.
PG_VERSION = "16.8-1"
_WINDOWS_URL = (
    f"https://get.enterprisedb.com/postgresql/postgresql-{PG_VERSION}-windows-x64-binaries.zip"
)
MARKER = ".devkit-postgresql"


def _pgsql_bin(ctx: InstallContext) -> Path | None:
    candidates = [
        ctx.install_dir / "bin" / ("psql.exe" if is_windows() else "psql"),
        ctx.install_dir / "pgsql" / "bin" / ("psql.exe" if is_windows() else "psql"),
    ]
    for path in candidates:
        if path.is_file():
            return path
    return None


def _write_unix_wrappers(install_dir: Path, system_psql: Path) -> None:
    bin_dir = install_dir / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    for name in ("psql", "pg_dump", "pg_restore", "createdb", "dropdb"):
        system = shutil.which(name)
        target = bin_dir / name
        if not system and name != "psql":
            continue
        exe = system or str(system_psql)
        target.write_text(
            "#!/usr/bin/env bash\n"
            f'exec "{exe}" "$@"\n',
            encoding="utf-8",
            newline="\n",
        )
        mode = target.stat().st_mode
        target.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    (install_dir / MARKER).write_text(f"system-wrapper\n{system_psql}\n", encoding="utf-8")


class PostgresqlPlugin(Plugin):
    id = "postgresql"
    name = "PostgreSQL"
    description = (
        f"Download PostgreSQL {PG_VERSION} Windows binaries from EDB. "
        "On macOS/Linux, registers system PostgreSQL clients if installed."
    )

    def status(self, ctx: InstallContext):
        binary = _pgsql_bin(ctx)
        if binary:
            return binary_status(
                binary,
                ctx.install_dir,
                missing_detail="Install dir exists but psql is missing",
            )
        if (ctx.install_dir / MARKER).is_file():
            from devkit.plugin import InstallState, PluginStatus

            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail="system postgresql wrappers",
            )
        return binary_status(
            ctx.install_dir / "bin" / "psql",
            ctx.install_dir,
            missing_detail="Install dir exists but psql is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        host = current_os()
        if host == HostOS.WINDOWS:
            return self._install_windows(ctx)
        if host in {HostOS.MACOS, HostOS.LINUX}:
            return self._install_unix(ctx)
        raise RuntimeError(f"PostgreSQL is not supported on this OS: {host.value}")

    def _install_windows(self, ctx: InstallContext) -> InstallResult:
        print(f"PostgreSQL {PG_VERSION} (Windows x64 binaries)")
        print(f"URL: {_WINDOWS_URL}")
        progress = download_progress("Downloading PostgreSQL")
        install_archive_from_url(
            _WINDOWS_URL,
            ctx.install_dir,
            strip_top_level=True,
            progress=progress,
        )
        progress.done()
        binary = _pgsql_bin(ctx)
        if not binary:
            raise RuntimeError(f"PostgreSQL extracted but psql missing under {ctx.install_dir}")
        (ctx.install_dir / MARKER).write_text(PG_VERSION + "\n", encoding="utf-8")
        return InstallResult(
            ctx.install_dir,
            message=f"PostgreSQL {PG_VERSION} installed at {ctx.install_dir}",
        )

    def _install_unix(self, ctx: InstallContext) -> InstallResult:
        system = shutil.which("psql")
        if not system:
            raise RuntimeError(
                "PostgreSQL has no portable Unix archive in DevKit.\n"
                "Install clients with your package manager, then re-run:\n"
                "  Debian/Ubuntu:  sudo apt install postgresql-client\n"
                "  Fedora:         sudo dnf install postgresql\n"
                "  macOS:          brew install libpq && brew link --force libpq\n"
                "Then:  python main.py install postgresql"
            )
        ctx.install_dir.mkdir(parents=True, exist_ok=True)
        _write_unix_wrappers(ctx.install_dir, Path(system))
        return InstallResult(
            ctx.install_dir,
            message=f"Registered system PostgreSQL client at {system}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        binary = _pgsql_bin(ctx)
        paths = [binary.parent] if binary else []
        wrapper = ctx.install_dir / "bin"
        if wrapper.is_dir() and wrapper not in paths:
            paths.append(wrapper)
        return EnvSpec(
            paths=paths,
            vars={"POSTGRESQL_HOME": str(ctx.install_dir.resolve())},
        )
