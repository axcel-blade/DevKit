"""Tests for the Mono plugin (network / msiexec mocked)."""

from pathlib import Path

from devkit.platform import HostOS
from devkit.plugin import InstallContext, InstallState
from devkit.plugins import mono as mono_mod
from devkit.plugins.mono import MonoPlugin


def test_mono_windows_install(monkeypatch, tmp_path: Path):
    plugin = MonoPlugin()
    ctx = InstallContext(install_dir=tmp_path / "mono", home=tmp_path)

    monkeypatch.setattr(mono_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(mono_mod, "is_windows", lambda: True)

    def fake_download(url, dest=None, filename=None, progress=None):
        path = tmp_path / (filename or "mono.msi")
        path.write_bytes(b"msi")
        return path

    def fake_extract(msi, dest):
        dest = Path(dest)
        bin_dir = dest / "bin"
        bin_dir.mkdir(parents=True)
        (bin_dir / "mono.exe").write_text("mono", encoding="utf-8")
        return dest

    monkeypatch.setattr(mono_mod, "download_file", fake_download)
    monkeypatch.setattr(mono_mod, "extract_msi_admin", fake_extract)

    result = plugin.install(ctx)
    assert "Mono" in result.message
    assert plugin.status(ctx).state == InstallState.INSTALLED
    spec = plugin.env_spec(ctx)
    assert "MONO_HOME" in spec.vars
    assert any(p.name == "bin" for p in spec.paths)

    plugin.uninstall(ctx)
    assert not ctx.install_dir.exists()


def test_mono_linux_requires_system_mono(monkeypatch, tmp_path: Path):
    plugin = MonoPlugin()
    ctx = InstallContext(install_dir=tmp_path / "mono", home=tmp_path)
    monkeypatch.setattr(mono_mod, "current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(mono_mod.shutil, "which", lambda name: None)

    try:
        plugin.install(ctx)
        raised = False
    except RuntimeError as exc:
        raised = True
        assert "apt install" in str(exc)
    assert raised


def test_mono_linux_registers_wrappers(monkeypatch, tmp_path: Path):
    plugin = MonoPlugin()
    ctx = InstallContext(install_dir=tmp_path / "mono", home=tmp_path)
    system_mono = tmp_path / "usr" / "bin" / "mono"
    system_mono.parent.mkdir(parents=True)
    system_mono.write_text("#!/bin/sh\n", encoding="utf-8")

    monkeypatch.setattr(mono_mod, "current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(
        mono_mod.shutil,
        "which",
        lambda name: str(system_mono) if name == "mono" else None,
    )

    result = plugin.install(ctx)
    assert "Registered system Mono" in result.message
    assert (ctx.install_dir / "bin" / "mono").is_file()
    assert plugin.status(ctx).state == InstallState.INSTALLED
