"""kubectl plugin — latest stable binary from dl.k8s.io."""

from __future__ import annotations

import shutil
import stat
from pathlib import Path

from devkit.download import download_file
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status, read_text_url
from devkit.progress import download_progress

MARKER = ".devkit-kubectl"


def resolve_kubectl_download() -> tuple[str, str]:
    version = read_text_url("https://dl.k8s.io/release/stable.txt")
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        goos = "windows"
        name = "kubectl.exe"
    elif host == HostOS.LINUX:
        goos = "linux"
        name = "kubectl"
    elif host == HostOS.MACOS:
        goos = "darwin"
        name = "kubectl"
    else:
        raise RuntimeError(f"kubectl is not supported on this OS: {host.value}")
    goarch = "arm64" if arch == "aarch64" else "amd64"
    url = f"https://dl.k8s.io/release/{version}/bin/{goos}/{goarch}/{name}"
    return url, version


def _kubectl_bin(ctx: InstallContext) -> Path:
    return ctx.install_dir / ("kubectl.exe" if is_windows() else "kubectl")


class KubectlPlugin(Plugin):
    id = "kubectl"
    name = "kubectl"
    description = "Download latest stable kubectl binary and add it to PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _kubectl_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but kubectl binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_kubectl_download()
        print(f"kubectl {version}")
        print(f"URL: {url}")
        ctx.install_dir.mkdir(parents=True, exist_ok=True)
        dest = _kubectl_bin(ctx)
        progress = download_progress("Downloading kubectl")
        download_file(url, dest=dest, progress=progress)
        progress.done()
        if not is_windows():
            mode = dest.stat().st_mode
            dest.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            ctx.install_dir,
            message=f"kubectl {version} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(
            paths=[ctx.install_dir],
            vars={"KUBECTL_HOME": str(ctx.install_dir.resolve())},
        )
