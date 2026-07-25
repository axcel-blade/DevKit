"""PHP plugin — Windows ZIP from downloads.php.net; system PHP wrappers on Unix.

Pairs with the Composer plugin. On Windows, installs the latest NTS x64 build
(suitable for CLI / Composer). On macOS/Linux, registers wrappers around system PHP.
"""

from __future__ import annotations

import shutil
import stat
from pathlib import Path

from devkit.download import download_json, install_archive_from_url
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

_RELEASES_JSON = "https://downloads.php.net/~windows/releases/releases.json"
_RELEASES_BASE = "https://downloads.php.net/~windows/releases/"
MARKER = ".devkit-php"


def resolve_php_windows_url() -> tuple[str, str]:
    """Pick the newest NTS x64 Windows ZIP from the official releases index."""
    meta = download_json(_RELEASES_JSON)
    # Keys are major.minor strings like "8.4", "8.5".
    candidates: list[tuple[tuple[int, ...], str, str]] = []
    for info in meta.values():
        if not isinstance(info, dict):
            continue
        version = str(info.get("version") or "")
        # Prefer NTS (non-thread-safe) CLI builds; fall back to TS.
        build = None
        for key in info:
            if isinstance(key, str) and key.startswith("nts-") and key.endswith("-x64"):
                build = info[key]
                break
        if build is None:
            for key in info:
                if isinstance(key, str) and key.startswith("ts-") and key.endswith("-x64"):
                    build = info[key]
                    break
        if not isinstance(build, dict):
            continue
        zip_info = build.get("zip")
        if not isinstance(zip_info, dict) or not zip_info.get("path"):
            continue
        try:
            ver_tuple = tuple(int(p) for p in version.split("."))
        except ValueError:
            continue
        path = str(zip_info["path"])
        candidates.append((ver_tuple, version, path))

    if not candidates:
        raise RuntimeError("No suitable Windows PHP ZIP found in releases.json")
    candidates.sort(key=lambda item: item[0], reverse=True)
    _, version, path = candidates[0]
    return f"{_RELEASES_BASE}{path}", version


def _php_binary(ctx: InstallContext) -> Path | None:
    if is_windows():
        exe = ctx.install_dir / "php.exe"
        return exe if exe.is_file() else None
    for rel in ("bin/php", "php"):
        path = ctx.install_dir / rel
        if path.is_file():
            return path
    return None


def _ensure_php_ini(install_dir: Path) -> None:
    """Copy php.ini-development to php.ini when missing (Windows builds)."""
    ini = install_dir / "php.ini"
    if ini.is_file():
        return
    for name in ("php.ini-development", "php.ini-production"):
        src = install_dir / name
        if src.is_file():
            shutil.copy2(src, ini)
            return


def _write_unix_wrappers(install_dir: Path, system_php: Path) -> None:
    bin_dir = install_dir / "bin"
    bin_dir.mkdir(parents=True, exist_ok=True)
    for name in ("php", "php-cgi", "phpdbg"):
        system = shutil.which(name)
        target = bin_dir / name
        if not system and name != "php":
            continue
        exe = system or str(system_php)
        target.write_text(
            "#!/usr/bin/env bash\n"
            f'exec "{exe}" "$@"\n',
            encoding="utf-8",
            newline="\n",
        )
        mode = target.stat().st_mode
        target.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    (install_dir / MARKER).write_text(f"system-wrapper\n{system_php}\n", encoding="utf-8")


class PhpPlugin(Plugin):
    """Install PHP for CLI use (Composer-friendly)."""

    id = "php"
    name = "PHP"
    description = (
        "Install latest PHP NTS x64 ZIP on Windows. "
        "On macOS/Linux, registers system PHP if already installed."
    )

    def status(self, ctx: InstallContext) -> PluginStatus:
        binary = _php_binary(ctx)
        if binary and binary.is_file():
            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail=str(binary),
            )
        if (ctx.install_dir / MARKER).is_file():
            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail="system php wrappers",
            )
        if ctx.install_dir.exists() and any(ctx.install_dir.iterdir()):
            return PluginStatus(
                state=InstallState.PARTIAL,
                install_dir=ctx.install_dir,
                detail="Install dir exists but php binary is missing",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        host = current_os()
        if host == HostOS.WINDOWS:
            return self._install_windows(ctx)
        if host in {HostOS.MACOS, HostOS.LINUX}:
            return self._install_unix(ctx)
        raise RuntimeError(f"PHP is not supported on this OS: {host.value}")

    def _install_windows(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_php_windows_url()
        print(f"PHP {version} (Windows NTS/TS x64)")
        print(f"URL: {url}")
        # Official Windows ZIP has php.exe at the archive root.
        progress = download_progress("Downloading PHP")
        install_archive_from_url(
            url,
            ctx.install_dir,
            strip_top_level=False,
            progress=progress,
        )
        progress.done()
        binary = _php_binary(ctx)
        if not binary:
            raise RuntimeError(f"PHP ZIP extracted but php.exe missing under {ctx.install_dir}")
        _ensure_php_ini(ctx.install_dir)
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"PHP {version} installed at {ctx.install_dir}",
        )

    def _install_unix(self, ctx: InstallContext) -> InstallResult:
        system_php = shutil.which("php")
        if not system_php:
            raise RuntimeError(
                "PHP does not ship a portable macOS/Linux archive for DevKit.\n"
                "Install PHP with your platform tools, then re-run this command:\n"
                "  macOS:          brew install php\n"
                "  Debian/Ubuntu:  sudo apt install php-cli\n"
                "  Fedora:         sudo dnf install php-cli\n"
                "Then:  python main.py install php"
            )
        ctx.install_dir.mkdir(parents=True, exist_ok=True)
        _write_unix_wrappers(ctx.install_dir, Path(system_php))
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Registered system PHP at {system_php}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        # Windows: php.exe is in install_dir root. Unix wrappers live in bin/.
        paths: list[Path] = [ctx.install_dir]
        bin_dir = ctx.install_dir / "bin"
        if bin_dir.is_dir():
            paths.append(bin_dir)
        return EnvSpec(
            paths=paths,
            vars={"PHP_HOME": str(ctx.install_dir.resolve())},
        )
