"""Docker CLI plugin — official static client binaries (engine/daemon not included).

Downloads the Docker CLI (+ bundled tools from the static archive) into the machine
``dev`` folder. Talking to a real daemon still requires Docker Engine or Desktop
separately (or ``DOCKER_HOST`` pointing at a remote engine).
"""

from __future__ import annotations

import re
import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status, read_text_url
from devkit.progress import download_progress

MARKER = ".devkit-docker"
_BASE = "https://download.docker.com"


def _channel_dir() -> tuple[str, str, str]:
    """Return ``(os_slug, arch_slug, archive_ext)`` for static Docker downloads."""
    host = current_os()
    arch = cpu_arch()
    if arch == "aarch64":
        arch_slug = "aarch64"
    elif arch == "x64":
        arch_slug = "x86_64"
    else:
        raise RuntimeError(f"Unsupported arch for Docker CLI: {arch}")

    if host == HostOS.LINUX:
        return "linux", arch_slug, "tgz"
    if host == HostOS.MACOS:
        return "mac", arch_slug, "tgz"
    if host == HostOS.WINDOWS:
        if arch != "x64":
            raise RuntimeError("Docker static Windows builds are x86_64 only")
        return "win", "x86_64", "zip"
    raise RuntimeError(f"Docker CLI is not supported on this OS: {host.value}")


def resolve_docker_download() -> tuple[str, str]:
    """Return ``(download_url, version)`` for the latest stable static Docker archive."""
    os_slug, arch_slug, ext = _channel_dir()
    index_url = f"{_BASE}/{os_slug}/static/stable/{arch_slug}/"
    html = read_text_url(index_url)
    versions = re.findall(rf"docker-(\d+\.\d+\.\d+)\.{re.escape(ext)}", html)
    if not versions:
        raise RuntimeError(f"No Docker static archives found at {index_url}")

    def key(v: str) -> tuple[int, ...]:
        return tuple(int(p) for p in v.split("."))

    version = max(versions, key=key)
    name = f"docker-{version}.{ext}"
    return f"{index_url}{name}", version


def _docker_bin(ctx: InstallContext) -> Path:
    name = "docker.exe" if is_windows() else "docker"
    direct = ctx.install_dir / name
    if direct.is_file():
        return direct
    nested = ctx.install_dir / "docker" / name
    if nested.is_file():
        return nested
    matches = [p for p in ctx.install_dir.rglob(name) if p.is_file()]
    if matches:
        matches.sort(key=lambda p: (len(p.parts), str(p)))
        return matches[0]
    return direct


def _layout_docker(ctx: InstallContext) -> None:
    """Flatten ``docker/`` archive contents into ``install_dir`` when needed."""
    binary = _docker_bin(ctx)
    if binary.is_file() and binary.parent != ctx.install_dir:
        # Copy CLI tools from the nested docker/ folder to the install root.
        for item in binary.parent.iterdir():
            dest = ctx.install_dir / item.name
            if item.is_file() and not dest.exists():
                shutil.copy2(item, dest)


class DockerPlugin(Plugin):
    id = "docker"
    name = "Docker CLI"
    description = (
        "Download official static Docker CLI binaries and add them to PATH. "
        "Does not install the Docker engine/daemon."
    )

    def status(self, ctx: InstallContext):
        return binary_status(
            _docker_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but docker binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_docker_download()
        print(f"Docker CLI {version}")
        print(f"URL: {url}")
        print(
            "Note: this installs the client only. Start Docker Engine/Desktop "
            "(or set DOCKER_HOST) to talk to a daemon."
        )
        progress = download_progress("Downloading Docker CLI")
        # Archives contain a top-level docker/ directory with binaries.
        install_archive_from_url(url, ctx.install_dir, strip_top_level=True, progress=progress)
        progress.done()
        _layout_docker(ctx)
        if not _docker_bin(ctx).is_file():
            raise RuntimeError(f"Docker CLI extracted but binary not found under {ctx.install_dir}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            ctx.install_dir,
            message=f"Docker CLI {version} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        binary = _docker_bin(ctx)
        path_dir = binary.parent if binary.is_file() else ctx.install_dir
        return EnvSpec(
            paths=[path_dir],
            vars={"DOCKER_HOME": str(ctx.install_dir.resolve())},
        )
