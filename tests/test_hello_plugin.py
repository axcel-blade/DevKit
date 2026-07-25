"""Tests for the hello stub plugin."""

from pathlib import Path

from devkit.plugin import InstallContext, InstallState
from devkit.plugins.hello import BIN_NAME, MARKER, HelloPlugin


def test_hello_install_uninstall(tmp_path: Path):
    plugin = HelloPlugin()
    ctx = InstallContext(install_dir=tmp_path / "hello", home=tmp_path)

    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED

    result = plugin.install(ctx)
    assert result.install_dir == ctx.install_dir
    assert (ctx.install_dir / MARKER).is_file()
    assert (ctx.install_dir / BIN_NAME / "hello.txt").is_file()
    assert (tmp_path / ".cache" / "hello-sdk.zip").is_file()
    assert plugin.status(ctx).state == InstallState.INSTALLED

    spec = plugin.env_spec(ctx)
    assert ctx.install_dir / BIN_NAME in spec.paths
    assert spec.vars["DEVKIT_HELLO_ROOT"] == str(ctx.install_dir.resolve())

    plugin.uninstall(ctx)
    assert not ctx.install_dir.exists()
    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
