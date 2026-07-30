"""Tests for the Docker CLI plugin (network mocked)."""

from pathlib import Path

from devkit.platform import HostOS
from devkit.plugin import InstallContext, InstallState
from devkit.plugins import docker as docker_mod
from devkit.plugins.docker import DockerPlugin


def test_resolve_docker_download(monkeypatch):
    html = """
    <a href="docker-28.0.0.tgz">docker-28.0.0.tgz</a>
    <a href="docker-29.1.0.tgz">docker-29.1.0.tgz</a>
    <a href="docker-29.0.5.tgz">docker-29.0.5.tgz</a>
    """
    monkeypatch.setattr(docker_mod, "read_text_url", lambda url: html)
    monkeypatch.setattr(docker_mod, "current_os", lambda: HostOS.LINUX)
    monkeypatch.setattr(docker_mod, "cpu_arch", lambda: "x64")
    url, version = docker_mod.resolve_docker_download()
    assert version == "29.1.0"
    assert url.endswith("docker-29.1.0.tgz")
    assert "/linux/static/stable/x86_64/" in url


def test_resolve_docker_windows_zip(monkeypatch):
    html = '<a href="docker-29.6.2.zip">docker-29.6.2.zip</a>'
    monkeypatch.setattr(docker_mod, "read_text_url", lambda url: html)
    monkeypatch.setattr(docker_mod, "current_os", lambda: HostOS.WINDOWS)
    monkeypatch.setattr(docker_mod, "cpu_arch", lambda: "x64")
    url, version = docker_mod.resolve_docker_download()
    assert version == "29.6.2"
    assert url.endswith(".zip")


def test_docker_install(monkeypatch, tmp_path: Path):
    plugin = DockerPlugin()
    ctx = InstallContext(install_dir=tmp_path / "docker", home=tmp_path)
    monkeypatch.setattr(
        docker_mod,
        "resolve_docker_download",
        lambda: ("https://example.com/docker-29.0.0.tgz", "29.0.0"),
    )

    def fake_install(url, dest, strip_top_level=True, progress=None, filename=None):
        dest = Path(dest)
        dest.mkdir(parents=True, exist_ok=True)
        name = "docker.exe" if docker_mod.is_windows() else "docker"
        (dest / name).write_text("docker", encoding="utf-8")
        return dest

    monkeypatch.setattr(docker_mod, "install_archive_from_url", fake_install)
    assert plugin.status(ctx).state == InstallState.NOT_INSTALLED
    result = plugin.install(ctx)
    assert "29.0.0" in result.message
    assert plugin.status(ctx).state == InstallState.INSTALLED
    assert plugin.env_spec(ctx).vars["DOCKER_HOME"]
