//! CMake plugin — latest official binary from Kitware GitHub releases.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, github_latest_release, pick_release_asset};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const MARKER: &str = ".devkit-cmake";

fn resolve_cmake_download() -> Result<(String, String)> {
    let release = github_latest_release("Kitware", "CMake")?;
    let host = current_os();
    let arch = cpu_arch();
    match host {
        HostOS::Windows => {
            if arch != "x64" {
                bail!("Unsupported Windows arch for CMake ZIP: {arch}");
            }
            pick_release_asset(&release, &["windows-x86_64.zip"])
        }
        HostOS::Linux => {
            if arch == "aarch64" {
                pick_release_asset(&release, &["linux-aarch64.tar.gz"])
            } else {
                pick_release_asset(&release, &["linux-x86_64.tar.gz"])
            }
        }
        HostOS::MacOS => pick_release_asset(&release, &["macos-universal.tar.gz"]),
        HostOS::Other => bail!("CMake is not supported on this OS: other"),
    }
}

fn cmake_bin(ctx: &InstallContext) -> PathBuf {
    // Kitware archives nest under cmake-<ver>-<plat>/bin after strip, or CMake.app on macOS.
    if is_windows() {
        return ctx.install_dir.join("bin").join("cmake.exe");
    }
    let app = ctx
        .install_dir
        .join("CMake.app")
        .join("Contents")
        .join("bin")
        .join("cmake");
    if app.is_file() {
        return app;
    }
    ctx.install_dir.join("bin").join("cmake")
}

pub struct CmakePlugin;

impl Plugin for CmakePlugin {
    fn id(&self) -> &'static str {
        "cmake"
    }

    fn name(&self) -> &'static str {
        "CMake"
    }

    fn description(&self) -> &'static str {
        "Download latest CMake binary and set CMAKE_HOME / PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &cmake_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but cmake binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_cmake_download()?;
        println!("CMake {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading CMake");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = cmake_bin(ctx);
        if !binary.is_file() {
            bail!(
                "CMake extracted but binary not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("CMake {version} installed at {}", ctx.install_dir.display()),
        ))
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let binary = cmake_bin(ctx);
        let parent = binary.parent().unwrap_or(&ctx.install_dir).to_path_buf();
        EnvSpec {
            paths: vec![parent],
            vars: vec![(
                "CMAKE_HOME".to_string(),
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
        let plugin = CmakePlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("cmake"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }

    #[test]
    fn cmake_bin_defaults_to_bin_dir_on_unix() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = InstallContext {
            install_dir: tmp.path().join("cmake"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        if !is_windows() {
            assert_eq!(cmake_bin(&ctx), ctx.install_dir.join("bin").join("cmake"));
        }
    }
}
