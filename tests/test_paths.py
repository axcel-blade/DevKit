"""Unit tests for path helpers."""

from pathlib import Path

from devkit.paths import cache_dir, default_dev_root, home, plugin_install_dir, preferred_dev_roots
from devkit.platform import HostOS


def test_preferred_roots_windows(monkeypatch):
    monkeypatch.setattr("devkit.paths.current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setenv("SystemDrive", "C:")
    assert preferred_dev_roots() == [Path(r"C:\dev")]


def test_preferred_roots_macos(monkeypatch, tmp_path: Path):
    monkeypatch.setattr("devkit.paths.current_os", lambda: HostOS.MACOS)
    monkeypatch.setattr(Path, "home", classmethod(lambda cls: tmp_path))
    assert preferred_dev_roots() == [Path("/opt/dev"), tmp_path / "dev"]


def test_preferred_roots_linux(monkeypatch, tmp_path: Path):
    monkeypatch.setattr("devkit.paths.current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(Path, "home", classmethod(lambda cls: tmp_path))
    assert preferred_dev_roots() == [Path("/opt/dev"), tmp_path / "dev"]


def test_default_dev_root_falls_back_to_home(monkeypatch, tmp_path: Path):
    monkeypatch.setattr("devkit.paths.current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(Path, "home", classmethod(lambda cls: tmp_path))
    # /opt/dev is typically not creatable in tests; expect ~/dev
    monkeypatch.setattr(
        "devkit.paths._is_creatable",
        lambda path: path == tmp_path / "dev",
    )
    assert default_dev_root() == tmp_path / "dev"


def test_home_override(monkeypatch, tmp_path: Path):
    custom = tmp_path / "custom-dev"
    custom.mkdir()
    monkeypatch.setenv("DEVKIT_HOME", str(custom))
    assert home() == custom.resolve()
    assert plugin_install_dir("flutter") == (custom / "flutter").resolve()
    assert cache_dir() == (custom / ".cache").resolve()
