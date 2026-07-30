"""pnpm plugin — standalone binary from pnpm GitHub releases."""

from __future__ import annotations

import shutil
import stat
from pathlib import Path

from devkit.download import download_file, install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status, github_latest_release
from devkit.progress import download_progress

MARKER = ".devkit-pnpm"


def resolve_pnpm_download() -> tuple[str, str]:
    release = github_latest_release("pnpm", "pnpm")
    tag = str(release.get("tag_name") or "unknown")
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        name = "pnpm-win32-arm64.zip" if arch == "aarch64" else "pnpm-win32-x64.zip"
    elif host == HostOS.LINUX:
        name = "pnpm-linux-arm64.tar.gz" if arch == "aarch64" else "pnpm-linux-x64.tar.gz"
    elif host == HostOS.MACOS:
        # pnpm currently publishes darwin-arm64; x64 may be unavailable on latest releases.
        name = "pnpm-darwin-arm64.tar.gz" if arch == "aarch64" else "pnpm-darwin-x64.tar.gz"
    else:
        raise RuntimeError(f"pnpm is not supported on this OS: {host.value}")
    for asset in release.get("assets") or []:
        if isinstance(asset, dict) and asset.get("name") == name:
            url = str(asset.get("browser_download_url") or "")
            if url:
                return url, tag
    raise RuntimeError(f"No pnpm asset named {name} in {tag}")


def _pnpm_bin(ctx: InstallContext) -> Path:
    return ctx.install_dir / ("pnpm.exe" if is_windows() else "pnpm")


class PnpmPlugin(Plugin):
    id = "pnpm"
    name = "pnpm"
    description = "Download latest pnpm standalone binary and add it to PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _pnpm_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but pnpm binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_pnpm_download()
        print(f"pnpm {version}")
        print(f"URL: {url}")
        ctx.install_dir.mkdir(parents=True, exist_ok=True)
        progress = download_progress("Downloading pnpm")
        if url.endswith((".zip", ".tar.gz")):
            install_archive_from_url(
                url,
                ctx.install_dir,
                strip_top_level=False,
                progress=progress,
            )
            progress.done()
            # Normalize binary name to pnpm(.exe) at install root when nested.
            name = "pnpm.exe" if is_windows() else "pnpm"
            matches = [p for p in ctx.install_dir.rglob(name) if p.is_file()]
            if not matches:
                raise RuntimeError(f"pnpm archive extracted but {name} not found")
            matches.sort(key=lambda p: (len(p.parts), str(p)))
            target = ctx.install_dir / name
            if matches[0] != target:
                shutil.copy2(matches[0], target)
        else:
            dest = _pnpm_bin(ctx)
            download_file(url, dest=dest, progress=progress)
            progress.done()
            if not is_windows():
                mode = dest.stat().st_mode
                dest.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
        if not _pnpm_bin(ctx).is_file():
            raise RuntimeError(f"pnpm binary missing at {_pnpm_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(ctx.install_dir, message=f"pnpm {version} installed at {ctx.install_dir}")

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(paths=[ctx.install_dir], vars={"PNPM_HOME": str(ctx.install_dir.resolve())})
