"""Tests for the Node.js plugin (network mocked)."""

from pathlib import Path

from devkit.plugin import InstallContext, InstallState
from devkit.plugins import node as node_mod
from devkit.plugins.node import NodePlugin


def test_resolve_node_lts_download(monkeypatch):
    index = [
        {
            "version": "v24.18.0",
            "lts": "Krypton",
            "files": ["win-x64-zip", "linux-x64", "osx-arm64-tar", "linux-arm64"],
        },
        {
            "version": "v22.0.0",
            "lts": False,
            "files": ["win-x64-zip", "linux-x64"],
        },
    ]
    monkeypatch.setattr(node_mod, "download_json_value", lambda url: index)
    monkeypatch.setattr(node_mod, "_archive_file_key", lambda: "linux-x64")
    url, version = node_mod.resolve_node_lts_download()
    assert version == "v24.18.0"
    assert url.endswith("/v24.18.0/node-v24.18.0-linux-x64.tar.xz")


def test_resolve_node_lts_missing_file(monkeypatch):
    monkeypatch.setattr(
        node_mod,
        "download_json_value",
        lambda url: [{"version": "v24.0.0", "lts": "Krypton", "files": ["linux-x64"]}],
    )
    monkeypatch.setattr(node_mod, "_archive_file_key", lambda: "win-arm64-zip")
    try:
        node_mod.resolve_node_lts_download()
        raised = False
    except RuntimeError as exc:
        raised = True
        assert "win-arm64-zip" in str(exc)
    assert raised


def test_node_install_uninstall(monkeypatch, tmp_path: Path):
    plugin = NodePlugin()
    ctx = InstallContext(install_dir=tmp_path / "node", home=tmp_path)

    monkeypatch.setattr(
        node_mod,
        "resolve_node_lts_download",
        lambda: ("https://example.com/node-v24.18.0-linux-x64.tar.xz", "v24.18.0"),
    )

    def fake_install(url, dest, strip_top_level=True, progress=None, filename=None):
        dest = Path(dest)
        if node_mod.is_windows():
            (dest).mkdir(parents=True, exist_ok=True)
            (dest / "node.exe").write_text("node", encoding="utf-8")
            (dest / "npm.cmd").write_text("npm", encoding="utf-8")
        else:
            bin_dir = dest / "bin"
            bin_dir.mkdir(parents=True)
            (bin_dir / "node").write_text("node", encoding="utf-8")
            (bin_dir / "npm").write_text("npm", encoding="utf-8")
        return dest

    monkeypatch.setattr(node_mod, "install_archive_from_url", fake_install)

    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
    result = plugin.install(ctx)
    assert "v24.18.0" in result.message
    assert plugin.status(ctx).state == InstallState.INSTALLED

    spec = plugin.env_spec(ctx)
    assert spec.vars["NODE_HOME"] == str(ctx.install_dir.resolve())
    if node_mod.is_windows():
        assert ctx.install_dir in spec.paths
    else:
        assert ctx.install_dir / "bin" in spec.paths

    plugin.uninstall(ctx)
    assert not ctx.install_dir.exists()
