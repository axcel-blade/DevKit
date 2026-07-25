"""Helpers to extract Windows MSI and macOS PKG installers into a folder.

Used by plugins such as Mono that ship platform installers instead of plain ZIP/tar.
"""

from __future__ import annotations

import shutil
import subprocess
import tempfile
from pathlib import Path


def extract_msi_admin(msi_path: Path | str, dest: Path | str) -> Path:
    """Extract a Windows MSI into ``dest`` via ``msiexec /a`` (no full install)."""
    msi_path = Path(msi_path)
    dest = Path(dest)
    if dest.exists():
        shutil.rmtree(dest)
    dest.mkdir(parents=True, exist_ok=True)

    # Administrative install copies payload files under TARGETDIR.
    cmd = [
        "msiexec",
        "/a",
        str(msi_path),
        "/qn",
        f"TARGETDIR={dest}",
    ]
    result = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if result.returncode != 0:
        raise RuntimeError(
            "msiexec failed to extract MSI "
            f"(exit {result.returncode}): {result.stderr or result.stdout}"
        )
    return dest


def extract_pkg(pkg_path: Path | str, dest: Path | str) -> Path:
    """Extract a macOS .pkg into ``dest`` using ``pkgutil`` + ``cpio``."""
    pkg_path = Path(pkg_path)
    dest = Path(dest)
    if dest.exists():
        shutil.rmtree(dest)
    dest.mkdir(parents=True, exist_ok=True)

    with tempfile.TemporaryDirectory() as tmp:
        expanded = Path(tmp) / "expanded"
        expand = subprocess.run(
            ["pkgutil", "--expand", str(pkg_path), str(expanded)],
            capture_output=True,
            text=True,
            check=False,
        )
        if expand.returncode != 0:
            raise RuntimeError(
                f"pkgutil --expand failed: {expand.stderr or expand.stdout}"
            )

        # Prefer the largest Payload (main framework package).
        payloads = list(expanded.rglob("Payload"))
        if not payloads:
            raise RuntimeError(f"No Payload found inside {pkg_path.name}")
        payload = max(payloads, key=lambda p: p.stat().st_size)

        extract_dir = Path(tmp) / "root"
        extract_dir.mkdir()
        # Payload is usually a gzip-compressed cpio archive.
        with payload.open("rb") as fh:
            gunzip = subprocess.run(
                ["gunzip", "-dc"],
                stdin=fh,
                capture_output=True,
                check=False,
            )
        if gunzip.returncode != 0:
            # Some payloads are raw cpio
            data = payload.read_bytes()
        else:
            data = gunzip.stdout

        cpio = subprocess.run(
            ["cpio", "-id"],
            input=data,
            cwd=extract_dir,
            capture_output=True,
            check=False,
        )
        if cpio.returncode != 0:
            raise RuntimeError(
                f"cpio extract failed: {cpio.stderr.decode(errors='replace')}"
            )

        # Mono MDK unpacks under Library/Frameworks/Mono.framework/...
        # Copy everything into dest so we can point PATH at Commands.
        for item in extract_dir.iterdir():
            target = dest / item.name
            if item.is_dir():
                shutil.copytree(item, target)
            else:
                shutil.copy2(item, target)

    return dest
