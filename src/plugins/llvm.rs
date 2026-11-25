//! LLVM / Clang plugin — official prebuilt toolchain from llvm-project GitHub releases.
//!
//! Uses the `LLVM-<ver>-<OS>-<ARCH>.tar.xz` assets. Upstream publishes no
//! Intel macOS build, so that host is pointed at Homebrew instead.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, github_latest_release};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use serde_json::Value;
use std::path::PathBuf;

const MARKER: &str = ".devkit-llvm";

/// Asset name suffix for this host, e.g. `-Linux-X64.tar.xz`.
fn asset_suffix() -> Result<&'static str> {
    let arm = cpu_arch() == "aarch64";
    Ok(match current_os() {
        HostOS::Windows if arm => "-Windows-ARM64.tar.xz",
        HostOS::Windows => "-Windows-X64.tar.xz",
        HostOS::Linux if arm => "-Linux-ARM64.tar.xz",
        HostOS::Linux => "-Linux-X64.tar.xz",
        HostOS::MacOS if arm => "-macOS-ARM64.tar.xz",
        HostOS::MacOS => bail!(
            "LLVM publishes no Intel macOS binaries.\n\
             Install it with Homebrew instead:  brew install llvm"
        ),
        HostOS::Other => bail!("LLVM is not supported on this OS: other"),
    })
}

/// `llvmorg-21.1.6` -> `21.1.6`.
fn version_from_tag(tag: &str) -> String {
    tag.strip_prefix("llvmorg-").unwrap_or(tag).to_string()
}

/// Exact-suffix match so `.sig` / `.jsonl` siblings are never picked.
fn pick_asset(release: &Value, suffix: &str) -> Option<String> {
    release.get("assets")?.as_array()?.iter().find_map(|a| {
        let name = a.get("name")?.as_str()?;
        if name.starts_with("LLVM-") && name.ends_with(suffix) {
            a.get("browser_download_url")?.as_str().map(str::to_string)
        } else {
            None
        }
    })
}

fn resolve_llvm_download() -> Result<(String, String)> {
    let suffix = asset_suffix()?;
    let release = github_latest_release("llvm", "llvm-project")?;
    let tag = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let version = version_from_tag(tag);
    match pick_asset(&release, suffix) {
        Some(url) => Ok((url, version)),
        None => bail!("No LLVM asset ending in {suffix} in release {tag}"),
    }
}

fn clang_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join("bin")
        .join(if is_windows() { "clang.exe" } else { "clang" })
}

pub struct LlvmPlugin;

impl Plugin for LlvmPlugin {
    fn id(&self) -> &'static str {
        "llvm"
    }

    fn name(&self) -> &'static str {
        "LLVM / Clang"
    }

    fn description(&self) -> &'static str {
        "Download the latest LLVM / Clang toolchain and set LLVM_HOME / PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &clang_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but clang binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_llvm_download()?;
        println!("LLVM {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading LLVM");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = clang_bin(ctx);
        if !binary.is_file() {
            bail!("LLVM extracted but clang not found at {}", binary.display());
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("LLVM {version} installed at {}", ctx.install_dir.display()),
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
        Ok(Some(resolve_llvm_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.join("bin")],
            vars: vec![(
                "LLVM_HOME".to_string(),
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
        let ctx = InstallContext {
            install_dir: tmp.path().join("llvm"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(LlvmPlugin.status(&ctx).state, InstallState::NotInstalled);
    }

    #[test]
    fn version_from_tag_strips_prefix() {
        assert_eq!(version_from_tag("llvmorg-21.1.6"), "21.1.6");
    }

    #[test]
    fn pick_asset_skips_signatures() {
        let release = serde_json::json!({ "assets": [
            { "name": "LLVM-21.1.6-Linux-X64.tar.xz.sig", "browser_download_url": "sig" },
            { "name": "LLVM-21.1.6-Linux-X64.tar.xz", "browser_download_url": "ok" },
        ]});
        assert_eq!(
            pick_asset(&release, "-Linux-X64.tar.xz").as_deref(),
            Some("ok")
        );
    }
}
