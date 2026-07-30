"""Deno plugin — latest ZIP from denoland/deno GitHub releases."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status, github_latest_release, pick_release_asset
from devkit.progress import download_progress

MARKER = ".devkit-deno"


def resolve_deno_download() -> tuple[str, str]:
    release = github_latest_release("denoland", "deno")
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        if arch == "aarch64":
            return pick_release_asset(release, "deno-aarch64-pc-windows-msvc.zip")
        return pick_release_asset(release, "deno-x86_64-pc-windows-msvc.zip")
    if host == HostOS.LINUX:
        if arch == "aarch64":
            return pick_release_asset(release, "deno-aarch64-unknown-linux-gnu.zip")
        return pick_release_asset(release, "deno-x86_64-unknown-linux-gnu.zip")
    if host == HostOS.MACOS:
        if arch == "aarch64":
            return pick_release_asset(release, "deno-aarch64-apple-darwin.zip")
        return pick_release_asset(release, "deno-x86_64-apple-darwin.zip")
    raise RuntimeError(f"Deno is not supported on this OS: {host.value}")


def _deno_bin(ctx: InstallContext) -> Path:
    return ctx.install_dir / ("deno.exe" if is_windows() else "deno")


class DenoPlugin(Plugin):
    id = "deno"
    name = "Deno"
    description = "Download latest Deno runtime ZIP and add it to PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _deno_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but deno binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_deno_download()
        print(f"Deno {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading Deno")
        install_archive_from_url(url, ctx.install_dir, strip_top_level=False, progress=progress)
        progress.done()
        if not _deno_bin(ctx).is_file():
            raise RuntimeError(f"Deno extracted but binary not found at {_deno_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(ctx.install_dir, message=f"Deno {version} installed at {ctx.install_dir}")

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(paths=[ctx.install_dir], vars={"DENO_INSTALL": str(ctx.install_dir.resolve())})
