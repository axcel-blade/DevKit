//! Maven plugin — latest Apache Maven binary ZIP (all platforms).

use crate::download::install_archive_from_url;
use crate::platform::is_windows;
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, parse_maven_latest_version, read_text_url};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const LISTING: &str = "https://downloads.apache.org/maven/maven-3/";
const MARKER: &str = ".devkit-maven";

/// Return `(download_url, version)` for the newest Maven 3.x binary ZIP.
///
/// Apache doesn't publish a releases API for Maven, so this scrapes the
/// Apache mirror's directory-listing HTML instead (see
/// `plugin_utils::parse_maven_latest_version`).
fn resolve_maven_download() -> Result<(String, String)> {
    let html = read_text_url(LISTING)?;
    let version = parse_maven_latest_version(&html)?;
    let name = format!("apache-maven-{version}-bin.zip");
    let url = format!("https://downloads.apache.org/maven/maven-3/{version}/binaries/{name}");
    Ok((url, version))
}

fn mvn_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join("bin")
        .join(if is_windows() { "mvn.cmd" } else { "mvn" })
}

pub struct MavenPlugin;

impl Plugin for MavenPlugin {
    fn id(&self) -> &'static str {
        "maven"
    }

    fn name(&self) -> &'static str {
        "Maven"
    }

    fn description(&self) -> &'static str {
        "Download latest Apache Maven binary ZIP and set MAVEN_HOME / PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &mvn_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but mvn binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_maven_download()?;
        println!("Maven {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Maven");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = mvn_bin(ctx);
        if !binary.is_file() {
            bail!("Maven extracted but mvn not found at {}", binary.display());
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("Maven {version} installed at {}", ctx.install_dir.display()),
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
        Ok(Some(resolve_maven_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.join("bin")],
            vars: vec![(
                "MAVEN_HOME".to_string(),
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
        let plugin = MavenPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("maven"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
