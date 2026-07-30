"""Helpers to extract Windows MSI and macOS PKG installers into a folder.

Used by plugins such as Mono that ship platform installers instead of plain ZIP/tar.
"""

from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from pathlib import Path


def _windows_path(path: Path) -> str:
    """Return an absolute Windows path with backslashes for msiexec."""
    text = str(path.resolve())
    if os.name == "nt":
        return text.replace("/", "\\")
    return text


def extract_msi_admin(msi_path: Path | str, dest: Path | str) -> Path:
    """Extract a Windows MSI into ``dest`` via ``msiexec /a`` (no full install).

    GitHub Actions and other CI hosts often return opaque exit 1603 when paths use
    forward slashes or when ``TARGETDIR`` is poorly quoted. We normalize paths,
    write a verbose log, and wait for msiexec via PowerShell ``Start-Process``.
    """
    msi_path = Path(msi_path).resolve()
    dest = Path(dest).resolve()
    if not msi_path.is_file():
        raise FileNotFoundError(f"MSI not found: {msi_path}")

    if dest.exists():
        shutil.rmtree(dest)
    # Parent only — let Windows Installer create TARGETDIR itself.
    dest.parent.mkdir(parents=True, exist_ok=True)

    msi_arg = _windows_path(msi_path)
    # Trailing backslash required by many MSIs; keep it out of PS single-quotes.
    dest_arg = _windows_path(dest).rstrip("\\") + "\\"
    log_path = dest.parent / f"{dest.name}.msiexec.log"
    if log_path.exists():
        log_path.unlink()
    log_arg = _windows_path(log_path)

    # Build ArgumentList in PowerShell so a trailing '\' cannot break quoting.
    ps_script = f"""
$ErrorActionPreference = 'Stop'
$msi = '{msi_arg.replace("'", "''")}'
$dest = '{dest_arg.rstrip(chr(92)).replace("'", "''")}' + [char]92
$log = '{log_arg.replace("'", "''")}'
$argList = @('/a', $msi, '/qn', '/norestart', ('TARGETDIR=' + $dest), '/l*v', $log)
$p = Start-Process -FilePath 'msiexec.exe' -ArgumentList $argList -Wait -PassThru
if ($null -eq $p) {{ exit 1 }}
exit $p.ExitCode
"""
    result = subprocess.run(
        [
            "powershell.exe",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            ps_script,
        ],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        log_tail = ""
        if log_path.is_file():
            try:
                text = log_path.read_text(encoding="utf-8", errors="replace")
                log_tail = text[-4000:]
            except OSError:
                log_tail = "(could not read msiexec log)"
        raise RuntimeError(
            "msiexec failed to extract MSI "
            f"(exit {result.returncode}): {result.stderr or result.stdout}\n"
            f"Log ({log_path}):\n{log_tail}"
        )
    if not dest.exists() or not any(dest.iterdir()):
        raise RuntimeError(
            f"msiexec reported success but TARGETDIR is empty: {dest}"
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
