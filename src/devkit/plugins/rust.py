"""Rust plugin — install stable toolchain via rustup into the machine ``dev`` folder."""

from __future__ import annotations

import os
import shutil
import stat
import subprocess
from pathlib import Path

from devkit.download import download_file
from devkit.platform import HostOS, cpu_arch, current_os, is_windows
from devkit.plugin import EnvSpec, InstallContext, InstallResult, Plugin
from devkit.plugin_utils import binary_status
from devkit.progress import download_progress

MARKER = ".devkit-rust"


def _rust_host_triple() -> str:
    host = current_os()
    arch = cpu_arch()
    if host == HostOS.WINDOWS:
        cpu = "aarch64" if arch == "aarch64" else "x86_64"
        return f"{cpu}-pc-windows-msvc"
    if host == HostOS.LINUX:
        cpu = "aarch64" if arch == "aarch64" else "x86_64"
        return f"{cpu}-unknown-linux-gnu"
    if host == HostOS.MACOS:
        cpu = "aarch64" if arch == "aarch64" else "x86_64"
        return f"{cpu}-apple-darwin"
    raise RuntimeError(f"Rust is not supported on this OS: {host.value}")


def resolve_rustup_init_url() -> str:
    triple = _rust_host_triple()
    name = "rustup-init.exe" if is_windows() else "rustup-init"
    return f"https://static.rust-lang.org/rustup/dist/{triple}/{name}"


def _cargo_bin(ctx: InstallContext) -> Path:
    return ctx.install_dir / "cargo" / "bin" / ("cargo.exe" if is_windows() else "cargo")


class RustPlugin(Plugin):
    id = "rust"
    name = "Rust"
    description = (
        "Install Rust stable via rustup into the machine dev folder "
        "(sets CARGO_HOME / RUSTUP_HOME / PATH)."
    )

    def status(self, ctx: InstallContext):
        return binary_status(
            _cargo_bin(ctx),
            ctx.install_dir,
            missing_detail="Install dir exists but cargo binary is missing",
        )

    def install(self, ctx: InstallContext) -> InstallResult:
        url = resolve_rustup_init_url()
        print("Rust (rustup stable)")
        print(f"URL: {url}")
        progress = download_progress("Downloading rustup-init")
        init = download_file(
            url,
            filename="rustup-init.exe" if is_windows() else "rustup-init",
            progress=progress,
        )
        progress.done()
        if not is_windows():
            mode = init.stat().st_mode
            init.chmod(mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)

        cargo_home = ctx.install_dir / "cargo"
        rustup_home = ctx.install_dir / "rustup"
        ctx.install_dir.mkdir(parents=True, exist_ok=True)
        env = os.environ.copy()
        env["CARGO_HOME"] = str(cargo_home)
        env["RUSTUP_HOME"] = str(rustup_home)
        # Non-interactive default toolchain; do not touch user PATH (DevKit owns env).
        cmd = [
            str(init),
            "-y",
            "--no-modify-path",
            "--default-toolchain",
            "stable",
            "--profile",
            "default",
        ]
        print("Running rustup-init ...")
        result = subprocess.run(cmd, env=env, capture_output=True, text=True, check=False)
        if result.returncode != 0:
            raise RuntimeError(
                "rustup-init failed "
                f"(exit {result.returncode}): {result.stderr or result.stdout}"
            )
        if not _cargo_bin(ctx).is_file():
            raise RuntimeError(f"rustup finished but cargo not found at {_cargo_bin(ctx)}")
        (ctx.install_dir / MARKER).write_text("stable\n", encoding="utf-8")
        return InstallResult(
            ctx.install_dir,
            message=f"Rust stable installed at {ctx.install_dir}",
        )

    def uninstall(self, ctx: InstallContext) -> None:
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        cargo_home = ctx.install_dir / "cargo"
        rustup_home = ctx.install_dir / "rustup"
        return EnvSpec(
            paths=[cargo_home / "bin"],
            vars={
                "CARGO_HOME": str(cargo_home.resolve()),
                "RUSTUP_HOME": str(rustup_home.resolve()),
            },
        )
