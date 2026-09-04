//! Ninja plugin — latest single-binary ZIP from ninja-build GitHub releases.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, github_latest_release, pick_release_asset};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const MARKER: &str = ".devkit-ninja";

fn resolve_ninja_download() -> Result<(String, String)> {
    let release = github_latest_release("ninja-build", "ninja")?;
    let host = current_os();
    let arch = cpu_arch();
    match host {
        HostOS::Windows => {
            if arch == "aarch64" {
                pick_release_asset(&release, &["ninja-winarm64.zip"])
            } else {
                pick_release_asset(&release, &["ninja-win.zip"])
            }
        }
        HostOS::Linux => {
            if arch == "aarch64" {
                pick_release_asset(&release, &["ninja-linux-aarch64.zip"])
            } else {
                pick_release_asset(&release, &["ninja-linux.zip"])
            }
        }
        HostOS::MacOS => pick_release_asset(&release, &["ninja-mac.zip"]),
        HostOS::Other => bail!("Ninja is not supported on this OS: other"),
    }
}

fn ninja_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join(if is_windows() { "ninja.exe" } else { "ninja" })
}

pub struct NinjaPlugin;

impl Plugin for NinjaPlugin {
    fn id(&self) -> &'static str {
        "ninja"
    }

    fn name(&self) -> &'static str {
        "Ninja"
    }

    fn description(&self) -> &'static str {
        "Download latest Ninja build tool binary and add it to PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &ninja_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but ninja binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_ninja_download()?;
        println!("Ninja {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Ninja");
        // ZIP contains ninja(.exe) at the archive root.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            false,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = ninja_bin(ctx);
        if !binary.is_file() {
            bail!(
                "Ninja extracted but binary not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("Ninja {version} installed at {}", ctx.install_dir.display()),
        ))
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.clone()],
            vars: vec![(
                "NINJA_HOME".to_string(),
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
    use crate::plugin::InstallState;

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = NinjaPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("ninja"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
