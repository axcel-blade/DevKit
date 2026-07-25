"""Tests for the Gradle plugin (network mocked)."""

from pathlib import Path

from devkit.plugin import InstallContext, InstallState
from devkit.plugins import gradle as gradle_mod
from devkit.plugins.gradle import GradlePlugin


def test_resolve_gradle_download(monkeypatch):
    meta = {
        "version": "9.6.1",
        "downloadUrl": "https://services.gradle.org/distributions/gradle-9.6.1-bin.zip",
        "broken": False,
    }
    monkeypatch.setattr(gradle_mod, "download_json", lambda url: meta)
    url, version = gradle_mod.resolve_gradle_download()
    assert version == "9.6.1"
    assert url.endswith("gradle-9.6.1-bin.zip")


def test_resolve_gradle_rejects_broken(monkeypatch):
    monkeypatch.setattr(
        gradle_mod,
        "download_json",
        lambda url: {
            "version": "9.0.0",
            "downloadUrl": "https://example.com/gradle.zip",
            "broken": True,
        },
    )
    try:
        gradle_mod.resolve_gradle_download()
        raised = False
    except RuntimeError as exc:
        raised = True
        assert "broken" in str(exc).lower()
    assert raised


def test_gradle_install_uninstall(monkeypatch, tmp_path: Path):
    plugin = GradlePlugin()
    ctx = InstallContext(install_dir=tmp_path / "gradle", home=tmp_path)

    monkeypatch.setattr(
        gradle_mod,
        "resolve_gradle_download",
        lambda: ("https://example.com/gradle-9.6.1-bin.zip", "9.6.1"),
    )

    def fake_install(url, dest, strip_top_level=True, progress=None, filename=None):
        dest = Path(dest)
        bin_dir = dest / "bin"
        bin_dir.mkdir(parents=True)
        name = "gradle.bat" if gradle_mod.is_windows() else "gradle"
        (bin_dir / name).write_text("gradle", encoding="utf-8")
        return dest

    monkeypatch.setattr(gradle_mod, "install_archive_from_url", fake_install)

    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
    result = plugin.install(ctx)
    assert "9.6.1" in result.message
    assert plugin.status(ctx).state == InstallState.INSTALLED

    spec = plugin.env_spec(ctx)
    assert ctx.install_dir / "bin" in spec.paths
    assert spec.vars["GRADLE_HOME"] == str(ctx.install_dir.resolve())

    plugin.uninstall(ctx)
    assert not ctx.install_dir.exists()
