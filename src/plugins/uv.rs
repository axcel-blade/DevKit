//! uv plugin — latest archive from astral-sh/uv GitHub releases.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, github_latest_release, pick_release_asset};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const MARKER: &str = ".devkit-uv";

fn resolve_uv_download() -> Result<(String, String)> {
    let release = github_latest_release("astral-sh", "uv")?;
    let host = current_os();
    let arch = cpu_arch();
    let asset = match host {
        HostOS::Windows => {
            if arch == "aarch64" {
                "uv-aarch64-pc-windows-msvc.zip"
            } else {
                "uv-x86_64-pc-windows-msvc.zip"
            }
        }
        HostOS::Linux => {
            if arch == "aarch64" {
                "uv-aarch64-unknown-linux-gnu.tar.gz"
            } else {
                "uv-x86_64-unknown-linux-gnu.tar.gz"
            }
        }
        HostOS::MacOS => {
            if arch == "aarch64" {
                "uv-aarch64-apple-darwin.tar.gz"
            } else {
                "uv-x86_64-apple-darwin.tar.gz"
            }
        }
        HostOS::Other => bail!("uv is not supported on this OS: other"),
    };
    pick_release_asset(&release, &[asset])
}

fn uv_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join(if is_windows() { "uv.exe" } else { "uv" })
}

pub struct UvPlugin;

impl Plugin for UvPlugin {
    fn id(&self) -> &'static str {
        "uv"
    }

    fn name(&self) -> &'static str {
        "uv"
    }

    fn description(&self) -> &'static str {
        "Download latest uv (and uvx) and add them to PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &uv_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but uv binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_uv_download()?;
        println!("uv {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading uv");
        // Release archives place uv and uvx at the archive root.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            false,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = uv_bin(ctx);
        if !binary.is_file() {
            bail!(
                "uv extracted but binary not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("uv {version} installed at {}", ctx.install_dir.display()),
        ))
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    /// Same resolver `install` uses, so the string matches the marker it writes.
    fn latest_version(&self, _ctx: &InstallContext) -> Result<Option<String>> {
        Ok(Some(resolve_uv_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.clone()],
            vars: vec![(
                "UV_INSTALL_DIR".to_string(),
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
        let plugin = UvPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("uv"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
