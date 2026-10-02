//! Android NDK plugin — native C/C++ toolchain (clang, ndk-build, CMake
//! toolchain file) for Android builds. Companion to the android plugin.
//!
//! Downloads Google's pinned NDK ZIP, strips the `android-ndk-<rev>/` top
//! folder and sets `ANDROID_NDK_HOME` / `ANDROID_NDK_ROOT`.

use crate::download::install_archive_from_url;
use crate::platform::{current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

// Pin a known NDK release; bump when cutting a release that tracks a newer NDK.
const NDK_REVISION: &str = "r29";
const REPO: &str = "https://dl.google.com/android/repository";
const MARKER: &str = ".devkit-android-ndk";

/// Return `(download_url, revision)` for this OS.
fn resolve_ndk_url(host: HostOS) -> Result<(String, String)> {
    let slug = match host {
        HostOS::Windows => "windows",
        HostOS::MacOS => "darwin",
        HostOS::Linux => "linux",
        HostOS::Other => bail!("Android NDK is not supported on: other"),
    };
    let name = format!("android-ndk-{NDK_REVISION}-{slug}.zip");
    Ok((format!("{REPO}/{name}"), NDK_REVISION.to_string()))
}

fn ndk_build(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir.join(if is_windows() {
        "ndk-build.cmd"
    } else {
        "ndk-build"
    })
}

pub struct AndroidNdkPlugin;

impl Plugin for AndroidNdkPlugin {
    fn id(&self) -> &'static str {
        "android-ndk"
    }

    fn name(&self) -> &'static str {
        "Android NDK"
    }

    fn description(&self) -> &'static str {
        "Download the Android NDK native toolchain and set ANDROID_NDK_HOME / ANDROID_NDK_ROOT."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &ndk_build(ctx),
            &ctx.install_dir,
            "Install dir exists but ndk-build is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, revision) = resolve_ndk_url(current_os())?;
        println!("Android NDK {revision}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Android NDK");
        // ZIP root is android-ndk-<rev>/; strip it so the NDK sits at install_dir.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = ndk_build(ctx);
        if !binary.is_file() {
            bail!(
                "Android NDK extracted but ndk-build not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{revision}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Android NDK {revision} installed at {}",
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

    /// DevKit pins this revision, so "latest" is the pinned revision it installs.
    fn latest_version(&self, _ctx: &InstallContext) -> Result<Option<String>> {
        Ok(Some(NDK_REVISION.to_string()))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let home = ctx
            .install_dir
            .canonicalize()
            .unwrap_or_else(|_| ctx.install_dir.clone())
            .display()
            .to_string();
        EnvSpec {
            paths: vec![ctx.install_dir.clone()],
            vars: vec![
                ("ANDROID_NDK_HOME".to_string(), home.clone()),
                ("ANDROID_NDK_ROOT".to_string(), home),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::InstallState;

    fn ctx(tmp: &std::path::Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("android-ndk"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn resolve_ndk_url_matches_host() {
        let (url, rev) = resolve_ndk_url(HostOS::Linux).unwrap();
        assert_eq!(rev, NDK_REVISION);
        assert_eq!(url, format!("{REPO}/android-ndk-{NDK_REVISION}-linux.zip"));
        assert!(resolve_ndk_url(HostOS::Windows)
            .unwrap()
            .0
            .ends_with("-windows.zip"));
        assert!(resolve_ndk_url(HostOS::MacOS)
            .unwrap()
            .0
            .ends_with("-darwin.zip"));
        assert!(resolve_ndk_url(HostOS::Other).is_err());
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let c = ctx(tmp.path());
        assert_eq!(
            AndroidNdkPlugin.status(&c).state,
            InstallState::NotInstalled
        );
    }

    #[test]
    fn status_installed_when_ndk_build_present() {
        let tmp = tempfile::tempdir().unwrap();
        let c = ctx(tmp.path());
        std::fs::create_dir_all(&c.install_dir).unwrap();
        std::fs::write(ndk_build(&c), b"stub").unwrap();
        assert_eq!(AndroidNdkPlugin.status(&c).state, InstallState::Installed);
    }

    #[test]
    fn env_spec_sets_ndk_vars() {
        let tmp = tempfile::tempdir().unwrap();
        let c = ctx(tmp.path());
        let spec = AndroidNdkPlugin.env_spec(&c);
        assert_eq!(spec.paths, vec![c.install_dir.clone()]);
        assert_eq!(spec.vars[0].0, "ANDROID_NDK_HOME");
        assert_eq!(spec.vars[1].0, "ANDROID_NDK_ROOT");
    }
}
