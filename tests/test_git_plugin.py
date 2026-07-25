"""Tests for the Git plugin (network mocked)."""

from pathlib import Path

from devkit.platform import HostOS
from devkit.plugin import InstallContext, InstallState
from devkit.plugins import git as git_mod
from devkit.plugins.git import GitPlugin


def test_resolve_mingit_url(monkeypatch):
    meta = {
        "tag_name": "v2.55.0.windows.3",
        "assets": [
            {
                "name": "MinGit-2.55.0.3-busybox-64-bit.zip",
                "browser_download_url": "https://example.com/busybox.zip",
            },
            {
                "name": "MinGit-2.55.0.3-64-bit.zip",
                "browser_download_url": "https://example.com/mingit64.zip",
            },
            {
                "name": "MinGit-2.55.0.3-arm64.zip",
                "browser_download_url": "https://example.com/mingit-arm.zip",
            },
        ],
    }
    monkeypatch.setattr(git_mod, "download_json", lambda url: meta)
    monkeypatch.setattr(git_mod, "cpu_arch", lambda: "x64")
    url, version = git_mod.resolve_mingit_url()
    assert version == "v2.55.0.windows.3"
    assert url.endswith("mingit64.zip")


def test_git_windows_install(monkeypatch, tmp_path: Path):
    plugin = GitPlugin()
    ctx = InstallContext(install_dir=tmp_path / "git", home=tmp_path)

    monkeypatch.setattr(git_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(git_mod, "is_windows", lambda: True)
    monkeypatch.setattr(
        git_mod,
        "resolve_mingit_url",
        lambda: ("https://example.com/MinGit.zip", "v2.55.0.windows.3"),
    )

    def fake_install(url, dest, strip_top_level=False, progress=None, filename=None):
        dest = Path(dest)
        cmd = dest / "cmd"
        cmd.mkdir(parents=True)
        (cmd / "git.exe").write_text("git", encoding="utf-8")
        return dest

    monkeypatch.setattr(git_mod, "install_archive_from_url", fake_install)

    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
    result = plugin.install(ctx)
    assert "MinGit" in result.message
    assert plugin.status(ctx).state == InstallState.INSTALLED
    spec = plugin.env_spec(ctx)
    assert ctx.install_dir / "cmd" in spec.paths
    assert spec.vars["GIT_HOME"] == str(ctx.install_dir.resolve())

    plugin.uninstall(ctx)
    assert not ctx.install_dir.exists()


def test_git_unix_requires_system_git(monkeypatch, tmp_path: Path):
    plugin = GitPlugin()
    ctx = InstallContext(install_dir=tmp_path / "git", home=tmp_path)
    monkeypatch.setattr(git_mod, "current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(git_mod.shutil, "which", lambda name: None)

    try:
        plugin.install(ctx)
        raised = False
    except RuntimeError as exc:
        raised = True
        assert "apt install git" in str(exc)
    assert raised


def test_git_unix_registers_wrappers(monkeypatch, tmp_path: Path):
    plugin = GitPlugin()
    ctx = InstallContext(install_dir=tmp_path / "git", home=tmp_path)
    system_git = tmp_path / "usr" / "bin" / "git"
    system_git.parent.mkdir(parents=True)
    system_git.write_text("#!/bin/sh\n", encoding="utf-8")

    monkeypatch.setattr(git_mod, "current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(git_mod, "is_windows", lambda: False)
    monkeypatch.setattr(
        git_mod.shutil,
        "which",
        lambda name: str(system_git) if name == "git" else None,
    )

    result = plugin.install(ctx)
    assert "Registered system Git" in result.message
    assert (ctx.install_dir / "bin" / "git").is_file()
    assert plugin.status(ctx).state == InstallState.INSTALLED
