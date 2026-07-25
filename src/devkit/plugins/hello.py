"""Hello stub plugin — download-style ZIP install into the machine ``dev`` folder."""

from __future__ import annotations

import io
import shutil
import zipfile
from pathlib import Path

from devkit.download import install_zip_file
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)

MARKER = ".devkit-hello"
BIN_NAME = "bin"


def _build_demo_zip(path: Path) -> Path:
    """Create a tiny SDK-like ZIP (top-level folder + bin/) for the stub."""
    path.parent.mkdir(parents=True, exist_ok=True)
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", compression=zipfile.ZIP_DEFLATED) as zf:
        zf.writestr("hello-sdk/bin/hello.txt", "hello from DevKit\n")
        zf.writestr("hello-sdk/README.txt", "DevKit hello stub SDK\n")
    path.write_bytes(buf.getvalue())
    return path


class HelloPlugin(Plugin):
    """Stub that builds a ZIP, extracts under ``C:\\dev\\hello``, and sets PATH."""

    id = "hello"
    name = "Hello"
    description = (
        "Stub plugin: ZIP → extract into machine dev folder → set PATH/env "
        "(same flow as Flutter-style SDK installs)."
    )

    def status(self, ctx: InstallContext) -> PluginStatus:
        marker = ctx.install_dir / MARKER
        if marker.is_file():
            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail="Hello marker present",
            )
        if ctx.install_dir.exists() and any(ctx.install_dir.iterdir()):
            return PluginStatus(
                state=InstallState.PARTIAL,
                install_dir=ctx.install_dir,
                detail="Install dir exists without marker",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        # Same pattern a Flutter plugin would use, without a network download:
        # 1) obtain a ZIP  2) extract into C:\dev\hello  3) CLI applies env_spec
        cache_zip = ctx.home / ".cache" / "hello-sdk.zip"
        _build_demo_zip(cache_zip)
        install_zip_file(cache_zip, ctx.install_dir, strip_top_level=True)
        (ctx.install_dir / MARKER).write_text("installed\n", encoding="utf-8")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"extracted ZIP into {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        bin_dir = ctx.install_dir / BIN_NAME
        return EnvSpec(
            paths=[bin_dir],
            vars={
                "DEVKIT_HELLO_ROOT": str(ctx.install_dir.resolve()),
            },
        )
