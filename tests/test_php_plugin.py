"""Tests for the PHP plugin (network mocked)."""

from pathlib import Path

from devkit.platform import HostOS
from devkit.plugin import InstallContext, InstallState
from devkit.plugins import php as php_mod
from devkit.plugins.php import PhpPlugin


def test_resolve_php_windows_url(monkeypatch):
    meta = {
        "8.3": {
            "version": "8.3.32",
            "nts-vs16-x64": {
                "zip": {"path": "php-8.3.32-nts-Win32-vs16-x64.zip"},
            },
        },
        "8.4": {
            "version": "8.4.23",
            "nts-vs17-x64": {
                "zip": {"path": "php-8.4.23-nts-Win32-vs17-x64.zip"},
            },
        },
    }
    monkeypatch.setattr(php_mod, "download_json", lambda url: meta)
    url, version = php_mod.resolve_php_windows_url()
    assert version == "8.4.23"
    assert url.endswith("php-8.4.23-nts-Win32-vs17-x64.zip")


def test_php_windows_install(monkeypatch, tmp_path: Path):
    plugin = PhpPlugin()
    ctx = InstallContext(install_dir=tmp_path / "php", home=tmp_path)
    monkeypatch.setattr(php_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(php_mod, "is_windows", lambda: True)
    monkeypatch.setattr(
        php_mod,
        "resolve_php_windows_url",
        lambda: ("https://example.com/php.zip", "8.4.23"),
    )

    def fake_install(url, dest, strip_top_level=False, progress=None, filename=None):
        dest = Path(dest)
        dest.mkdir(parents=True, exist_ok=True)
        (dest / "php.exe").write_text("php", encoding="utf-8")
        (dest / "php.ini-development").write_text("; ini\n", encoding="utf-8")
        return dest

    monkeypatch.setattr(php_mod, "install_archive_from_url", fake_install)

    result = plugin.install(ctx)
    assert "8.4.23" in result.message
    assert (ctx.install_dir / "php.ini").is_file()
    assert plugin.status(ctx).state == InstallState.INSTALLED
    assert php_mod.PhpPlugin().env_spec(ctx).vars["PHP_HOME"] == str(ctx.install_dir.resolve())


def test_php_unix_requires_system(monkeypatch, tmp_path: Path):
    plugin = PhpPlugin()
    ctx = InstallContext(install_dir=tmp_path / "php", home=tmp_path)
    monkeypatch.setattr(php_mod, "current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(php_mod.shutil, "which", lambda name: None)
    try:
        plugin.install(ctx)
        raised = False
    except RuntimeError as exc:
        raised = True
        assert "apt install php-cli" in str(exc)
    assert raised
