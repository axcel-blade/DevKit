"""Tests for the Android SDK cmdline-tools plugin (network mocked)."""

from pathlib import Path

from devkit.plugin import InstallContext, InstallState
from devkit.plugins import android as android_mod
from devkit.plugins.android import AndroidPlugin


def test_resolve_cmdline_tools_url(monkeypatch):
    from devkit.platform import HostOS

    monkeypatch.setattr(android_mod, "current_os", lambda: HostOS.WINDOWS)
    url, build = android_mod.resolve_cmdline_tools_url()
    assert build == android_mod.CMDLINE_TOOLS_BUILD
    assert "commandlinetools-win-" in url
    assert url.endswith(f"{build}_latest.zip")


def test_layout_cmdline_tools(tmp_path: Path):
    ctx = InstallContext(install_dir=tmp_path / "android", home=tmp_path)
    tools = ctx.install_dir / "cmdline-tools"
    tools.mkdir(parents=True)
    (tools / "bin").mkdir()
    name = "sdkmanager.bat" if android_mod.is_windows() else "sdkmanager"
    (tools / "bin" / name).write_text("sdk", encoding="utf-8")
    (tools / "lib").mkdir()

    android_mod._layout_cmdline_tools(ctx)
    assert android_mod._sdkmanager(ctx).is_file()
    assert not (tools / "bin").exists()
    assert (tools / "latest" / "bin" / name).is_file()


def test_android_install_uninstall(monkeypatch, tmp_path: Path):
    plugin = AndroidPlugin()
    ctx = InstallContext(install_dir=tmp_path / "android", home=tmp_path)

    monkeypatch.setattr(
        android_mod,
        "resolve_cmdline_tools_url",
        lambda: ("https://example.com/commandlinetools-win-1_latest.zip", "1"),
    )

    def fake_install(url, dest, strip_top_level=False, progress=None, filename=None):
        dest = Path(dest)
        tools = dest / "cmdline-tools"
        bin_dir = tools / "bin"
        bin_dir.mkdir(parents=True)
        name = "sdkmanager.bat" if android_mod.is_windows() else "sdkmanager"
        (bin_dir / name).write_text("sdk", encoding="utf-8")
        return dest

    monkeypatch.setattr(android_mod, "install_archive_from_url", fake_install)

    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
    result = plugin.install(ctx)
    assert "cmdline-tools" in result.message.lower() or "1" in result.message
    assert plugin.status(ctx).state == InstallState.INSTALLED

    spec = plugin.env_spec(ctx)
    home = str(ctx.install_dir.resolve())
    assert spec.vars["ANDROID_HOME"] == home
    assert spec.vars["ANDROID_SDK_ROOT"] == home
    assert ctx.install_dir / "cmdline-tools" / "latest" / "bin" in spec.paths

    plugin.uninstall(ctx)
    assert not ctx.install_dir.exists()
