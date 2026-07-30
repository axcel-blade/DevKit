"""Terraform plugin — latest ZIP from HashiCorp releases."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import download_json, install_archive_from_url
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status
from devkit.progress import download_progress

_INDEX = "https://releases.hashicorp.com/terraform/index.json"
MARKER = ".devkit-terraform"


def resolve_terraform_download() -> tuple[str, str]:
    meta = download_json(_INDEX)
    versions = meta.get("versions") or {}
    if not isinstance(versions, dict) or not versions:
        raise RuntimeError("Unexpected Terraform releases index")
    # Prefer newest non-prerelease version key.
    def is_release(v: str) -> bool:
        return all(p.isdigit() for p in v.split("."))

    release_versions = [v for v in versions if is_release(v)]
    if not release_versions:
        raise RuntimeError("No stable Terraform versions found")
    version = max(release_versions, key=lambda v: tuple(int(p) for p in v.split(".")))
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        goos = "windows"
    elif host == HostOS.LINUX:
        goos = "linux"
    elif host == HostOS.MACOS:
        goos = "darwin"
    else:
        raise RuntimeError(f"Terraform is not supported on this OS: {host.value}")
    goarch = "arm64" if arch == "aarch64" else "amd64"
    builds = (versions[version] or {}).get("builds") or []
    for build in builds:
        if not isinstance(build, dict):
            continue
        if build.get("os") == goos and build.get("arch") == goarch:
            url = str(build.get("url") or "")
            if url:
                return url, version
    raise RuntimeError(f"No Terraform build for {goos}/{goarch} version {version}")


def _terraform_bin(ctx: InstallContext) -> Path:
    return ctx.install_dir / ("terraform.exe" if is_windows() else "terraform")


class TerraformPlugin(Plugin):
    id = "terraform"
    name = "Terraform"
    description = "Download latest Terraform ZIP and add it to PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _terraform_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but terraform binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_terraform_download()
        print(f"Terraform {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading Terraform")
        install_archive_from_url(url, ctx.install_dir, strip_top_level=False, progress=progress)
        progress.done()
        if not _terraform_bin(ctx).is_file():
            raise RuntimeError(f"Terraform extracted but binary not found at {_terraform_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(
            ctx.install_dir,
            message=f"Terraform {version} installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(
            paths=[ctx.install_dir],
            vars={"TERRAFORM_HOME": str(ctx.install_dir.resolve())},
        )
