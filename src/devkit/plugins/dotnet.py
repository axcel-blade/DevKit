""".NET SDK plugin — latest LTS SDK ZIP from Microsoft release metadata."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import download_json, install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status
from devkit.progress import download_progress

# Track .NET LTS channel (8.0). Bump when DevKit moves to a newer LTS.
DOTNET_CHANNEL = "8.0"
_META = f"https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/{DOTNET_CHANNEL}/releases.json"
MARKER = ".devkit-dotnet"


def _rid() -> str:
    host = current_os()
    arch = cpu_arch()
    if arch == "aarch64":
        cpu = "arm64"
    elif arch == "x64":
        cpu = "x64"
    else:
        raise RuntimeError(f"Unsupported arch for .NET SDK: {arch}")
    if host == HostOS.WINDOWS:
        return f"win-{cpu}"
    if host == HostOS.LINUX:
        return f"linux-{cpu}"
    if host == HostOS.MACOS:
        return f"osx-{cpu}"
    raise RuntimeError(f".NET SDK is not supported on this OS: {host.value}")


def resolve_dotnet_download() -> tuple[str, str]:
    """Return ``(sdk_zip_url, version)`` for the latest GA SDK on this RID."""
    meta = download_json(_META)
    rid = _rid()
    releases = meta.get("releases") or []
    if not isinstance(releases, list):
        raise TypeError("Unexpected .NET releases metadata")
    for release in releases:
        if not isinstance(release, dict):
            continue
        sdk = release.get("sdk") or {}
        if not isinstance(sdk, dict):
            continue
        version = str(sdk.get("version") or "").strip()
        files = sdk.get("files") or []
        for f in files:
            if not isinstance(f, dict):
                continue
            name = str(f.get("name") or "")
            url = str(f.get("url") or "")
            if rid in name and name.endswith(".zip") and url:
                return url, version
    raise RuntimeError(f"No .NET SDK ZIP found for RID {rid} in channel {DOTNET_CHANNEL}")


def _dotnet_bin(ctx: InstallContext) -> Path:
    return ctx.install_dir / ("dotnet.exe" if is_windows() else "dotnet")


class DotnetPlugin(Plugin):
    id = "dotnet"
    name = ".NET SDK"
    description = f"Download .NET SDK {DOTNET_CHANNEL} LTS ZIP and set DOTNET_ROOT / PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _dotnet_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but dotnet binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_dotnet_download()
        print(f".NET SDK {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading .NET SDK")
        install_archive_from_url(url, ctx.install_dir, strip_top_level=False, progress=progress)
        progress.done()
        if not _dotnet_bin(ctx).is_file():
            raise RuntimeError(f".NET SDK extracted but dotnet not found at {_dotnet_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            ctx.install_dir,
            message=f".NET SDK {version} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        root = str(ctx.install_dir.resolve())
        return EnvSpec(paths=[ctx.install_dir], vars={"DOTNET_ROOT": root})
