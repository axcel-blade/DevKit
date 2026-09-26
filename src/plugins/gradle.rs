//! Gradle plugin — download the latest binary distribution into the machine `dev` folder.
//!
//! Uses the official Gradle current-version API (same ZIP for Windows, macOS, and Linux).
//! Sets `GRADLE_HOME` and prepends `bin` to PATH.

use crate::download::{download_json, install_archive_from_url};
use crate::platform::is_windows;
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const CURRENT_API: &str = "https://services.gradle.org/versions/current";
const MARKER: &str = ".devkit-gradle";

/// Return `(download_url, version)` for the current Gradle release.
fn resolve_gradle_download() -> Result<(String, String)> {
    let meta = download_json(CURRENT_API)?;
    let version = meta
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let url = meta
        .get("downloadUrl")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if version.is_empty() || url.is_empty() {
        bail!("Unexpected response from Gradle versions API");
    }
    if meta
        .get("broken")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        bail!("Gradle marks version {version} as broken; refusing to install");
    }
    Ok((url, version))
}

fn gradle_bin(ctx: &InstallContext) -> PathBuf {
    if is_windows() {
        ctx.install_dir.join("bin").join("gradle.bat")
    } else {
        ctx.install_dir.join("bin").join("gradle")
    }
}

pub struct GradlePlugin;

impl Plugin for GradlePlugin {
    fn id(&self) -> &'static str {
        "gradle"
    }

    fn name(&self) -> &'static str {
        "Gradle"
    }

    fn description(&self) -> &'static str {
        "Download latest Gradle binary ZIP and set GRADLE_HOME / PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &gradle_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but gradle binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_gradle_download()?;
        println!("Gradle {version}");
        println!("URL: {url}");
        // Archive root is gradle-<version>/; strip so bin/ lands in install_dir.
        let mut progress = download_progress("Downloading Gradle");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = gradle_bin(ctx);
        if !binary.is_file() {
            bail!(
                "Gradle extracted but binary not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Gradle {version} installed at {}",
                ctx.install_dir.display()
            ),
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
        Ok(Some(resolve_gradle_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.join("bin")],
            vars: vec![(
                "GRADLE_HOME".to_string(),
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
        let plugin = GradlePlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("gradle"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
