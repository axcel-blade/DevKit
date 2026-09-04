//! Flutter SDK plugin — download a channel/version into the machine `dev` folder.

use crate::download::{download_json, install_archive_from_url};
use crate::platform::{current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const RELEASES_BASE: &str = "https://storage.googleapis.com/flutter_infra_release/releases";
const FLUTTER_CHANNELS: [&str; 3] = ["stable", "beta", "dev"];

fn releases_platform() -> Result<&'static str> {
    match current_os() {
        HostOS::Windows => Ok("windows"),
        HostOS::MacOS => Ok("macos"),
        HostOS::Linux => Ok("linux"),
        HostOS::Other => bail!("Flutter is not supported on this OS: other"),
    }
}

fn arch_hints() -> Vec<&'static str> {
    let machine = std::env::consts::ARCH.to_lowercase();
    match machine.as_str() {
        "arm64" | "aarch64" => vec!["arm64", "aarch64"],
        "x86_64" | "amd64" | "x64" => vec!["x64", "x86_64", "amd64"],
        _ => vec![],
    }
}

/// Higher score = better match for this machine (prefer arch-specific builds).
fn score_archive(archive: &str) -> i32 {
    let name = archive.to_lowercase();
    let hints = arch_hints();
    let mut score = 0;
    for &hint in &hints {
        if name.contains(hint) {
            score += 10;
        }
    }
    // Prefer non-arm builds when we are on x64 (avoid accidental arm archives).
    if !hints.contains(&"arm64") && name.contains("arm64") {
        score -= 20;
    }
    score
}

fn normalize_channel(channel: Option<&str>) -> Result<String> {
    let value = channel.unwrap_or("stable").trim().to_lowercase();
    if !FLUTTER_CHANNELS.contains(&value.as_str()) {
        let known = FLUTTER_CHANNELS.join(", ");
        bail!(
            "Unknown Flutter channel '{}'. Use one of: {known}",
            channel.unwrap_or("")
        );
    }
    Ok(value)
}

/// Exact match, or prefix match (`3.24` matches `3.24.5`).
fn match_version(release_version: &str, requested: &str) -> bool {
    let rv = release_version.trim();
    let req = requested.trim();
    if rv == req {
        return true;
    }
    rv.starts_with(&format!("{req}."))
}

/// Return `(download_url, version)` for a Flutter channel (and optional version).
fn resolve_flutter_url(channel: &str, version: Option<&str>) -> Result<(String, String)> {
    let channel = normalize_channel(Some(channel))?;
    let plat = releases_platform()?;
    let meta = download_json(&format!("{RELEASES_BASE}/releases_{plat}.json"))?;
    let base_url = meta
        .get("base_url")
        .and_then(|v| v.as_str())
        .unwrap_or(RELEASES_BASE)
        .trim_end_matches('/')
        .to_string();
    let empty_map = serde_json::Map::new();
    let current = meta
        .get("current_release")
        .and_then(|v| v.as_object())
        .unwrap_or(&empty_map);
    let releases = meta
        .get("releases")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Unexpected Flutter releases metadata"))?;

    let mut candidates: Vec<&serde_json::Value> = releases
        .iter()
        .filter(|r| {
            r.get("channel").and_then(|v| v.as_str()) == Some(channel.as_str())
                && r.get("archive").is_some()
        })
        .collect();
    if candidates.is_empty() {
        bail!("No Flutter releases found for channel '{channel}'");
    }

    if let Some(version) = version {
        let matched: Vec<&serde_json::Value> = candidates
            .iter()
            .copied()
            .filter(|r| {
                let rv = r.get("version").and_then(|v| v.as_str()).unwrap_or("");
                match_version(rv, version)
            })
            .collect();
        if matched.is_empty() {
            bail!("No Flutter {channel} release matching version '{version}'");
        }
        candidates = matched;
    } else if let Some(channel_hash) = current.get(&channel).and_then(|v| v.as_str()) {
        let hashed: Vec<&serde_json::Value> = candidates
            .iter()
            .copied()
            .filter(|r| r.get("hash").and_then(|v| v.as_str()) == Some(channel_hash))
            .collect();
        if !hashed.is_empty() {
            candidates = hashed;
        }
    }

    let best = candidates
        .into_iter()
        .max_by_key(|r| {
            let archive = r.get("archive").and_then(|v| v.as_str()).unwrap_or("");
            score_archive(archive)
        })
        .unwrap();
    let archive = best
        .get("archive")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let resolved = best
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    Ok((format!("{base_url}/{archive}"), resolved))
}

fn flutter_bin(ctx: &InstallContext) -> PathBuf {
    if is_windows() {
        ctx.install_dir.join("bin").join("flutter.bat")
    } else {
        ctx.install_dir.join("bin").join("flutter")
    }
}

pub struct FlutterPlugin;

impl Plugin for FlutterPlugin {
    fn id(&self) -> &'static str {
        "flutter"
    }

    fn name(&self) -> &'static str {
        "Flutter"
    }

    fn description(&self) -> &'static str {
        "Download Flutter SDK into the machine dev folder (optional --channel / --version)."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        let binary = flutter_bin(ctx);
        if binary.is_file() {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail(binary.display().to_string());
        }
        let has_contents = ctx.install_dir.is_dir()
            && std::fs::read_dir(&ctx.install_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_contents {
            return PluginStatus::new(InstallState::Partial, Some(ctx.install_dir.clone()))
                .with_detail("Install dir exists but flutter binary is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let channel = normalize_channel(ctx.channel.as_deref())?;
        let (url, version) = resolve_flutter_url(&channel, ctx.version.as_deref())?;
        println!("Flutter {channel} {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Flutter");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = flutter_bin(ctx);
        if !binary.is_file() {
            bail!(
                "Flutter archive extracted but binary not found at {}",
                binary.display()
            );
        }
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Flutter {channel} {version} installed at {}",
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

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.join("bin")],
            vars: vec![(
                "FLUTTER_ROOT".to_string(),
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
    fn normalize_channel_defaults_to_stable() {
        assert_eq!(normalize_channel(None).unwrap(), "stable");
        assert_eq!(normalize_channel(Some("Beta")).unwrap(), "beta");
    }

    #[test]
    fn normalize_channel_rejects_unknown() {
        assert!(normalize_channel(Some("nightly")).is_err());
    }

    #[test]
    fn match_version_exact_and_prefix() {
        assert!(match_version("3.24.5", "3.24.5"));
        assert!(match_version("3.24.5", "3.24"));
        assert!(!match_version("3.245", "3.24"));
    }

    #[test]
    fn score_archive_prefers_matching_arch() {
        // Whatever the host arch is, an archive naming that arch should score >= one that doesn't.
        let hints = arch_hints();
        if let Some(hint) = hints.first() {
            let matching = format!("flutter_{hint}_1.0.zip");
            assert!(score_archive(&matching) >= score_archive("flutter_other_1.0.zip"));
        }
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = FlutterPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("flutter"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
