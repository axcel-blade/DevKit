//! pnpm plugin — standalone binary from pnpm GitHub releases.

use crate::download::{download_file, install_archive_from_url};
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, find_files_named, github_latest_release};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

const MARKER: &str = ".devkit-pnpm";

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Return `(download_url, version_tag)` for the latest pnpm standalone build.
fn resolve_pnpm_download() -> Result<(String, String)> {
    let release = github_latest_release("pnpm", "pnpm")?;
    let tag = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let host = current_os();
    let arch = cpu_arch();
    let name = match host {
        HostOS::Windows => {
            if arch == "aarch64" {
                "pnpm-win32-arm64.zip"
            } else {
                "pnpm-win32-x64.zip"
            }
        }
        HostOS::Linux => {
            if arch == "aarch64" {
                "pnpm-linux-arm64.tar.gz"
            } else {
                "pnpm-linux-x64.tar.gz"
            }
        }
        HostOS::MacOS => {
            if arch == "aarch64" {
                "pnpm-darwin-arm64.tar.gz"
            } else {
                "pnpm-darwin-x64.tar.gz"
            }
        }
        HostOS::Other => bail!("pnpm is not supported on this OS: other"),
    };
    if let Some(assets) = release.get("assets").and_then(|v| v.as_array()) {
        for asset in assets {
            if asset.get("name").and_then(|v| v.as_str()) == Some(name) {
                let url = asset
                    .get("browser_download_url")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                if !url.is_empty() {
                    return Ok((url.to_string(), tag));
                }
            }
        }
    }
    bail!("No pnpm asset named {name} in {tag}")
}

fn pnpm_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join(if is_windows() { "pnpm.exe" } else { "pnpm" })
}

pub struct PnpmPlugin;

impl Plugin for PnpmPlugin {
    fn id(&self) -> &'static str {
        "pnpm"
    }

    fn name(&self) -> &'static str {
        "pnpm"
    }

    fn description(&self) -> &'static str {
        "Download latest pnpm standalone binary and add it to PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &pnpm_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but pnpm binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_pnpm_download()?;
        println!("pnpm {version}");
        println!("URL: {url}");
        std::fs::create_dir_all(&ctx.install_dir)?;
        let mut progress = download_progress("Downloading pnpm");

        if url.ends_with(".zip") || url.ends_with(".tar.gz") {
            install_archive_from_url(
                &url,
                &ctx.install_dir,
                false,
                None,
                Some(&mut |d, t| progress.update(d, t)),
            )?;
            progress.done();
            // Normalize binary name to pnpm(.exe) at install root when nested.
            let name = if is_windows() { "pnpm.exe" } else { "pnpm" };
            let matches = find_files_named(&ctx.install_dir, name);
            if matches.is_empty() {
                bail!("pnpm archive extracted but {name} not found");
            }
            let target = ctx.install_dir.join(name);
            if matches[0] != target {
                std::fs::copy(&matches[0], &target)?;
            }
        } else {
            let dest = pnpm_bin(ctx);
            download_file(
                &url,
                Some(&dest),
                None,
                Some(&mut |d, t| progress.update(d, t)),
            )?;
            progress.done();
            if !is_windows() {
                make_executable(&dest)?;
            }
        }

        if !pnpm_bin(ctx).is_file() {
            bail!("pnpm binary missing at {}", pnpm_bin(ctx).display());
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("pnpm {version} installed at {}", ctx.install_dir.display()),
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
                "PNPM_HOME".to_string(),
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

    fn ctx(tmp: &Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("pnpm"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = PnpmPlugin;
        let c = ctx(tmp.path());
        assert_eq!(plugin.status(&c).state, InstallState::NotInstalled);
    }
}
