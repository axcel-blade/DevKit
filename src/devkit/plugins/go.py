"""Go plugin — latest stable toolchain from go.dev."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import download_json_value, install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status
from devkit.progress import download_progress

_DL_JSON = "https://go.dev/dl/?mode=json"
MARKER = ".devkit-go"


def _go_os_arch() -> tuple[str, str]:
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        goos = "windows"
    elif host == HostOS.MACOS:
        goos = "darwin"
    elif host == HostOS.LINUX:
        goos = "linux"
    else:
        raise RuntimeError(f"Go is not supported on this OS: {host.value}")
    goarch = "arm64" if arch == "aarch64" else "amd64"
    if arch not in {"x64", "aarch64"}:
        raise RuntimeError(f"Unsupported arch for Go: {arch}")
    return goos, goarch


def resolve_go_download() -> tuple[str, str]:
    """Return ``(download_url, version)`` for the latest stable Go archive."""
    data = download_json_value(_DL_JSON)
    if not isinstance(data, list):
        raise TypeError("Unexpected go.dev/dl JSON")
    goos, goarch = _go_os_arch()
    for entry in data:
        if not isinstance(entry, dict) or not entry.get("stable"):
            continue
        version = str(entry.get("version") or "")
        for f in entry.get("files") or []:
            if not isinstance(f, dict) or f.get("kind") != "archive":
                continue
            if f.get("os") == goos and f.get("arch") == goarch:
                name = str(f.get("filename") or "")
                return f"https://go.dev/dl/{name}", version
    raise RuntimeError(f"No stable Go archive for {goos}/{goarch}")


def _go_bin(ctx: InstallContext) -> Path:
    name = "go.exe" if is_windows() else "go"
    return ctx.install_dir / "bin" / name


class GoPlugin(Plugin):
    id = "go"
    name = "Go"
    description = "Download latest stable Go toolchain and set GOROOT / PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _go_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but go binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_go_download()
        print(f"Go {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading Go")
        # Archive root is go/; strip so bin/ lands in install_dir.
        install_archive_from_url(url, ctx.install_dir, strip_top_level=True, progress=progress)
        progress.done()
        if not _go_bin(ctx).is_file():
            raise RuntimeError(f"Go extracted but binary not found at {_go_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(ctx.install_dir, message=f"Go {version} installed at {ctx.install_dir}")

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        root = str(ctx.install_dir.resolve())
        return EnvSpec(paths=[ctx.install_dir / "bin"], vars={"GOROOT": root, "GO_HOME": root})
