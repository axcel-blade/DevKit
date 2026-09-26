//! PMD plugin — PMD Source Code Analyzer binary ZIP (all platforms).
//!
//! Downloads the official `pmd-dist-*-bin.zip` (or older `pmd-bin-*.zip`)
//! from GitHub releases. Needs Java to run; pair with the `jdk` plugin.

use crate::download::install_archive_from_url;
use crate::platform::is_windows;
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{
    binary_status, github_latest_release, github_release_by_tag, pick_release_asset,
};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use serde_json::Value;
use std::path::PathBuf;

const MARKER: &str = ".devkit-pmd";

/// Pick the binary distribution ZIP from a PMD GitHub release payload.
fn pick_pmd_bin_asset(release: &Value) -> Result<(String, String)> {
    pick_release_asset(release, &["pmd-dist", "-bin.zip"])
        .or_else(|_| pick_release_asset(release, &["pmd-bin-", ".zip"]))
}

/// Return `(download_url, version_label)` for latest or `--version`.
fn resolve_pmd_download(requested: Option<&str>) -> Result<(String, String)> {
    let release = if let Some(raw) = requested {
        let version = raw.trim().trim_start_matches('v');
        if version.is_empty() {
            bail!("Invalid PMD version. Use a release like 7.26.0.");
        }
        github_release_by_tag("pmd", "pmd", &format!("pmd_releases/{version}"))?
    } else {
        github_latest_release("pmd", "pmd")?
    };
    let (url, tag) = pick_pmd_bin_asset(&release)?;
    let version = tag
        .rsplit('/')
        .next()
        .unwrap_or(tag.as_str())
        .trim_start_matches('v')
        .to_string();
    Ok((url, version))
}

fn pmd_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join("bin")
        .join(if is_windows() { "pmd.bat" } else { "pmd" })
}

fn java_available() -> bool {
    which::which("java").is_ok()
}

pub struct PmdPlugin;

impl Plugin for PmdPlugin {
    fn id(&self) -> &'static str {
        "pmd"
    }

    fn name(&self) -> &'static str {
        "PMD"
    }

    fn description(&self) -> &'static str {
        "Download PMD Source Code Analyzer binary ZIP from GitHub \
         (optional --version) and set PMD_HOME / PATH. Needs Java; use the jdk plugin."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        let mut status = binary_status(
            &pmd_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but pmd launcher is missing",
        );
        if status.state == crate::plugin::InstallState::Installed && !java_available() {
            status.detail.push_str(" (warning: java not found on PATH)");
        }
        status
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        if !java_available() {
            println!(
                "Warning: `java` was not found on PATH. \
                 Install the jdk plugin (or another JDK) to run PMD."
            );
        }
        let (url, version) = resolve_pmd_download(ctx.version.as_deref())?;
        println!("PMD {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading PMD");
        // Archive root is pmd-bin-<ver>/; strip so bin/ lands in install_dir.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = pmd_bin(ctx);
        if !binary.is_file() {
            bail!(
                "PMD extracted but launcher not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("PMD {version} installed at {}", ctx.install_dir.display()),
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
        Ok(Some(resolve_pmd_download(None)?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.join("bin")],
            vars: vec![(
                "PMD_HOME".to_string(),
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
    use serde_json::json;

    #[test]
    fn pick_pmd_bin_prefers_dist_bin_zip() {
        let release = json!({
            "tag_name": "pmd_releases/7.26.0",
            "html_url": "https://github.com/pmd/pmd/releases/tag/pmd_releases/7.26.0",
            "assets": [
                {
                    "name": "pmd-dist-7.26.0-src.zip",
                    "browser_download_url": "https://example.com/src.zip"
                },
                {
                    "name": "pmd-dist-7.26.0-bin.zip",
                    "browser_download_url": "https://example.com/pmd-dist-7.26.0-bin.zip"
                }
            ]
        });
        let (url, tag) = pick_pmd_bin_asset(&release).unwrap();
        assert_eq!(url, "https://example.com/pmd-dist-7.26.0-bin.zip");
        assert_eq!(tag, "pmd_releases/7.26.0");
    }

    #[test]
    fn pick_pmd_bin_falls_back_to_legacy_name() {
        let release = json!({
            "tag_name": "pmd_releases/6.55.0",
            "html_url": "https://github.com/pmd/pmd/releases",
            "assets": [
                {
                    "name": "pmd-bin-6.55.0.zip",
                    "browser_download_url": "https://example.com/pmd-bin-6.55.0.zip"
                }
            ]
        });
        let (url, _) = pick_pmd_bin_asset(&release).unwrap();
        assert!(url.ends_with("pmd-bin-6.55.0.zip"));
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = PmdPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("pmd"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
