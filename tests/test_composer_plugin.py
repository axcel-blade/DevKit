"""Tests for the Composer plugin (network mocked)."""

from pathlib import Path

from devkit.plugin import InstallContext, InstallState
from devkit.plugins import composer as composer_mod
from devkit.plugins.composer import ComposerPlugin


def test_composer_install_uninstall(monkeypatch, tmp_path: Path):
    plugin = ComposerPlugin()
    ctx = InstallContext(install_dir=tmp_path / "composer", home=tmp_path)

    monkeypatch.setattr(composer_mod, "_php_available", lambda: True)

    def fake_download(url, dest=None, filename=None, progress=None):
        target = Path(dest) if dest is not None else tmp_path / (filename or "composer.phar")
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(b"PHAR")
        return target

    monkeypatch.setattr(composer_mod, "download_file", fake_download)

    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
    result = plugin.install(ctx)
    assert "Composer installed" in result.message
    assert (ctx.install_dir / "composer.phar").is_file()
    assert plugin.status(ctx).state == InstallState.INSTALLED

    if composer_mod.is_windows():
        assert (ctx.install_dir / "composer.bat").is_file()
    else:
        assert (ctx.install_dir / "composer").is_file()

    spec = plugin.env_spec(ctx)
    assert ctx.install_dir in spec.paths
    assert "COMPOSER_HOME" in spec.vars

    plugin.uninstall(ctx)
    assert not ctx.install_dir.exists()
