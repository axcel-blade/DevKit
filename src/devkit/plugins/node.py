"""Node.js plugin — latest LTS binary from nodejs.org into the machine ``dev`` folder.

Installs ``node``, ``npm``, and ``npx``. Sets ``NODE_HOME`` and prepends ``bin`` to PATH.
"""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import download_json_value, install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)
from devkit.progress import download_progress

_DIST_INDEX = "https://nodejs.org/dist/index.json"
_DIST_BASE = "https://nodejs.org/dist"
MARKER = ".devkit-node"


def _archive_file_key() -> str:
    """Match a key from Node's ``files`` list for this OS/arch."""
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        if arch == "aarch64":
            return "win-arm64-zip"
        if arch == "x64":
            return "win-x64-zip"
        raise RuntimeError(f"Unsupported Windows arch for Node.js: {arch}")
    if host == HostOS.LINUX:
        if arch == "aarch64":
            return "linux-arm64"
        if arch == "x64":
            return "linux-x64"
        raise RuntimeError(f"Unsupported Linux arch for Node.js: {arch}")
    if host == HostOS.MACOS:
        if arch == "aarch64":
            return "osx-arm64-tar"
        if arch == "x64":
            return "osx-x64-tar"
        raise RuntimeError(f"Unsupported macOS arch for Node.js: {arch}")
    raise RuntimeError(f"Node.js is not supported on this OS: {host.value}")


def _archive_name(version: str, file_key: str) -> str:
    """Build the dist archive filename for a Node version + file key."""
    # Strip leading ``v`` from version for the path segment; keep it in filenames.
    ver = version if version.startswith("v") else f"v{version}"
    mapping = {
        "win-x64-zip": f"node-{ver}-win-x64.zip",
        "win-arm64-zip": f"node-{ver}-win-arm64.zip",
        "linux-x64": f"node-{ver}-linux-x64.tar.xz",
        "linux-arm64": f"node-{ver}-linux-arm64.tar.xz",
        "osx-x64-tar": f"node-{ver}-darwin-x64.tar.gz",
        "osx-arm64-tar": f"node-{ver}-darwin-arm64.tar.gz",
    }
    name = mapping.get(file_key)
    if not name:
        raise RuntimeError(f"No Node.js archive mapping for file key: {file_key}")
    return name


def resolve_node_lts_download() -> tuple[str, str]:
    """Return ``(download_url, version)`` for the newest LTS release on this OS."""
    data = download_json_value(_DIST_INDEX)
    if not isinstance(data, list):
        raise TypeError("Unexpected Node.js dist index (expected array)")
    file_key = _archive_file_key()
    for entry in data:
        if not isinstance(entry, dict) or not entry.get("lts"):
            continue
        files = entry.get("files") or []
        if file_key not in files:
            continue
        version = str(entry.get("version") or "").strip()
        if not version:
            continue
        name = _archive_name(version, file_key)
        return f"{_DIST_BASE}/{version}/{name}", version
    raise RuntimeError(f"No Node.js LTS release found with file key {file_key}")


def _node_bin(ctx: InstallContext) -> Path:
    if is_windows():
        return ctx.install_dir / "node.exe"
    return ctx.install_dir / "bin" / "node"


class NodePlugin(Plugin):
    """Install Node.js LTS (includes npm / npx) for all platforms."""

    id = "node"
    name = "Node.js"
    description = "Download latest Node.js LTS binary and set NODE_HOME / PATH."

    def status(self, ctx: InstallContext) -> PluginStatus:
        binary = _node_bin(ctx)
        if binary.is_file():
            return PluginStatus(
                state=InstallState.INSTALLED,
                install_dir=ctx.install_dir,
                detail=str(binary),
            )
        if ctx.install_dir.exists() and any(ctx.install_dir.iterdir()):
            return PluginStatus(
                state=InstallState.PARTIAL,
                install_dir=ctx.install_dir,
                detail="Install dir exists but node binary is missing",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_node_lts_download()
        print(f"Node.js LTS {version}")
        print(f"URL: {url}")
        # Archive root is node-vX.Y.Z-<plat>/; strip so node.exe or bin/ lands in install_dir.
        progress = download_progress("Downloading Node.js")
        install_archive_from_url(
            url,
            ctx.install_dir,
            strip_top_level=True,
            progress=progress,
        )
        progress.done()
        binary = _node_bin(ctx)
        if not binary.is_file():
            raise RuntimeError(f"Node.js extracted but binary not found at {binary}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Node.js {version} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        # Windows ZIP lays binaries at the root; Unix uses bin/.
        path_dir = ctx.install_dir if is_windows() else ctx.install_dir / "bin"
        return EnvSpec(
            paths=[path_dir],
            vars={"NODE_HOME": str(ctx.install_dir.resolve())},
        )
