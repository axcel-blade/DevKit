"""Tests for the Flutter plugin (network mocked)."""

from pathlib import Path

from devkit.plugin import InstallContext, InstallState
from devkit.plugins import flutter as flutter_mod
from devkit.plugins.flutter import FlutterPlugin


def _sample_meta() -> dict:
    return {
        "base_url": "https://example.com/releases",
        "current_release": {"stable": "abc", "beta": "bbb"},
        "releases": [
            {
                "hash": "abc",
                "channel": "stable",
                "version": "3.44.8",
                "archive": "stable/windows/flutter_windows_3.44.8-stable.zip",
            },
            {
                "hash": "abc",
                "channel": "stable",
                "version": "3.44.8",
                "archive": "stable/windows/flutter_windows_arm64_3.44.8-stable.zip",
            },
            {
                "hash": "old",
                "channel": "stable",
                "version": "3.24.5",
                "archive": "stable/windows/flutter_windows_3.24.5-stable.zip",
            },
            {
                "hash": "bbb",
                "channel": "beta",
                "version": "3.45.0-0.1.pre",
                "archive": "beta/windows/flutter_windows_3.45.0-0.1.pre-beta.zip",
            },
        ],
    }


def test_resolve_stable_flutter_url(monkeypatch):
    monkeypatch.setattr(flutter_mod, "download_json", lambda url: _sample_meta())
    monkeypatch.setattr(flutter_mod, "_releases_platform", lambda: "windows")
    monkeypatch.setattr(flutter_mod, "_arch_hints", lambda: ("x64", "amd64"))

    url, version = flutter_mod.resolve_stable_flutter_url()
    assert version == "3.44.8"
    assert url.endswith("flutter_windows_3.44.8-stable.zip")
    assert "arm64" not in url


def test_resolve_flutter_url_channel_and_version(monkeypatch):
    monkeypatch.setattr(flutter_mod, "download_json", lambda url: _sample_meta())
    monkeypatch.setattr(flutter_mod, "_releases_platform", lambda: "windows")
    monkeypatch.setattr(flutter_mod, "_arch_hints", lambda: ("x64", "amd64"))

    url, version = flutter_mod.resolve_flutter_url(channel="beta")
    assert version.startswith("3.45.0")
    assert "beta" in url

    url, version = flutter_mod.resolve_flutter_url(channel="stable", version="3.24")
    assert version == "3.24.5"
    assert "3.24.5" in url


def test_resolve_flutter_url_bad_channel():
    try:
        flutter_mod.resolve_flutter_url(channel="nightly")
        raised = False
    except RuntimeError as exc:
        raised = True
        assert "channel" in str(exc).lower()
    assert raised


def test_flutter_install_uninstall(monkeypatch, tmp_path: Path):
    plugin = FlutterPlugin()
    ctx = InstallContext(install_dir=tmp_path / "flutter", home=tmp_path)

    monkeypatch.setattr(
        flutter_mod,
        "resolve_flutter_url",
        lambda **kwargs: ("https://example.com/flutter.zip", "3.44.8"),
    )

    def fake_install(url, dest, strip_top_level=True, progress=None, filename=None):
        dest = Path(dest)
        bin_dir = dest / "bin"
        bin_dir.mkdir(parents=True)
        if flutter_mod.is_windows():
            (bin_dir / "flutter.bat").write_text("@echo off\n", encoding="utf-8")
        else:
            (bin_dir / "flutter").write_text("#!/bin/sh\n", encoding="utf-8")
        return dest

    monkeypatch.setattr(flutter_mod, "install_archive_from_url", fake_install)

    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
    result = plugin.install(ctx)
    assert "3.44.8" in result.message
    assert plugin.status(ctx).state == InstallState.INSTALLED

    spec = plugin.env_spec(ctx)
    assert ctx.install_dir / "bin" in spec.paths
    assert spec.vars["FLUTTER_ROOT"] == str(ctx.install_dir.resolve())

    plugin.uninstall(ctx)
    assert not ctx.install_dir.exists()


def test_flutter_install_respects_channel(monkeypatch, tmp_path: Path):
    plugin = FlutterPlugin()
    ctx = InstallContext(
        install_dir=tmp_path / "flutter",
        home=tmp_path,
        channel="beta",
        version="3.45.0-0.1.pre",
    )
    seen: dict = {}

    def fake_resolve(*, channel="stable", version=None):
        seen["channel"] = channel
        seen["version"] = version
        return ("https://example.com/flutter.zip", version or "x")

    monkeypatch.setattr(flutter_mod, "resolve_flutter_url", fake_resolve)

    def fake_install(url, dest, strip_top_level=True, progress=None, filename=None):
        dest = Path(dest)
        bin_dir = dest / "bin"
        bin_dir.mkdir(parents=True)
        name = "flutter.bat" if flutter_mod.is_windows() else "flutter"
        (bin_dir / name).write_text("x", encoding="utf-8")
        return dest

    monkeypatch.setattr(flutter_mod, "install_archive_from_url", fake_install)
    plugin.install(ctx)
    assert seen == {"channel": "beta", "version": "3.45.0-0.1.pre"}
