//! Bun plugin — latest ZIP from oven-sh/bun GitHub releases.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{
    binary_status, find_files_named, github_latest_release, pick_release_asset,
};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const MARKER: &str = ".devkit-bun";

fn resolve_bun_download() -> Result<(String, String)> {
    let release = github_latest_release("oven-sh", "bun")?;
    let host = current_os();
    let arch = cpu_arch();
    let needle = match host {
        HostOS::Windows => {
            if arch == "aarch64" {
                "bun-windows-aarch64.zip"
            } else {
                "bun-windows-x64.zip"
            }
        }
        HostOS::Linux => {
            if arch == "aarch64" {
                "bun-linux-aarch64.zip"
            } else {
                "bun-linux-x64.zip"
            }
        }
        HostOS::MacOS => {
            if arch == "aarch64" {
                "bun-darwin-aarch64.zip"
            } else {
                "bun-darwin-x64.zip"
            }
        }
        HostOS::Other => bail!("Bun is not supported on this OS: other"),
    };

    // Prefer exact asset name (skip -profile / -baseline variants).
    if let Some(assets) = release.get("assets").and_then(|v| v.as_array()) {
        for asset in assets {
            if asset.get("name").and_then(|v| v.as_str()) == Some(needle) {
                let url = asset
                    .get("browser_download_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tag = release
                    .get("tag_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                if !url.is_empty() {
                    return Ok((url, tag));
                }
            }
        }
    }
    let stem = needle.trim_end_matches(".zip");
    pick_release_asset(&release, &[stem, ".zip"])
}

/// ZIP often contains bun-<plat>/bun(.exe); after strip or nested search.
fn bun_bin(ctx: &InstallContext) -> PathBuf {
    let name = if is_windows() { "bun.exe" } else { "bun" };
    let direct = ctx.install_dir.join(name);
    if direct.is_file() {
        return direct;
    }
    let matches = find_files_named(&ctx.install_dir, name);
    if !matches.is_empty() {
        return matches.into_iter().next().unwrap();
    }
    direct
}

/// If archive nested bun under a subfolder, flatten binary to install_dir.
fn layout_bun(ctx: &InstallContext) -> Result<()> {
    let binary = bun_bin(ctx);
    if let Some(file_name) = binary.file_name() {
        let target = ctx.install_dir.join(file_name);
        if binary.is_file() && binary != target {
            std::fs::copy(&binary, &target)?;
        }
    }
    Ok(())
}

pub struct BunPlugin;

impl Plugin for BunPlugin {
    fn id(&self) -> &'static str {
        "bun"
    }

    fn name(&self) -> &'static str {
        "Bun"
    }

    fn description(&self) -> &'static str {
        "Download latest Bun runtime ZIP and add it to PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &bun_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but bun binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_bun_download()?;
        println!("Bun {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Bun");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        layout_bun(ctx)?;
        let binary = bun_bin(ctx);
        if !binary.is_file() {
            bail!(
                "Bun extracted but binary not found under {}",
                ctx.install_dir.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("Bun {version} installed at {}", ctx.install_dir.display()),
        ))
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let binary = bun_bin(ctx);
        let path_entry = if binary.is_file() {
            binary.parent().unwrap_or(&ctx.install_dir).to_path_buf()
        } else {
            ctx.install_dir.clone()
        };
        EnvSpec {
            paths: vec![path_entry],
            vars: vec![(
                "BUN_INSTALL".to_string(),
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
        let plugin = BunPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("bun"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }

    #[test]
    fn bun_bin_finds_nested_binary() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = InstallContext {
            install_dir: tmp.path().join("bun"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        let name = if is_windows() { "bun.exe" } else { "bun" };
        let nested = ctx.install_dir.join("nested").join(name);
        std::fs::create_dir_all(nested.parent().unwrap()).unwrap();
        std::fs::write(&nested, b"stub").unwrap();
        assert_eq!(bun_bin(&ctx), nested);
    }
}
