//! Android platform-tools plugin (adb / fastboot) — companion to the android plugin.

use crate::download::install_archive_from_url;
use crate::platform::{current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const REPO: &str = "https://dl.google.com/android/repository";
const MARKER: &str = ".devkit-platform-tools";

fn resolve_platform_tools_url() -> Result<(String, String)> {
    let slug = match current_os() {
        HostOS::Windows => "windows",
        HostOS::MacOS => "darwin",
        HostOS::Linux => "linux",
        HostOS::Other => bail!("platform-tools are not supported on: other"),
    };
    // Google publishes a rolling `latest` ZIP per OS.
    let name = format!("platform-tools-latest-{slug}.zip");
    Ok((format!("{REPO}/{name}"), "latest".to_string()))
}

fn adb_name() -> &'static str {
    if is_windows() {
        "adb.exe"
    } else {
        "adb"
    }
}

fn adb_bin(ctx: &InstallContext) -> PathBuf {
    // ZIP root is platform-tools/; may remain if strip_top_level False, or stripped.
    let nested = ctx.install_dir.join("platform-tools").join(adb_name());
    if nested.is_file() {
        return nested;
    }
    ctx.install_dir.join(adb_name())
}

pub struct PlatformToolsPlugin;

impl Plugin for PlatformToolsPlugin {
    fn id(&self) -> &'static str {
        "platform-tools"
    }

    fn name(&self) -> &'static str {
        "Android platform-tools"
    }

    fn description(&self) -> &'static str {
        "Download Android SDK platform-tools (adb, fastboot) and add them to PATH. \
         Pairs with the android cmdline-tools plugin for Flutter."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &adb_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but adb is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_platform_tools_url()?;
        println!("Android platform-tools ({version})");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading platform-tools");
        // Keep platform-tools/ folder as Google ships it.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            false,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        if !adb_bin(ctx).is_file() {
            bail!(
                "platform-tools extracted but adb not found at {}",
                adb_bin(ctx).display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Android platform-tools installed at {}",
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
        let adb = adb_bin(ctx);
        let path_dir = if adb.is_file() {
            adb.parent().unwrap_or(&ctx.install_dir).to_path_buf()
        } else {
            ctx.install_dir.join("platform-tools")
        };
        EnvSpec {
            paths: vec![path_dir.clone()],
            vars: vec![(
                "ANDROID_PLATFORM_TOOLS".to_string(),
                path_dir
                    .canonicalize()
                    .unwrap_or(path_dir)
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
    use std::path::Path;

    fn ctx(tmp: &Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("platform-tools"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = PlatformToolsPlugin;
        let c = ctx(tmp.path());
        assert_eq!(plugin.status(&c).state, InstallState::NotInstalled);
    }

    #[test]
    fn adb_bin_prefers_nested_platform_tools_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let c = ctx(tmp.path());
        let nested_dir = c.install_dir.join("platform-tools");
        std::fs::create_dir_all(&nested_dir).unwrap();
        std::fs::write(nested_dir.join(adb_name()), b"stub").unwrap();
        assert_eq!(adb_bin(&c), nested_dir.join(adb_name()));
    }
}
