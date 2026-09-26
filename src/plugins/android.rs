//! Android SDK command-line tools plugin (helper for Flutter / Android builds).
//!
//! Downloads Google's cmdline-tools ZIP into the machine `dev` folder, lays out
//! `cmdline-tools/latest/` as required by `sdkmanager`, and sets
//! `ANDROID_HOME` / `ANDROID_SDK_ROOT`. Does not accept licenses or install
//! platform-tools — run `sdkmanager` yourself after install.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

// Pin a known cmdline-tools build; bump when cutting a release that tracks newer tools.
const CMDLINE_TOOLS_BUILD: &str = "16111833";
// Google stopped publishing Intel macOS ZIPs after this build (newer builds ship
// `mac_arm64` only), so Intel Macs stay on the last build that has a `mac` ZIP.
const CMDLINE_TOOLS_BUILD_MAC_INTEL: &str = "15641748";
const REPO: &str = "https://dl.google.com/android/repository";
const MARKER: &str = ".devkit-android";

/// Return `(download_url, build_id)` for this OS/arch.
fn resolve_cmdline_tools_url() -> Result<(String, String)> {
    let (slug, build) = match current_os() {
        HostOS::Windows => ("win", CMDLINE_TOOLS_BUILD),
        HostOS::MacOS if cpu_arch() == "aarch64" => ("mac_arm64", CMDLINE_TOOLS_BUILD),
        HostOS::MacOS => ("mac", CMDLINE_TOOLS_BUILD_MAC_INTEL),
        HostOS::Linux => ("linux", CMDLINE_TOOLS_BUILD),
        HostOS::Other => bail!("Android cmdline-tools are not supported on: other"),
    };
    let name = format!("commandlinetools-{slug}-{build}_latest.zip");
    Ok((format!("{REPO}/{name}"), build.to_string()))
}

fn sdkmanager(ctx: &InstallContext) -> PathBuf {
    let base = ctx
        .install_dir
        .join("cmdline-tools")
        .join("latest")
        .join("bin");
    if is_windows() {
        base.join("sdkmanager.bat")
    } else {
        base.join("sdkmanager")
    }
}

/// Move extracted `cmdline-tools/` contents under `cmdline-tools/latest/`.
///
/// Google's ZIP unpacks as `cmdline-tools/{bin,lib,...}`. `sdkmanager`
/// expects `cmdline-tools/latest/bin/sdkmanager` (or a versioned sibling).
fn layout_cmdline_tools(ctx: &InstallContext) -> Result<()> {
    let root = &ctx.install_dir;
    let extracted = root.join("cmdline-tools");
    let latest = extracted.join("latest");
    if sdkmanager(ctx).is_file() {
        return Ok(());
    }
    if !extracted.is_dir() {
        bail!("Expected cmdline-tools folder under {}", root.display());
    }
    // Zip unpacks as install_dir/cmdline-tools/{bin,lib,...}; nest under latest/.
    if latest.exists() {
        std::fs::remove_dir_all(&latest)?;
    }
    std::fs::create_dir_all(&latest)?;
    let children: Vec<_> = std::fs::read_dir(&extracted)?
        .filter_map(|e| e.ok())
        .collect();
    for child in children {
        if child.file_name().to_str() == Some("latest") {
            continue;
        }
        std::fs::rename(child.path(), latest.join(child.file_name()))?;
    }
    Ok(())
}

pub struct AndroidPlugin;

impl Plugin for AndroidPlugin {
    fn id(&self) -> &'static str {
        "android"
    }

    fn name(&self) -> &'static str {
        "Android SDK"
    }

    fn description(&self) -> &'static str {
        "Download Android SDK cmdline-tools and set ANDROID_HOME / ANDROID_SDK_ROOT."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        let binary = sdkmanager(ctx);
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
                .with_detail("Install dir exists but sdkmanager is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, build) = resolve_cmdline_tools_url()?;
        println!("Android cmdline-tools {build}");
        println!("URL: {url}");
        // Keep top-level cmdline-tools/ then nest under latest/.
        let mut progress = download_progress("Downloading Android cmdline-tools");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            false,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        layout_cmdline_tools(ctx)?;
        let binary = sdkmanager(ctx);
        if !binary.is_file() {
            bail!(
                "cmdline-tools extracted but sdkmanager not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{build}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Android cmdline-tools {build} installed at {}. \
                 Accept licenses and install packages with sdkmanager \
                 (e.g. platform-tools) as needed for Flutter.",
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
        Ok(Some(resolve_cmdline_tools_url()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let home = ctx
            .install_dir
            .canonicalize()
            .unwrap_or_else(|_| ctx.install_dir.clone())
            .display()
            .to_string();
        EnvSpec {
            paths: vec![
                ctx.install_dir
                    .join("cmdline-tools")
                    .join("latest")
                    .join("bin"),
                ctx.install_dir.join("platform-tools"),
            ],
            vars: vec![
                ("ANDROID_HOME".to_string(), home.clone()),
                ("ANDROID_SDK_ROOT".to_string(), home),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_cmdline_tools_url_contains_build() {
        let (url, build) = resolve_cmdline_tools_url().unwrap();
        assert!(build == CMDLINE_TOOLS_BUILD || build == CMDLINE_TOOLS_BUILD_MAC_INTEL);
        assert!(url.contains(&build));
        assert!(url.starts_with(REPO));
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = AndroidPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("android"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }

    #[test]
    fn layout_cmdline_tools_nests_contents() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = InstallContext {
            install_dir: tmp.path().join("android"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        let bin = ctx.install_dir.join("cmdline-tools").join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join("sdkmanager"), b"stub").unwrap();
        layout_cmdline_tools(&ctx).unwrap();
        assert!(ctx
            .install_dir
            .join("cmdline-tools")
            .join("latest")
            .join("bin")
            .join("sdkmanager")
            .is_file());
    }
}
