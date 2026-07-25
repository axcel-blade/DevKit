"""Flutter SDK plugin — download a channel/version into the machine ``dev`` folder."""

from __future__ import annotations

import platform
import shutil

from devkit.download import download_json, install_archive_from_url
from devkit.platform import HostOS, current_os, is_windows
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)
from devkit.progress import download_progress

_RELEASES_BASE = "https://storage.googleapis.com/flutter_infra_release/releases"
FLUTTER_CHANNELS = ("stable", "beta", "dev")


def _releases_platform() -> str:
    host = current_os()
    if host == HostOS.WINDOWS:
        return "windows"
    if host == HostOS.MACOS:
        return "macos"
    if host == HostOS.LINUX:
        return "linux"
    raise RuntimeError(f"Flutter is not supported on this OS: {host.value}")


def _arch_hints() -> tuple[str, ...]:
    machine = platform.machine().lower()
    if machine in {"arm64", "aarch64"}:
        return ("arm64", "aarch64")
    if machine in {"x86_64", "amd64", "x64"}:
        return ("x64", "x86_64", "amd64")
    return (machine,)


def _score_archive(archive: str) -> int:
    """Higher score = better match for this machine (prefer arch-specific builds)."""
    name = archive.lower()
    hints = _arch_hints()
    score = 0
    for hint in hints:
        if hint in name:
            score += 10
    # Prefer non-arm builds when we are on x64 (avoid accidental arm archives).
    if "arm64" not in hints and "arm64" in name:
        score -= 20
    return score


def _normalize_channel(channel: str | None) -> str:
    value = (channel or "stable").strip().lower()
    if value not in FLUTTER_CHANNELS:
        known = ", ".join(FLUTTER_CHANNELS)
        raise RuntimeError(f"Unknown Flutter channel '{channel}'. Use one of: {known}")
    return value


def _match_version(release_version: str, requested: str) -> bool:
    """Exact match, or prefix match (``3.24`` matches ``3.24.5``)."""
    rv = release_version.strip()
    req = requested.strip()
    if rv == req:
        return True
    return rv.startswith(req + ".")


def resolve_flutter_url(
    *,
    channel: str = "stable",
    version: str | None = None,
) -> tuple[str, str]:
    """Return ``(download_url, version)`` for a Flutter channel (and optional version)."""
    channel = _normalize_channel(channel)
    plat = _releases_platform()
    meta = download_json(f"{_RELEASES_BASE}/releases_{plat}.json")
    base_url = str(meta.get("base_url", _RELEASES_BASE)).rstrip("/")
    current = meta.get("current_release") or {}
    releases = meta.get("releases") or []
    if not isinstance(releases, list):
        raise TypeError("Unexpected Flutter releases metadata")

    candidates = [
        r
        for r in releases
        if isinstance(r, dict) and r.get("channel") == channel and r.get("archive")
    ]
    if not candidates:
        raise RuntimeError(f"No Flutter releases found for channel '{channel}'")

    if version:
        matched = [
            r for r in candidates if _match_version(str(r.get("version") or ""), version)
        ]
        if not matched:
            raise RuntimeError(
                f"No Flutter {channel} release matching version '{version}'"
            )
        candidates = matched
    else:
        channel_hash = current.get(channel)
        if channel_hash:
            hashed = [r for r in candidates if r.get("hash") == channel_hash]
            if hashed:
                candidates = hashed

    best = max(candidates, key=lambda r: _score_archive(str(r["archive"])))
    archive = str(best["archive"])
    resolved = str(best.get("version", "unknown"))
    return f"{base_url}/{archive}", resolved


def resolve_stable_flutter_url() -> tuple[str, str]:
    """Return ``(download_url, version)`` for the current OS stable channel."""
    return resolve_flutter_url(channel="stable")


def _flutter_bin(ctx: InstallContext):
    if is_windows():
        return ctx.install_dir / "bin" / "flutter.bat"
    return ctx.install_dir / "bin" / "flutter"


class FlutterPlugin(Plugin):
    """Install the Flutter SDK for Windows, macOS, or Linux."""

    id = "flutter"
    name = "Flutter"
    description = (
        "Download Flutter SDK into the machine dev folder "
        "(optional --channel / --version)."
    )

    def status(self, ctx: InstallContext) -> PluginStatus:
        binary = _flutter_bin(ctx)
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
                detail="Install dir exists but flutter binary is missing",
            )
        return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        channel = _normalize_channel(ctx.channel)
        url, version = resolve_flutter_url(channel=channel, version=ctx.version)
        print(f"Flutter {channel} {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading Flutter")
        install_archive_from_url(
            url,
            ctx.install_dir,
            strip_top_level=True,
            progress=progress,
        )
        progress.done()
        binary = _flutter_bin(ctx)
        if not binary.is_file():
            raise RuntimeError(
                f"Flutter archive extracted but binary not found at {binary}"
            )
        return InstallResult(
            install_dir=ctx.install_dir,
            message=f"Flutter {channel} {version} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(
            paths=[ctx.install_dir / "bin"],
            vars={"FLUTTER_ROOT": str(ctx.install_dir.resolve())},
        )
