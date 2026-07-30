"""Tests for newly added SDK plugins (network mocked where needed)."""

from pathlib import Path

from devkit.platform import HostOS
from devkit.plugin import InstallContext, InstallState
from devkit.plugin_utils import parse_maven_latest_version
from devkit.plugins import bun as bun_mod
from devkit.plugins import cmake as cmake_mod
from devkit.plugins import go as go_mod
from devkit.plugins import kubectl as kubectl_mod
from devkit.plugins import maven as maven_mod
from devkit.plugins import ninja as ninja_mod
from devkit.plugins import platform_tools as pt_mod
from devkit.plugins import python as python_mod
from devkit.plugins import sqlite as sqlite_mod
from devkit.plugins.go import GoPlugin
from devkit.plugins.maven import MavenPlugin


def test_parse_maven_latest_version():
    html = '<a href="3.9.6/">3.9.6/</a><a href="3.9.16/">3.9.16/</a><a href="3.8.9/">3.8.9/</a>'
    assert parse_maven_latest_version(html) == "3.9.16"


def test_resolve_go_download(monkeypatch):
    data = [
        {
            "version": "go1.22.0",
            "stable": True,
            "files": [
                {
                    "filename": "go1.22.0.windows-amd64.zip",
                    "os": "windows",
                    "arch": "amd64",
                    "kind": "archive",
                }
            ],
        }
    ]
    monkeypatch.setattr(go_mod, "download_json_value", lambda url: data)
    monkeypatch.setattr(go_mod, "_go_os_arch", lambda: ("windows", "amd64"))
    url, version = go_mod.resolve_go_download()
    assert version == "go1.22.0"
    assert url.endswith("go1.22.0.windows-amd64.zip")


def test_go_install(monkeypatch, tmp_path: Path):
    plugin = GoPlugin()
    ctx = InstallContext(install_dir=tmp_path / "go", home=tmp_path)
    monkeypatch.setattr(go_mod, "resolve_go_download", lambda: ("https://example.com/go.zip", "go1.22.0"))

    def fake_install(url, dest, strip_top_level=True, progress=None, filename=None):
        dest = Path(dest)
        bin_dir = dest / "bin"
        bin_dir.mkdir(parents=True)
        name = "go.exe" if go_mod.is_windows() else "go"
        (bin_dir / name).write_text("go", encoding="utf-8")
        return dest

    monkeypatch.setattr(go_mod, "install_archive_from_url", fake_install)
    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
    result = plugin.install(ctx)
    assert "go1.22.0" in result.message
    assert plugin.env_spec(ctx).vars["GOROOT"]


def test_resolve_python_windows(monkeypatch):
    monkeypatch.setattr(python_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(python_mod, "cpu_arch", lambda: "x64")
    url, version = python_mod.resolve_python_download()
    assert version == python_mod.PYTHON_VERSION
    assert "x86_64-pc-windows-msvc-install_only.tar.gz" in url
    assert python_mod.PBS_TAG in url


def test_resolve_cmake(monkeypatch):
    release = {
        "tag_name": "v4.0.0",
        "assets": [
            {
                "name": "cmake-4.0.0-windows-x86_64.zip",
                "browser_download_url": "https://example.com/cmake.zip",
            }
        ],
    }
    monkeypatch.setattr(cmake_mod, "github_latest_release", lambda *a: release)
    monkeypatch.setattr(cmake_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(cmake_mod, "cpu_arch", lambda: "x64")
    url, version = cmake_mod.resolve_cmake_download()
    assert version == "v4.0.0"
    assert url.endswith("cmake.zip")


def test_resolve_ninja(monkeypatch):
    release = {
        "tag_name": "v1.12.0",
        "assets": [
            {"name": "ninja-win.zip", "browser_download_url": "https://example.com/ninja-win.zip"}
        ],
    }
    monkeypatch.setattr(ninja_mod, "github_latest_release", lambda *a: release)
    monkeypatch.setattr(ninja_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(ninja_mod, "cpu_arch", lambda: "x64")
    url, version = ninja_mod.resolve_ninja_download()
    assert version == "v1.12.0"
    assert "ninja-win.zip" in url


def test_resolve_maven(monkeypatch):
    monkeypatch.setattr(
        maven_mod,
        "read_text_url",
        lambda url: '<a href="3.9.9/">3.9.9/</a><a href="3.9.10/">3.9.10/</a>',
    )
    url, version = maven_mod.resolve_maven_download()
    assert version == "3.9.10"
    assert url.endswith("apache-maven-3.9.10-bin.zip")


def test_maven_install(monkeypatch, tmp_path: Path):
    plugin = MavenPlugin()
    ctx = InstallContext(install_dir=tmp_path / "maven", home=tmp_path)
    monkeypatch.setattr(maven_mod, "resolve_maven_download", lambda: ("https://ex/m.zip", "3.9.10"))

    def fake_install(url, dest, strip_top_level=True, progress=None, filename=None):
        dest = Path(dest)
        bin_dir = dest / "bin"
        bin_dir.mkdir(parents=True)
        name = "mvn.cmd" if maven_mod.is_windows() else "mvn"
        (bin_dir / name).write_text("mvn", encoding="utf-8")
        return dest

    monkeypatch.setattr(maven_mod, "install_archive_from_url", fake_install)
    result = plugin.install(ctx)
    assert "3.9.10" in result.message
    assert plugin.env_spec(ctx).vars["MAVEN_HOME"]


def test_resolve_kubectl(monkeypatch):
    monkeypatch.setattr(kubectl_mod, "read_text_url", lambda url: "v1.30.0")
    monkeypatch.setattr(kubectl_mod, "current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(kubectl_mod, "cpu_arch", lambda: "x64")
    url, version = kubectl_mod.resolve_kubectl_download()
    assert version == "v1.30.0"
    assert url.endswith("/linux/amd64/kubectl")


def test_resolve_sqlite(monkeypatch):
    html = "2026/sqlite-tools-win-x64-3530400.zip,1,abc\n2026/sqlite-tools-osx-arm64-3530400.zip,1,abc"
    monkeypatch.setattr(sqlite_mod, "read_text_url", lambda url: html)
    monkeypatch.setattr(sqlite_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(sqlite_mod, "cpu_arch", lambda: "x64")
    url, version = sqlite_mod.resolve_sqlite_download()
    assert version == "3530400"
    assert url.endswith("sqlite-tools-win-x64-3530400.zip")


def test_resolve_platform_tools(monkeypatch):
    monkeypatch.setattr(pt_mod, "current_os", lambda: HostOS.WINDOWS)
    url, version = pt_mod.resolve_platform_tools_url()
    assert version == "latest"
    assert "platform-tools-latest-windows.zip" in url


def test_resolve_bun(monkeypatch):
    release = {
        "tag_name": "bun-v1.2.0",
        "assets": [
            {
                "name": "bun-windows-x64.zip",
                "browser_download_url": "https://example.com/bun-windows-x64.zip",
            },
            {
                "name": "bun-windows-x64-profile.zip",
                "browser_download_url": "https://example.com/bun-windows-x64-profile.zip",
            },
        ],
    }
    monkeypatch.setattr(bun_mod, "github_latest_release", lambda *a: release)
    monkeypatch.setattr(bun_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(bun_mod, "cpu_arch", lambda: "x64")
    url, version = bun_mod.resolve_bun_download()
    assert version == "bun-v1.2.0"
    assert url.endswith("bun-windows-x64.zip")
    assert "profile" not in url
