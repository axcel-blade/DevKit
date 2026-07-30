"""Additional plugin resolve/install tests (dotnet, terraform, deno, pnpm, rust helpers)."""

from pathlib import Path

from devkit.platform import HostOS
from devkit.plugin import InstallContext, InstallState
from devkit.plugins import deno as deno_mod
from devkit.plugins import dotnet as dotnet_mod
from devkit.plugins import pnpm as pnpm_mod
from devkit.plugins import rust as rust_mod
from devkit.plugins import terraform as tf_mod
from devkit.plugins.deno import DenoPlugin
from devkit.plugins.dotnet import DotnetPlugin


def test_resolve_dotnet(monkeypatch):
    meta = {
        "releases": [
            {
                "sdk": {
                    "version": "8.0.100",
                    "files": [
                        {
                            "name": "dotnet-sdk-8.0.100-win-x64.zip",
                            "url": "https://example.com/sdk.zip",
                        }
                    ],
                }
            }
        ]
    }
    monkeypatch.setattr(dotnet_mod, "download_json", lambda url: meta)
    monkeypatch.setattr(dotnet_mod, "_rid", lambda: "win-x64")
    url, version = dotnet_mod.resolve_dotnet_download()
    assert version == "8.0.100"
    assert url.endswith("sdk.zip")


def test_dotnet_install(monkeypatch, tmp_path: Path):
    plugin = DotnetPlugin()
    ctx = InstallContext(install_dir=tmp_path / "dotnet", home=tmp_path)
    monkeypatch.setattr(
        dotnet_mod, "resolve_dotnet_download", lambda: ("https://ex/sdk.zip", "8.0.100")
    )

    def fake_install(url, dest, strip_top_level=False, progress=None, filename=None):
        dest = Path(dest)
        dest.mkdir(parents=True, exist_ok=True)
        name = "dotnet.exe" if dotnet_mod.is_windows() else "dotnet"
        (dest / name).write_text("dotnet", encoding="utf-8")
        return dest

    monkeypatch.setattr(dotnet_mod, "install_archive_from_url", fake_install)
    result = plugin.install(ctx)
    assert "8.0.100" in result.message
    assert plugin.status(ctx).state == InstallState.INSTALLED
    assert plugin.env_spec(ctx).vars["DOTNET_ROOT"]


def test_resolve_terraform(monkeypatch):
    meta = {
        "versions": {
            "1.9.0": {
                "builds": [
                    {
                        "os": "windows",
                        "arch": "amd64",
                        "url": "https://example.com/terraform_1.9.0_windows_amd64.zip",
                    }
                ]
            },
            "1.10.0-rc1": {"builds": []},
            "1.8.5": {"builds": []},
        }
    }
    monkeypatch.setattr(tf_mod, "download_json", lambda url: meta)
    monkeypatch.setattr(tf_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(tf_mod, "cpu_arch", lambda: "x64")
    url, version = tf_mod.resolve_terraform_download()
    assert version == "1.9.0"
    assert "terraform_1.9.0" in url


def test_resolve_deno(monkeypatch):
    release = {
        "tag_name": "v2.0.0",
        "assets": [
            {
                "name": "deno-x86_64-pc-windows-msvc.zip",
                "browser_download_url": "https://example.com/deno.zip",
            }
        ],
    }
    monkeypatch.setattr(deno_mod, "github_latest_release", lambda *a: release)
    monkeypatch.setattr(deno_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(deno_mod, "cpu_arch", lambda: "x64")
    url, version = deno_mod.resolve_deno_download()
    assert version == "v2.0.0"
    assert url.endswith("deno.zip")


def test_deno_install(monkeypatch, tmp_path: Path):
    plugin = DenoPlugin()
    ctx = InstallContext(install_dir=tmp_path / "deno", home=tmp_path)
    monkeypatch.setattr(deno_mod, "resolve_deno_download", lambda: ("https://ex/deno.zip", "v2.0.0"))

    def fake_install(url, dest, strip_top_level=False, progress=None, filename=None):
        dest = Path(dest)
        dest.mkdir(parents=True, exist_ok=True)
        name = "deno.exe" if deno_mod.is_windows() else "deno"
        (dest / name).write_text("deno", encoding="utf-8")
        return dest

    monkeypatch.setattr(deno_mod, "install_archive_from_url", fake_install)
    assert "v2.0.0" in plugin.install(ctx).message


def test_resolve_pnpm(monkeypatch):
    release = {
        "tag_name": "v9.0.0",
        "assets": [
            {
                "name": "pnpm-win32-x64.zip",
                "browser_download_url": "https://example.com/pnpm-win32-x64.zip",
            }
        ],
    }
    monkeypatch.setattr(pnpm_mod, "github_latest_release", lambda *a: release)
    monkeypatch.setattr(pnpm_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(pnpm_mod, "cpu_arch", lambda: "x64")
    url, version = pnpm_mod.resolve_pnpm_download()
    assert version == "v9.0.0"
    assert url.endswith("pnpm-win32-x64.zip")


def test_rustup_init_url(monkeypatch):
    monkeypatch.setattr(rust_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(rust_mod, "cpu_arch", lambda: "x64")
    monkeypatch.setattr(rust_mod, "is_windows", lambda: True)
    url = rust_mod.resolve_rustup_init_url()
    assert "x86_64-pc-windows-msvc" in url
    assert url.endswith("rustup-init.exe")
