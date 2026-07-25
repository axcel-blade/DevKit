"""Tests for HostOS helpers."""

from pathlib import Path

from devkit.platform import HostOS, current_os, pick_for_os, primary_shell_profile


def test_current_os_windows(monkeypatch):
    monkeypatch.setattr("devkit.platform.platform.system", lambda: "Windows")
    assert current_os() == HostOS.WINDOWS


def test_current_os_macos(monkeypatch):
    monkeypatch.setattr("devkit.platform.platform.system", lambda: "Darwin")
    assert current_os() == HostOS.MACOS


def test_current_os_linux(monkeypatch):
    monkeypatch.setattr("devkit.platform.platform.system", lambda: "Linux")
    assert current_os() == HostOS.LINUX


def test_pick_for_os(monkeypatch):
    urls = {
        "windows": "win.zip",
        "macos": "mac.zip",
        "linux": "linux.tar.xz",
    }
    monkeypatch.setattr("devkit.platform.current_os", lambda: HostOS.WINDOWS)
    assert pick_for_os(urls) == "win.zip"
    monkeypatch.setattr("devkit.platform.current_os", lambda: HostOS.MACOS)
    assert pick_for_os(urls) == "mac.zip"
    monkeypatch.setattr("devkit.platform.current_os", lambda: HostOS.LINUX)
    assert pick_for_os(urls) == "linux.tar.xz"


def test_pick_for_os_darwin_alias(monkeypatch):
    monkeypatch.setattr("devkit.platform.current_os", lambda: HostOS.MACOS)
    assert pick_for_os({"darwin": "mac.zip"}) == "mac.zip"


def test_primary_shell_profile_macos(monkeypatch, tmp_path: Path):
    monkeypatch.setattr("devkit.platform.current_os", lambda: HostOS.MACOS)
    monkeypatch.setattr(Path, "home", classmethod(lambda cls: tmp_path))
    monkeypatch.setenv("SHELL", "/bin/zsh")
    assert primary_shell_profile() == tmp_path / ".zshrc"


def test_primary_shell_profile_linux_bash(monkeypatch, tmp_path: Path):
    monkeypatch.setattr("devkit.platform.current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(Path, "home", classmethod(lambda cls: tmp_path))
    monkeypatch.setenv("SHELL", "/bin/bash")
    assert primary_shell_profile() == tmp_path / ".bashrc"
