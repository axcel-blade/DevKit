//! QEMU plugin — official Windows NSIS installer run silently into a
//! private folder; system QEMU registered as-is on macOS/Linux.

use crate::platform::{current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use anyhow::{bail, Result};
use std::path::PathBuf;
// Only the Windows install path downloads and runs the NSIS installer;
// gate these imports the same way installers.rs gates its Windows/macOS
// extractors, so `-D warnings` doesn't fail unused-import on other OSes.
#[cfg(windows)]
use crate::download::download_file;
#[cfg(windows)]
use crate::progress::download_progress;
#[cfg(windows)]
use anyhow::Context;

const MARKER: &str = ".devkit-qemu";
const QEMU_VERSION: &str = "8.2.2";
const QEMU_WINDOWS_URL: &str = "https://qemu.weilnetz.de/w64/2024/qemu-w64-setup-20240423.exe";

fn qemu_windows_binary(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir.join("qemu-system-x86_64.exe")
}

#[cfg(windows)]
fn install_windows(ctx: &InstallContext) -> Result<InstallResult> {
    use std::process::Command;

    println!("Downloading QEMU {QEMU_VERSION} (Windows x64 installer) ...");
    let mut progress = download_progress("Downloading QEMU");
    let installer = download_file(
        QEMU_WINDOWS_URL,
        None,
        Some(&format!("qemu-w64-setup-{QEMU_VERSION}.exe")),
        Some(&mut |d, t| progress.update(d, t)),
    )?;
    progress.done();

    if ctx.install_dir.exists() {
        std::fs::remove_dir_all(&ctx.install_dir)?;
    }
    std::fs::create_dir_all(&ctx.install_dir)?;

    println!(
        "Running silent NSIS install into {} ...",
        ctx.install_dir.display()
    );
    let dest = crate::paths::to_absolute(&ctx.install_dir);
    // NSIS silent install: /S suppresses the wizard, /D sets the target
    // directory and must be the last argument, unquoted, with no trailing slash.
    let dir_arg = format!("/D={}", dest.display().to_string().trim_end_matches('\\'));
    let status = Command::new(&installer)
        .arg("/S")
        .arg(dir_arg)
        .status()
        .context("failed to launch QEMU installer")?;
    if !status.success() {
        bail!("QEMU installer exited with {:?}", status.code());
    }

    let binary = qemu_windows_binary(ctx);
    if !binary.is_file() {
        bail!(
            "QEMU installed but qemu-system-x86_64.exe not found under {}",
            ctx.install_dir.display()
        );
    }
    std::fs::write(ctx.install_dir.join(MARKER), format!("{QEMU_VERSION}\n"))?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!(
            "QEMU {QEMU_VERSION} installed at {}",
            ctx.install_dir.display()
        ),
    ))
}

#[cfg(not(windows))]
fn install_windows(_ctx: &InstallContext) -> Result<InstallResult> {
    bail!("Windows QEMU install path invoked on a non-Windows OS")
}

fn install_unix(ctx: &InstallContext, label: &str, pkg_hint: &str) -> Result<InstallResult> {
    let system_qemu = which::which("qemu-system-x86_64")
        .or_else(|_| which::which("qemu-img"))
        .map_err(|_| {
            anyhow::anyhow!(
                "QEMU does not ship a portable {label} archive for DevKit.\n\
                 Install it with your package manager, then re-run this command:\n\
                 {pkg_hint}\n\
                 Then:  devkit install qemu"
            )
        })?;
    std::fs::create_dir_all(&ctx.install_dir)?;
    std::fs::write(
        ctx.install_dir.join(MARKER),
        format!("system-wrapper\n{}\n", system_qemu.display()),
    )?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!("Registered system QEMU at {}", system_qemu.display()),
    ))
}

pub struct QemuPlugin;

impl Plugin for QemuPlugin {
    fn id(&self) -> &'static str {
        "qemu"
    }

    fn name(&self) -> &'static str {
        "QEMU"
    }

    fn description(&self) -> &'static str {
        "Install QEMU: silent NSIS install (Windows) or register an \
         already-installed system QEMU (macOS/Linux via Homebrew/apt/dnf/pacman)."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        if is_windows() {
            let binary = qemu_windows_binary(ctx);
            if binary.is_file() {
                return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                    .with_detail(binary.display().to_string());
            }
        } else if ctx.install_dir.join(MARKER).is_file() {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail("system qemu wrapper");
        }
        let has_contents = ctx.install_dir.is_dir()
            && std::fs::read_dir(&ctx.install_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_contents {
            return PluginStatus::new(InstallState::Partial, Some(ctx.install_dir.clone()))
                .with_detail("Install dir exists but qemu binary is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        match current_os() {
            HostOS::Windows => install_windows(ctx),
            HostOS::MacOS => install_unix(ctx, "macOS", "  brew install qemu"),
            HostOS::Linux => install_unix(
                ctx,
                "Linux",
                "  Debian/Ubuntu:  sudo apt install qemu-system\n  \
                 Fedora:         sudo dnf install qemu\n  \
                 Arch:           sudo pacman -S qemu-full",
            ),
            HostOS::Other => bail!("QEMU is not supported on this OS: other"),
        }
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let mut paths = Vec::new();
        if is_windows() && qemu_windows_binary(ctx).is_file() {
            paths.push(ctx.install_dir.clone());
        }
        EnvSpec {
            paths,
            vars: vec![(
                "QEMU_HOME".to_string(),
                ctx.install_dir
                    .canonicalize()
                    .unwrap_or_else(|_| ctx.install_dir.clone())
                    .display()
                    .to_string(),
            )],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = QemuPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("qemu"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
