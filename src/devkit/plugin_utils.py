"""Shared helpers for simple binary/archive plugins.

Keeps GitHub release lookup and status checks consistent across the many
SDK plugins added in 0.5.0 (Go, CMake, Ninja, Deno, Bun, etc.).
"""

from __future__ import annotations

import json
import re
import urllib.error
import urllib.request
from pathlib import Path

from devkit.plugin import InstallState, PluginStatus


def github_latest_release(owner: str, repo: str) -> dict:
    """Fetch the latest GitHub release JSON for ``owner/repo``."""
    url = f"https://api.github.com/repos/{owner}/{repo}/releases/latest"
    req = urllib.request.Request(url, headers={"User-Agent": "DevKit", "Accept": "application/vnd.github+json"})
    try:
        with urllib.request.urlopen(req) as resp:
            data = json.loads(resp.read().decode("utf-8"))
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Failed to query GitHub release {owner}/{repo}: {exc}") from exc
    if not isinstance(data, dict):
        raise TypeError(f"Unexpected GitHub release payload for {owner}/{repo}")
    return data


def pick_release_asset(release: dict, *name_substrings: str) -> tuple[str, str]:
    """Return ``(browser_download_url, tag_name)`` for the first matching asset name."""
    tag = str(release.get("tag_name") or "unknown")
    assets = release.get("assets") or []
    for asset in assets:
        if not isinstance(asset, dict):
            continue
        name = str(asset.get("name") or "")
        url = str(asset.get("browser_download_url") or "")
        if not url:
            continue
        if all(s in name for s in name_substrings):
            return url, tag
    raise RuntimeError(
        f"No GitHub asset matching {name_substrings!r} in {release.get('html_url') or tag}"
    )


def binary_status(
    binary: Path,
    install_dir: Path,
    *,
    missing_detail: str,
) -> PluginStatus:
    """Common installed / partial / not_installed status for a marker binary."""
    if binary.is_file():
        return PluginStatus(
            state=InstallState.INSTALLED,
            install_dir=install_dir,
            detail=str(binary),
        )
    if install_dir.exists() and any(install_dir.iterdir()):
        return PluginStatus(
            state=InstallState.PARTIAL,
            install_dir=install_dir,
            detail=missing_detail,
        )
    return PluginStatus(state=InstallState.NOT_INSTALLED, install_dir=install_dir)


def read_text_url(url: str) -> str:
    """Download a small text document (e.g. kubectl stable.txt)."""
    try:
        with urllib.request.urlopen(url) as resp:
            return resp.read().decode("utf-8").strip()
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Failed to download {url}: {exc}") from exc


def parse_maven_latest_version(html: str) -> str:
    """Pick the newest ``X.Y.Z/`` directory entry from Apache Maven listing HTML."""
    versions = re.findall(r'href="(\d+\.\d+\.\d+)/"', html)
    if not versions:
        raise RuntimeError("Could not parse Apache Maven version listing")
    def key(v: str) -> tuple[int, ...]:
        return tuple(int(p) for p in v.split("."))
    return max(versions, key=key)
