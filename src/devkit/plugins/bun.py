"""Bun plugin — latest ZIP from oven-sh/bun GitHub releases."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status, github_latest_release, pick_release_asset
from devkit.progress import download_progress

MARKER = ".devkit-bun"


def resolve_bun_download() -> tuple[str, str]:
    release = github_latest_release("oven-sh", "bun")
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        needle = "bun-windows-aarch64.zip" if arch == "aarch64" else "bun-windows-x64.zip"
    elif host == HostOS.LINUX:
        needle = "bun-linux-aarch64.zip" if arch == "aarch64" else "bun-linux-x64.zip"
    elif host == HostOS.MACOS:
        needle = "bun-darwin-aarch64.zip" if arch == "aarch64" else "bun-darwin-x64.zip"
    else:
        raise RuntimeError(f"Bun is not supported on this OS: {host.value}")
    # Prefer exact asset name (skip -profile / -baseline variants).
    for asset in release.get("assets") or []:
        if isinstance(asset, dict) and asset.get("name") == needle:
            url = str(asset.get("browser_download_url") or "")
            tag = str(release.get("tag_name") or "unknown")
            if url:
                return url, tag
    return pick_release_asset(release, needle.replace(".zip", ""), ".zip")


def _bun_bin(ctx: InstallContext) -> Path:
    # ZIP often contains bun-<plat>/bun(.exe); after strip or nested search.
    name = "bun.exe" if is_windows() else "bun"
    direct = ctx.install_dir / name
    if direct.is_file():
        return direct
    matches = list(ctx.install_dir.rglob(name))
    files = [m for m in matches if m.is_file()]
    if files:
        files.sort(key=lambda p: (len(p.parts), str(p)))
        return files[0]
    return direct


def _layout_bun(ctx: InstallContext) -> None:
    """If archive nested bun under a subfolder, flatten binary to install_dir."""
    binary = _bun_bin(ctx)
    target = ctx.install_dir / binary.name
    if binary.is_file() and binary != target:
        shutil.copy2(binary, target)


class BunPlugin(Plugin):
    id = "bun"
    name = "Bun"
    description = "Download latest Bun runtime ZIP and add it to PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _bun_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but bun binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_bun_download()
        print(f"Bun {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading Bun")
        install_archive_from_url(url, ctx.install_dir, strip_top_level=True, progress=progress)
        progress.done()
        _layout_bun(ctx)
        if not _bun_bin(ctx).is_file():
            raise RuntimeError(f"Bun extracted but binary not found under {ctx.install_dir}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(ctx.install_dir, message=f"Bun {version} installed at {ctx.install_dir}")

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        binary = _bun_bin(ctx)
        return EnvSpec(
            paths=[binary.parent if binary.is_file() else ctx.install_dir],
            vars={"BUN_INSTALL": str(ctx.install_dir.resolve())},
        )
