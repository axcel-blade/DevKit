"""Windows CI smoke: download Mono MSI, admin-extract, verify mono.exe layout."""

from __future__ import annotations

import os
import sys
from pathlib import Path

import pytest

from devkit.download import download_file
from devkit.installers import extract_msi_admin
from devkit.plugins.mono import MONO_VERSION, MONO_WINDOWS_URL, _find_mono_bin_dir
from devkit.progress import download_progress


@pytest.mark.smoke
@pytest.mark.skipif(sys.platform != "win32", reason="Mono MSI smoke is Windows-only")
def test_mono_msi_admin_extract_layout(tmp_path: Path):
    """Real msiexec /a extract — run on windows-latest in CI.

    Uses a short path under RUNNER_TEMP (or the drive temp root) because long
    pytest tmp paths under AppData have caused msiexec exit 1603 on Actions.
    """
    if os.environ.get("DEVKIT_MONO_MSI_SMOKE") != "1":
        pytest.skip("Set DEVKIT_MONO_MSI_SMOKE=1 to run Mono MSI smoke")

    # Prefer short CI paths; fall back to pytest tmp for local runs.
    root = Path(os.environ.get("RUNNER_TEMP") or os.environ.get("TEMP") or tmp_path)
    work = root / "devkit-mono-msi-smoke"
    if work.exists():
        import shutil

        shutil.rmtree(work)
    work.mkdir(parents=True)

    os.environ["DEVKIT_HOME"] = str(work / "dev")

    progress = download_progress("Downloading Mono MSI (smoke)")
    msi = download_file(
        MONO_WINDOWS_URL,
        filename=f"mono-{MONO_VERSION}-x64.msi",
        progress=progress,
    )
    progress.done()
    assert msi.is_file() and msi.stat().st_size > 1_000_000

    dest = work / "extract"
    extract_msi_admin(msi, dest)
    bin_dir = _find_mono_bin_dir(dest)
    assert bin_dir is not None, f"mono.exe not found under {dest}"
    assert (bin_dir / "mono.exe").is_file()
