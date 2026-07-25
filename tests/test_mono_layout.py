"""Unit tests for Mono MSI layout discovery across nested Windows paths."""

from pathlib import Path

import pytest

from devkit.plugins.mono import _find_mono_bin_dir


@pytest.mark.parametrize(
    "rel",
    [
        "bin/mono.exe",
        "Mono/bin/mono.exe",
        "Program Files/Mono/bin/mono.exe",
        "Program Files (x86)/Mono/bin/mono.exe",
    ],
)
def test_find_mono_bin_dir_nested_windows_layouts(tmp_path: Path, rel: str):
    target = tmp_path / Path(rel)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text("mono", encoding="utf-8")
    found = _find_mono_bin_dir(tmp_path)
    assert found is not None
    assert (found / "mono.exe").is_file()


def test_find_mono_bin_dir_prefers_shallower(tmp_path: Path):
    shallow = tmp_path / "bin" / "mono.exe"
    deep = tmp_path / "x" / "y" / "bin" / "mono.exe"
    for path in (shallow, deep):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("mono", encoding="utf-8")
    found = _find_mono_bin_dir(tmp_path)
    assert found == shallow.parent
