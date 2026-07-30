"""Unit tests for MSI helpers (no real msiexec network install)."""

from pathlib import Path

import pytest

from devkit import installers
from devkit.installers import extract_msi_admin


def test_windows_path_normalizes_slashes(monkeypatch, tmp_path: Path):
    monkeypatch.setattr(installers, "os", type("O", (), {"name": "nt"})())
    p = (tmp_path / "dir" / "file.msi").resolve()
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_bytes(b"msi")
    text = installers._windows_path(p)
    assert "\\" in text
    # Drive-relative forward slashes should be gone on Windows normalization.
    assert "/" not in text


def test_extract_msi_admin_missing_file(tmp_path: Path):
    with pytest.raises(FileNotFoundError):
        extract_msi_admin(tmp_path / "missing.msi", tmp_path / "out")
