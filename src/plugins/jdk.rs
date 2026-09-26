//! Eclipse Temurin JDK plugin (Adoptium).

use crate::download::{download_json_value, install_archive_from_url};
use crate::platform::{adoptium_os, cpu_arch, is_windows};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

/// Default LTS line when `install jdk` is run without `--version`.
const JDK_FEATURE_VERSION: u32 = 25;

/// Parse a feature version from `--version` (`25` or `25.0.1` -> 25).
fn parse_jdk_feature(version: Option<&str>) -> Result<u32> {
    let version = match version {
        None => return Ok(JDK_FEATURE_VERSION),
        Some(v) if v.trim().is_empty() => return Ok(JDK_FEATURE_VERSION),
        Some(v) => v,
    };
    let mut raw = version.trim().to_lowercase();
    if raw.starts_with("jdk") {
        raw = raw[3..].trim_start_matches('-').to_string();
    }
    let major = raw.split('.').next().unwrap_or("");
    if major.is_empty() || !major.chars().all(|c| c.is_ascii_digit()) {
        bail!(
            "Invalid JDK version '{}'. Use a feature number like 21 or 25.",
            version
        );
    }
    let feature: u32 = major.parse().unwrap_or(0);
    if feature < 8 {
        bail!("Unsupported JDK feature version: {feature}");
    }
    Ok(feature)
}

/// Stable Adoptium URL for the latest GA Temurin JDK on this OS/arch.
fn temurin_download_url(feature: u32) -> Result<String> {
    let os_slug = adoptium_os()?;
    let arch = cpu_arch();
    if !["x64", "aarch64", "x86"].contains(&arch.as_str()) {
        bail!("Unsupported CPU arch for Temurin JDK: {arch}");
    }
    Ok(format!(
        "https://api.adoptium.net/v3/binary/latest/{feature}/ga/{os_slug}/{arch}/jdk/hotspot/normal/eclipse"
    ))
}

/// Extract the value of `JAVA_VERSION="..."` from a JDK `release` file.
fn parse_release_java_version(text: &str) -> Option<String> {
    text.lines()
        .find_map(|l| l.strip_prefix("JAVA_VERSION="))
        .map(|v| v.trim().trim_matches('"').to_string())
        .filter(|v| !v.is_empty())
}

fn java_bin(ctx: &InstallContext) -> PathBuf {
    if is_windows() {
        ctx.install_dir.join("bin").join("java.exe")
    } else {
        ctx.install_dir.join("bin").join("java")
    }
}

pub struct JdkPlugin;

impl Plugin for JdkPlugin {
    fn id(&self) -> &'static str {
        "jdk"
    }

    fn name(&self) -> &'static str {
        "JDK"
    }

    fn description(&self) -> &'static str {
        "Download Eclipse Temurin JDK (default 25 LTS; optional --version for feature line) \
         and set JAVA_HOME / PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        let java = java_bin(ctx);
        if java.is_file() {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail(java.display().to_string());
        }
        let has_contents = ctx.install_dir.is_dir()
            && std::fs::read_dir(&ctx.install_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_contents {
            return PluginStatus::new(InstallState::Partial, Some(ctx.install_dir.clone()))
                .with_detail("Install dir exists but java binary is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let feature = parse_jdk_feature(ctx.version.as_deref())?;
        let url = temurin_download_url(feature)?;
        println!("Temurin JDK {feature}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading JDK");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let java = java_bin(ctx);
        if !java.is_file() {
            bail!("JDK extracted but java not found at {}", java.display());
        }
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Temurin JDK {feature} installed at {}",
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

    /// Read `JAVA_VERSION="25.0.1"` from the JDK's `release` file.
    fn installed_version(&self, ctx: &InstallContext) -> Option<String> {
        let text = std::fs::read_to_string(ctx.install_dir.join("release")).ok()?;
        parse_release_java_version(&text)
    }

    /// Latest GA Temurin build for the installed feature line (or the default
    /// LTS line when nothing is installed), as `major.minor.security`.
    fn latest_version(&self, ctx: &InstallContext) -> Result<Option<String>> {
        let feature = self
            .installed_version(ctx)
            .and_then(|v| v.split('.').next().and_then(|m| m.parse::<u32>().ok()))
            .filter(|f| *f >= 9)
            .unwrap_or(JDK_FEATURE_VERSION);
        let url = format!(
            "https://api.adoptium.net/v3/assets/latest/{feature}/hotspot?architecture={}&image_type=jdk&os={}&vendor=eclipse",
            cpu_arch(),
            adoptium_os()?
        );
        let data = download_json_value(&url)?;
        let ver = data.get(0).and_then(|e| e.get("version"));
        let part = |k: &str| ver.and_then(|v| v.get(k)).and_then(|v| v.as_u64());
        Ok(match (part("major"), part("minor"), part("security")) {
            (Some(a), Some(b), Some(c)) => Some(format!("{a}.{b}.{c}")),
            _ => None,
        })
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.join("bin")],
            vars: vec![(
                "JAVA_HOME".to_string(),
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

    #[test]
    fn parse_jdk_feature_defaults_when_none() {
        assert_eq!(parse_jdk_feature(None).unwrap(), JDK_FEATURE_VERSION);
        assert_eq!(parse_jdk_feature(Some("  ")).unwrap(), JDK_FEATURE_VERSION);
    }

    #[test]
    fn parse_jdk_feature_parses_major() {
        assert_eq!(parse_jdk_feature(Some("21.0.2")).unwrap(), 21);
        assert_eq!(parse_jdk_feature(Some("17")).unwrap(), 17);
        assert_eq!(parse_jdk_feature(Some("jdk-17")).unwrap(), 17);
        assert_eq!(parse_jdk_feature(Some("JDK17")).unwrap(), 17);
    }

    #[test]
    fn parse_jdk_feature_rejects_invalid() {
        assert!(parse_jdk_feature(Some("abc")).is_err());
        assert!(parse_jdk_feature(Some("7")).is_err());
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = JdkPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("jdk"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
