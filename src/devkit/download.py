"""Download and extract archives into the machine ``dev`` folder.

Supports ZIP (Windows/macOS SDKs) and tar.gz / tar.xz (typical Linux SDKs).
"""

from __future__ import annotations

import json
import shutil
import tarfile
import tempfile
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from collections.abc import Callable, Mapping
from pathlib import Path

from devkit.paths import cache_dir, ensure_home
from devkit.platform import pick_for_os

ProgressCallback = Callable[[int, int | None], None]


def download_json_value(url: str) -> object:
    """Download a JSON document and return the parsed value (object or array).

    Node.js ``index.json`` is a top-level array; most other DevKit indexes are objects.
    """
    try:
        with urllib.request.urlopen(url) as resp:
            raw = resp.read().decode("utf-8")
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Failed to download JSON {url}: {exc}") from exc
    return json.loads(raw)


def download_json(url: str) -> dict:
    """Download a JSON document and return it as a dict."""
    data = download_json_value(url)
    if not isinstance(data, dict):
        raise TypeError(f"Expected JSON object from {url}")
    return data


def _filename_from_response(url: str, resp, explicit: str | None) -> str:
    if explicit:
        return explicit
    # Content-Disposition: attachment; filename="OpenJDK....zip"
    cd = resp.headers.get("Content-Disposition") or ""
    if "filename=" in cd:
        part = cd.split("filename=", 1)[1].strip().strip("\"'")
        if part:
            return Path(part).name
    # Final URL after redirects (Adoptium API, GitHub releases, etc.)
    final = getattr(resp, "geturl", lambda: url)()
    name = Path(urllib.parse.urlparse(final).path).name
    if name and name not in {"", "/", "latest"}:
        return name
    name = Path(urllib.parse.urlparse(url).path).name
    return name or "download.bin"


def download_file(
    url: str,
    dest: Path | None = None,
    *,
    filename: str | None = None,
    progress: ProgressCallback | None = None,
) -> Path:
    """Download ``url`` into the DevKit cache (or ``dest``).

    Returns the path to the downloaded file.
    """
    ensure_home()
    try:
        with urllib.request.urlopen(url) as resp:
            if dest is None:
                name = _filename_from_response(url, resp, filename)
                dest = cache_dir() / name
            else:
                dest = Path(dest)
            dest.parent.mkdir(parents=True, exist_ok=True)

            total = resp.headers.get("Content-Length")
            total_n = int(total) if total and total.isdigit() else None
            downloaded = 0
            with dest.open("wb") as out:
                while True:
                    chunk = resp.read(1024 * 256)
                    if not chunk:
                        break
                    out.write(chunk)
                    downloaded += len(chunk)
                    if progress:
                        progress(downloaded, total_n)
    except urllib.error.URLError as exc:
        raise RuntimeError(f"Failed to download {url}: {exc}") from exc

    return dest


def _unique_top_level(names: list[str]) -> str | None:
    """If all archive members share one top-level folder, return its name."""
    tops: set[str] = set()
    for name in names:
        normalized = name.replace("\\", "/").strip("/")
        if not normalized:
            continue
        tops.add(normalized.split("/", 1)[0])
    if len(tops) == 1:
        return next(iter(tops))
    return None


def _copy_tree_contents(source: Path, dest: Path) -> None:
    for item in source.iterdir():
        target = dest / item.name
        if item.is_dir():
            shutil.copytree(item, target)
        else:
            shutil.copy2(item, target)


def _prepare_dest(dest: Path) -> Path:
    if dest.exists():
        shutil.rmtree(dest)
    dest.mkdir(parents=True, exist_ok=True)
    return dest


def extract_zip(
    zip_path: Path | str,
    dest: Path | str,
    *,
    strip_top_level: bool = True,
) -> Path:
    """Extract a ZIP into ``dest``."""
    zip_path = Path(zip_path)
    dest = _prepare_dest(Path(dest))

    with zipfile.ZipFile(zip_path, "r") as zf:
        names = zf.namelist()
        top = _unique_top_level(names) if strip_top_level else None
        if not top:
            zf.extractall(dest)
            return dest

        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            zf.extractall(tmp_path)
            source = tmp_path / top
            if source.is_dir():
                _copy_tree_contents(source, dest)
            else:
                _copy_tree_contents(tmp_path, dest)
    return dest


def extract_tar(
    tar_path: Path | str,
    dest: Path | str,
    *,
    strip_top_level: bool = True,
) -> Path:
    """Extract a ``.tar``, ``.tar.gz``, ``.tgz``, or ``.tar.xz`` into ``dest``."""
    tar_path = Path(tar_path)
    dest = _prepare_dest(Path(dest))

    with tarfile.open(tar_path, "r:*") as tf:
        members = [m for m in tf.getmembers() if m.name.strip("/")]
        names = [m.name for m in members]
        top = _unique_top_level(names) if strip_top_level else None

        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            tf.extractall(tmp_path, filter="data")
            if top and (tmp_path / top).is_dir():
                _copy_tree_contents(tmp_path / top, dest)
            else:
                # tar may extract into a single folder even if filter differs
                children = list(tmp_path.iterdir())
                if strip_top_level and len(children) == 1 and children[0].is_dir():
                    _copy_tree_contents(children[0], dest)
                else:
                    _copy_tree_contents(tmp_path, dest)
    return dest


def detect_archive_format(path: Path | str) -> str:
    """Return ``zip``, ``tar``, or raise if unknown."""
    path = Path(path)
    name = path.name.lower()
    if name.endswith(".zip"):
        return "zip"
    if name.endswith((".tar.gz", ".tgz", ".tar.xz", ".tar.bz2", ".tar")):
        return "tar"
    # Sniff magic bytes
    with path.open("rb") as fh:
        magic = fh.read(6)
    if magic[:2] == b"PK":
        return "zip"
    if magic[:2] == b"\x1f\x8b" or magic[:5] == b"ustar" or magic[:6] == b"\xfd7zXZ":
        return "tar"
    raise ValueError(f"Unsupported archive format: {path}")


def extract_archive(
    archive_path: Path | str,
    dest: Path | str,
    *,
    strip_top_level: bool = True,
) -> Path:
    """Extract a ZIP or tar archive into ``dest``."""
    kind = detect_archive_format(archive_path)
    if kind == "zip":
        return extract_zip(archive_path, dest, strip_top_level=strip_top_level)
    return extract_tar(archive_path, dest, strip_top_level=strip_top_level)


def install_archive_from_url(
    url: str,
    dest: Path | str,
    *,
    strip_top_level: bool = True,
    filename: str | None = None,
    progress: ProgressCallback | None = None,
) -> Path:
    """Download an archive and extract it into ``dest`` (all platforms)."""
    archive = download_file(url, filename=filename, progress=progress)
    return extract_archive(archive, dest, strip_top_level=strip_top_level)


def install_archive_from_urls(
    urls: Mapping[str, str],
    dest: Path | str,
    *,
    strip_top_level: bool = True,
    progress: ProgressCallback | None = None,
) -> Path:
    """Download the URL for the current OS and extract it.

    Example::

        install_archive_from_urls({
            "windows": "https://.../flutter_windows.zip",
            "macos": "https://.../flutter_macos.zip",
            "linux": "https://.../flutter_linux.tar.xz",
        }, dest)
    """
    url = pick_for_os(dict(urls))
    return install_archive_from_url(
        url, dest, strip_top_level=strip_top_level, progress=progress
    )


# Backwards-compatible ZIP helpers
def install_zip_from_url(
    url: str,
    dest: Path | str,
    *,
    strip_top_level: bool = True,
    filename: str | None = None,
    progress: ProgressCallback | None = None,
) -> Path:
    """Download a ZIP from ``url`` and extract it into ``dest``."""
    return install_archive_from_url(
        url,
        dest,
        strip_top_level=strip_top_level,
        filename=filename,
        progress=progress,
    )


def install_zip_file(
    zip_path: Path | str,
    dest: Path | str,
    *,
    strip_top_level: bool = True,
) -> Path:
    """Extract an existing ZIP file into ``dest``."""
    return extract_zip(zip_path, dest, strip_top_level=strip_top_level)
