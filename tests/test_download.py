"""Tests for ZIP/tar download/extract helpers."""

import io
import tarfile
import zipfile
from pathlib import Path

from devkit.download import extract_archive, extract_tar, extract_zip, install_zip_file
from devkit.platform import HostOS


def _make_zip(path: Path, mapping: dict[str, str]) -> Path:
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w") as zf:
        for name, content in mapping.items():
            zf.writestr(name, content)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(buf.getvalue())
    return path


def _make_tar_gz(path: Path, mapping: dict[str, str]) -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(path, "w:gz") as tf:
        for name, content in mapping.items():
            data = content.encode("utf-8")
            info = tarfile.TarInfo(name=name)
            info.size = len(data)
            tf.addfile(info, io.BytesIO(data))
    return path


def test_extract_zip_strips_top_level(tmp_path: Path):
    zpath = _make_zip(
        tmp_path / "sdk.zip",
        {
            "flutter/bin/flutter": "#!/bin/sh\n",
            "flutter/README.md": "Flutter\n",
        },
    )
    dest = tmp_path / "out"
    extract_zip(zpath, dest, strip_top_level=True)
    assert (dest / "bin" / "flutter").is_file()
    assert (dest / "README.md").is_file()
    assert not (dest / "flutter").exists()


def test_extract_zip_keeps_top_level(tmp_path: Path):
    zpath = _make_zip(
        tmp_path / "sdk.zip",
        {"flutter/bin/flutter": "x", "other/file.txt": "y"},
    )
    dest = tmp_path / "out"
    extract_zip(zpath, dest, strip_top_level=True)
    assert (dest / "flutter" / "bin" / "flutter").is_file()
    assert (dest / "other" / "file.txt").is_file()


def test_install_zip_file(tmp_path: Path):
    zpath = _make_zip(
        tmp_path / "hello.zip",
        {"hello-sdk/bin/hello.txt": "hi\n"},
    )
    dest = tmp_path / "hello"
    install_zip_file(zpath, dest)
    assert (dest / "bin" / "hello.txt").read_text(encoding="utf-8") == "hi\n"


def test_extract_tar_gz_strips_top_level(tmp_path: Path):
    tpath = _make_tar_gz(
        tmp_path / "sdk.tar.gz",
        {
            "flutter/bin/flutter": "#!/bin/sh\n",
            "flutter/README.md": "Flutter\n",
        },
    )
    dest = tmp_path / "out"
    extract_tar(tpath, dest, strip_top_level=True)
    assert (dest / "bin" / "flutter").is_file()
    assert (dest / "README.md").is_file()


def test_extract_archive_auto_detects(tmp_path: Path):
    zpath = _make_zip(tmp_path / "a.zip", {"sdk/bin/x": "1"})
    tpath = _make_tar_gz(tmp_path / "a.tar.gz", {"sdk/bin/y": "2"})
    extract_archive(zpath, tmp_path / "z")
    extract_archive(tpath, tmp_path / "t")
    assert (tmp_path / "z" / "bin" / "x").is_file()
    assert (tmp_path / "t" / "bin" / "y").is_file()


def test_install_archive_from_urls_picks_os(monkeypatch, tmp_path: Path):
    from devkit import download as download_mod

    zpath = _make_zip(
        tmp_path / "win.zip",
        {"sdk/bin/tool.bat": "@echo off\n"},
    )

    monkeypatch.setattr("devkit.download.pick_for_os", lambda mapping: "https://example/win.zip")
    monkeypatch.setattr(
        download_mod,
        "download_file",
        lambda url, filename=None, progress=None: zpath,
    )
    monkeypatch.setattr("devkit.platform.current_os", lambda: HostOS.WINDOWS)

    dest = tmp_path / "tool"
    download_mod.install_archive_from_urls(
        {"windows": "https://example/win.zip", "linux": "https://example/linux.tar.xz"},
        dest,
    )
    assert (dest / "bin" / "tool.bat").is_file()
