//! Deno plugin — latest ZIP from denoland/deno GitHub releases.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, github_latest_release, pick_release_asset};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const MARKER: &str = ".devkit-deno";

fn resolve_deno_download() -> Result<(String, String)> {
    let release = github_latest_release("denoland", "deno")?;
    let host = current_os();
    let arch = cpu_arch();
    match host {
        HostOS::Windows => {
            if arch == "aarch64" {
                pick_release_asset(&release, &["deno-aarch64-pc-windows-msvc.zip"])
            } else {
                pick_release_asset(&release, &["deno-x86_64-pc-windows-msvc.zip"])
            }
        }
        HostOS::Linux => {
            if arch == "aarch64" {
                pick_release_asset(&release, &["deno-aarch64-unknown-linux-gnu.zip"])
            } else {
                pick_release_asset(&release, &["deno-x86_64-unknown-linux-gnu.zip"])
            }
        }
        HostOS::MacOS => {
            if arch == "aarch64" {
                pick_release_asset(&release, &["deno-aarch64-apple-darwin.zip"])
            } else {
                pick_release_asset(&release, &["deno-x86_64-apple-darwin.zip"])
            }
        }
        HostOS::Other => bail!("Deno is not supported on this OS: other"),
    }
}

fn deno_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join(if is_windows() { "deno.exe" } else { "deno" })
}

pub struct DenoPlugin;

impl Plugin for DenoPlugin {
    fn id(&self) -> &'static str {
        "deno"
    }

    fn name(&self) -> &'static str {
        "Deno"
    }

    fn description(&self) -> &'static str {
        "Download latest Deno runtime ZIP and add it to PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &deno_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but deno binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_deno_download()?;
        println!("Deno {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Deno");
        // Deno's release ZIP contains the binary at the archive root (no
        // wrapper folder), so there is nothing to strip.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            false,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = deno_bin(ctx);
        if !binary.is_file() {
            bail!(
                "Deno extracted but binary not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("Deno {version} installed at {}", ctx.install_dir.display()),
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
        Ok(Some(resolve_deno_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.clone()],
            vars: vec![(
                "DENO_INSTALL".to_string(),
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
        let plugin = DenoPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("deno"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
