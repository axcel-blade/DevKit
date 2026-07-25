"""PHP Composer plugin — install composer.phar into the machine ``dev`` folder."""

from __future__ import annotations

import shutil
import stat
from pathlib import Path

from devkit.download import download_file
from devkit.platform import is_windows
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)

COMPOSER_PHAR_URL = "https://getcomposer.org/download/latest-stable/composer.phar"
PHAR_NAME = "composer.phar"


def _php_available() -> bool:
    return shutil.which("php") is not None


def _wrapper_path(ctx: InstallContext) -> Path:
    if is_windows():
        return ctx.install_dir / "composer.bat"
    return ctx.install_dir / "composer"


def _write_wrappers(install_dir: Path) -> None:
    phar = install_dir / PHAR_NAME
    if is_windows():
        bat = install_dir / "composer.bat"
        bat.write_text(
            "@echo off\r\n"
            'php "%~dp0composer.phar" %*\r\n',
            encoding="utf-8",
        )
        # Also drop a composer.cmd for shells that prefer .cmd
        cmd = install_dir / "composer.cmd"
        cmd.write_text(
            "@echo off\r\n"
            'php "%~dp0composer.phar" %*\r\n',
            encoding="utf-8",
        )
    else:
        script = install_dir / "composer"
        script.write_text(
            "#!/usr/bin/env bash\n"
            'DIR="$(cd "$(dirname "$0")" && pwd)"\n'
            'exec php "$DIR/composer.phar" "$@"\n',
            encoding="utf-8",
            newline="\n",
        )
        mode = script.stat().st_mode
        script.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

    if not phar.is_file():
        raise RuntimeError(f"Missing {phar}")


class ComposerPlugin(Plugin):
    """Install PHP Composer (composer.phar) globally via PATH."""

    id = "composer"
    name = "Composer"
    description = (
        "Download latest stable Composer (composer.phar) into the machine dev folder. "
        "Requires PHP on PATH."
    )

    def status(self, ctx: InstallContext) -> PluginStatus:
        phar = ctx.install_dir / PHAR_NAME
        wrapper = _wrapper_path(ctx)
        if phar.is_file() and wrapper.is_file():
            detail = str(wrapper)
            if not _php_available():
                detail += " (warning: php not found on PATH)"
            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail=detail,
            )
        if ctx.install_dir.exists() and any(ctx.install_dir.iterdir()):
            return PluginStatus(
                state=InstallState.PARTIAL,
                install_dir=ctx.install_dir,
                detail="Install dir exists but composer files are incomplete",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        if not _php_available():
            print(
                "Warning: `php` was not found on PATH. "
                "Composer is installed, but you need PHP to run it."
            )
        ctx.install_dir.mkdir(parents=True, exist_ok=True)
        dest = ctx.install_dir / PHAR_NAME
        print(f"Downloading Composer from {COMPOSER_PHAR_URL} ...")
        download_file(COMPOSER_PHAR_URL, dest=dest, filename=PHAR_NAME)
        _write_wrappers(ctx.install_dir)
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Composer installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        # Put the wrapper directory on PATH so `composer` resolves.
        return EnvSpec(
            paths=[ctx.install_dir],
            vars={
                "COMPOSER_HOME": str((ctx.install_dir / "home").resolve()),
            },
        )
