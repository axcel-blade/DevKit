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


def test_parse_jdk_feature():
    assert jdk_mod.parse_jdk_feature(None) == jdk_mod.JDK_FEATURE_VERSION
    assert jdk_mod.parse_jdk_feature("17") == 17
    assert jdk_mod.parse_jdk_feature("21.0.2") == 21
    assert jdk_mod.parse_jdk_feature("jdk-25") == 25
    try:
        jdk_mod.parse_jdk_feature("temurin")
        raised = False
    except RuntimeError:
        raised = True
    assert raised


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


def test_jdk_install_respects_version(monkeypatch, tmp_path: Path):
    plugin = JdkPlugin()
    ctx = InstallContext(install_dir=tmp_path / "jdk", home=tmp_path, version="17")
    seen: list[str] = []

    def fake_url(feature: int = 21) -> str:
        seen.append(str(feature))
        return f"https://example.com/jdk-{feature}.zip"

    monkeypatch.setattr(jdk_mod, "temurin_download_url", fake_url)

    def fake_install(url, dest, strip_top_level=True, progress=None, filename=None):
        dest = Path(dest)
        bin_dir = dest / "bin"
        bin_dir.mkdir(parents=True)
        name = "java.exe" if jdk_mod.is_windows() else "java"
        (bin_dir / name).write_text("java", encoding="utf-8")
        return dest

    monkeypatch.setattr(jdk_mod, "install_archive_from_url", fake_install)
    result = plugin.install(ctx)
    assert seen == ["17"]
    assert "17" in result.message
