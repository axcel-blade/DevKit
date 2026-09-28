//! uv plugin — latest archive from astral-sh/uv GitHub releases.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, github_latest_release, pick_release_asset};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const MARKER: &str = ".devkit-uv";

fn uv_asset_name(host: HostOS, arch: &str) -> Result<&'static str> {
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
    Ok(asset)
}

fn resolve_uv_download() -> Result<(String, String)> {
    let release = github_latest_release("astral-sh", "uv")?;
    let asset = uv_asset_name(current_os(), &cpu_arch())?;
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

    fn ctx(tmp: &std::path::Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("uv"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn asset_name_matches_host() {
        assert_eq!(
            uv_asset_name(HostOS::Windows, "x86_64").unwrap(),
            "uv-x86_64-pc-windows-msvc.zip"
        );
        assert_eq!(
            uv_asset_name(HostOS::Windows, "aarch64").unwrap(),
            "uv-aarch64-pc-windows-msvc.zip"
        );
        assert_eq!(
            uv_asset_name(HostOS::Linux, "x86_64").unwrap(),
            "uv-x86_64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            uv_asset_name(HostOS::Linux, "aarch64").unwrap(),
            "uv-aarch64-unknown-linux-gnu.tar.gz"
        );
        assert_eq!(
            uv_asset_name(HostOS::MacOS, "x86_64").unwrap(),
            "uv-x86_64-apple-darwin.tar.gz"
        );
        assert_eq!(
            uv_asset_name(HostOS::MacOS, "aarch64").unwrap(),
            "uv-aarch64-apple-darwin.tar.gz"
        );
        assert!(uv_asset_name(HostOS::Other, "x86_64").is_err());
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = UvPlugin;
        let c = ctx(tmp.path());
        assert_eq!(plugin.status(&c).state, InstallState::NotInstalled);
    }

    #[test]
    fn status_installed_when_binary_present() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = UvPlugin;
        let c = ctx(tmp.path());
        std::fs::create_dir_all(&c.install_dir).unwrap();
        let name = if is_windows() { "uv.exe" } else { "uv" };
        std::fs::write(c.install_dir.join(name), b"stub").unwrap();
        assert_eq!(plugin.status(&c).state, InstallState::Installed);
    }

    #[test]
    fn env_spec_points_at_install_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = UvPlugin;
        let c = ctx(tmp.path());
        std::fs::create_dir_all(&c.install_dir).unwrap();
        let spec = plugin.env_spec(&c);
        assert_eq!(spec.paths, vec![c.install_dir.clone()]);
        assert_eq!(spec.vars.len(), 1);
        assert_eq!(spec.vars[0].0, "UV_INSTALL_DIR");
    }

    #[test]
    fn uninstall_removes_install_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = UvPlugin;
        let c = ctx(tmp.path());
        std::fs::create_dir_all(&c.install_dir).unwrap();
        plugin.uninstall(&c).unwrap();
        assert!(!c.install_dir.exists());
    }
}
