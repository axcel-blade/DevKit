"""Tests for the MySQL plugin (network mocked)."""

from pathlib import Path

from devkit.platform import HostOS
from devkit.plugin import InstallContext, InstallState
from devkit.plugins import mysql as mysql_mod
from devkit.plugins.mysql import MysqlPlugin


def test_resolve_mysql_url_windows(monkeypatch):
    monkeypatch.setattr(mysql_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(mysql_mod, "cpu_arch", lambda: "x64")
    url, version = mysql_mod.resolve_mysql_url()
    assert version == "8.4.10"
    assert url.endswith("mysql-8.4.10-winx64.zip")


def test_resolve_mysql_url_linux_arm(monkeypatch):
    monkeypatch.setattr(mysql_mod, "current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(mysql_mod, "cpu_arch", lambda: "aarch64")
    url, _ = mysql_mod.resolve_mysql_url()
    assert "linux-glibc2.28-aarch64.tar.xz" in url


def test_resolve_mysql_url_macos(monkeypatch):
    monkeypatch.setattr(mysql_mod, "current_os", lambda: HostOS.MACOS)
    monkeypatch.setattr(mysql_mod, "cpu_arch", lambda: "aarch64")
    url, _ = mysql_mod.resolve_mysql_url()
    assert "macos15-arm64.tar.gz" in url


def test_mysql_install_uninstall(monkeypatch, tmp_path: Path):
    plugin = MysqlPlugin()
    ctx = InstallContext(install_dir=tmp_path / "mysql", home=tmp_path)
    monkeypatch.setattr(
        mysql_mod,
        "resolve_mysql_url",
        lambda: ("https://example.com/mysql.zip", "8.4.10"),
    )

    def fake_install(url, dest, strip_top_level=True, progress=None, filename=None):
        dest = Path(dest)
        bin_dir = dest / "bin"
        bin_dir.mkdir(parents=True)
        name = "mysql.exe" if mysql_mod.is_windows() else "mysql"
        (bin_dir / name).write_text("mysql", encoding="utf-8")
        return dest

    monkeypatch.setattr(mysql_mod, "install_archive_from_url", fake_install)

    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
    result = plugin.install(ctx)
    assert "8.4.10" in result.message
    assert plugin.status(ctx).state == InstallState.INSTALLED
    spec = plugin.env_spec(ctx)
    assert spec.vars["MYSQL_HOME"] == str(ctx.install_dir.resolve())
    assert ctx.install_dir / "bin" in spec.paths

    plugin.uninstall(ctx)
    assert not ctx.install_dir.exists()
