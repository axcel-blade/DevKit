//! MSYS2 plugin — portable base runtime from the official msys2-installer
//! GitHub releases (Windows only).

use crate::download::install_archive_from_url;
use crate::platform::{current_os, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, github_latest_release, pick_release_asset};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const MARKER: &str = ".devkit-msys2";

fn resolve_msys2_download() -> Result<(String, String)> {
    let release = github_latest_release("msys2", "msys2-installer")?;
    pick_release_asset(&release, &["base-x86_64", ".tar.xz"])
}

fn msys2_bash(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir.join("usr").join("bin").join("bash.exe")
}

fn msys2_shell(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir.join("msys2_shell.cmd")
}

pub struct Msys2Plugin;

impl Plugin for Msys2Plugin {
    fn id(&self) -> &'static str {
        "msys2"
    }

    fn name(&self) -> &'static str {
        "MSYS2"
    }

    fn description(&self) -> &'static str {
        "Download the MSYS2 base runtime (portable, Windows only) and add its \
         usr/bin (bash, pacman, etc.) to PATH. Run 'pacman -Syu' once after \
         install to bring package databases up to date."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &msys2_bash(ctx),
            &ctx.install_dir,
            "Install dir exists but usr/bin/bash.exe is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        if current_os() != HostOS::Windows {
            bail!("MSYS2 is only supported on Windows");
        }
        let (url, version) = resolve_msys2_download()?;
        println!("MSYS2 {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading MSYS2");
        // Archive contains a top-level msys64/ directory.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let bash = msys2_bash(ctx);
        if !bash.is_file() {
            bail!(
                "MSYS2 extracted but usr/bin/bash.exe not found under {}",
                ctx.install_dir.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        println!(
            "MSYS2 installed. Run 'pacman -Syu' from {} at least once to update packages.",
            msys2_shell(ctx).display()
        );
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("MSYS2 {version} installed at {}", ctx.install_dir.display()),
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
        Ok(Some(resolve_msys2_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.join("usr").join("bin")],
            vars: vec![(
                "MSYS2_HOME".to_string(),
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
        let plugin = Msys2Plugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("msys2"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
