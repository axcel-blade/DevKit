"""Tests for the JDK plugin (network mocked)."""

from pathlib import Path

from devkit.plugin import InstallContext, InstallState
from devkit.plugins import jdk as jdk_mod
from devkit.plugins.jdk import JdkPlugin


def test_temurin_download_url(monkeypatch):
    monkeypatch.setattr(jdk_mod, "adoptium_os", lambda: "windows")
    monkeypatch.setattr(jdk_mod, "cpu_arch", lambda: "x64")
    url = jdk_mod.temurin_download_url(21)
    assert "api.adoptium.net" in url
    assert "/21/ga/windows/x64/jdk/" in url


def test_jdk_install_uninstall(monkeypatch, tmp_path: Path):
    plugin = JdkPlugin()
    ctx = InstallContext(install_dir=tmp_path / "jdk", home=tmp_path)

    def fake_install(url, dest, strip_top_level=True, progress=None, filename=None):
        dest = Path(dest)
        bin_dir = dest / "bin"
        bin_dir.mkdir(parents=True)
        name = "java.exe" if jdk_mod.is_windows() else "java"
        (bin_dir / name).write_text("java", encoding="utf-8")
        return dest

    monkeypatch.setattr(jdk_mod, "install_archive_from_url", fake_install)

    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
    result = plugin.install(ctx)
    assert "JDK" in result.message
    assert plugin.status(ctx).state == InstallState.INSTALLED
    spec = plugin.env_spec(ctx)
    assert spec.vars["JAVA_HOME"] == str(ctx.install_dir.resolve())
    assert ctx.install_dir / "bin" in spec.paths

    plugin.uninstall(ctx)
    assert not ctx.install_dir.exists()
