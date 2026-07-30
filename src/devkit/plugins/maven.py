"""Maven plugin — latest Apache Maven binary ZIP (all platforms)."""

from __future__ import annotations

import shutil
from pathlib import Path

from devkit.download import install_archive_from_url
from devkit.platform import is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status, parse_maven_latest_version, read_text_url
from devkit.progress import download_progress

_LISTING = "https://downloads.apache.org/maven/maven-3/"
MARKER = ".devkit-maven"


def resolve_maven_download() -> tuple[str, str]:
    """Return ``(download_url, version)`` for the newest Maven 3.x binary ZIP."""
    html = read_text_url(_LISTING)
    version = parse_maven_latest_version(html)
    name = f"apache-maven-{version}-bin.zip"
    url = f"https://downloads.apache.org/maven/maven-3/{version}/binaries/{name}"
    return url, version


def _mvn_bin(ctx: InstallContext) -> Path:
    return ctx.install_dir / "bin" / ("mvn.cmd" if is_windows() else "mvn")


class MavenPlugin(Plugin):
    id = "maven"
    name = "Maven"
    description = "Download latest Apache Maven binary ZIP and set MAVEN_HOME / PATH."

    def status(self, ctx: InstallContext):
        return binary_status(
            _mvn_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but mvn binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url, version = resolve_maven_download()
        print(f"Maven {version}")
        print(f"URL: {url}")
        progress = download_progress("Downloading Maven")
        install_archive_from_url(url, ctx.install_dir, strip_top_level=True, progress=progress)
        progress.done()
        if not _mvn_bin(ctx).is_file():
            raise RuntimeError(f"Maven extracted but mvn not found at {_mvn_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text(version + "\n", encoding="utf-8")
        return InstallResult(ctx.install_dir, message=f"Maven {version} installed at {ctx.install_dir}")

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(
            paths=[ctx.install_dir / "bin"],
            vars={"MAVEN_HOME": str(ctx.install_dir.resolve())},
        )
