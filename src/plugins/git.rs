//! Git plugin — MinGit on Windows; system Git wrappers on macOS/Linux.
//!
//! Windows: resolve the latest MinGit ZIP from git-for-windows GitHub releases,
//! extract under `<dev>/git`, and put `cmd`/`bin` on PATH.
//!
//! macOS/Linux: Git has no official portable archive here, so DevKit registers
//! thin wrappers around an already-installed system `git`.

use crate::download::{download_json, install_archive_from_url};
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::progress::download_progress;
use anyhow::{bail, Result};
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

const GFW_LATEST: &str = "https://api.github.com/repos/git-for-windows/git/releases/latest";
const MARKER: &str = ".devkit-git";

fn git_binary(ctx: &InstallContext) -> Option<PathBuf> {
    let exe = if is_windows() { "git.exe" } else { "git" };
    let candidates = [
        ctx.install_dir.join("cmd").join(exe),
        ctx.install_dir.join("bin").join(exe),
        ctx.install_dir.join("mingw64").join("bin").join("git.exe"),
        ctx.install_dir.join("mingw32").join("bin").join("git.exe"),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

fn path_dirs(ctx: &InstallContext) -> Vec<PathBuf> {
    ["cmd", "bin"]
        .iter()
        .map(|rel| ctx.install_dir.join(rel))
        .filter(|d| d.is_dir())
        .collect()
}

/// Pick the latest MinGit ZIP URL for this Windows CPU arch. Returns `(url, version_label)`.
fn resolve_mingit_url() -> Result<(String, String)> {
    let arch = cpu_arch();
    let want = match arch.as_str() {
        "aarch64" => regex::Regex::new(r"(?i)^MinGit-.*-arm64\.zip$").unwrap(),
        "x64" => regex::Regex::new(r"(?i)^MinGit-.*-64-bit\.zip$").unwrap(),
        _ => regex::Regex::new(r"(?i)^MinGit-.*-32-bit\.zip$").unwrap(),
    };

    let meta = download_json(GFW_LATEST)?;
    let tag = meta
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let assets = meta.get("assets").and_then(|v| v.as_array());
    let mut matches: Vec<(&str, &str)> = Vec::new();
    if let Some(assets) = assets {
        for asset in assets {
            let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
            if name.to_lowercase().contains("busybox") {
                continue;
            }
            let url = asset
                .get("browser_download_url")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if want.is_match(name) && !url.is_empty() {
                matches.push((name, url));
            }
        }
    }
    if matches.is_empty() {
        bail!("No MinGit ZIP found for arch={arch} in git-for-windows latest release");
    }
    Ok((matches[0].1.to_string(), tag))
}

#[cfg(unix)]
fn write_unix_wrappers(install_dir: &Path, system_git: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let bin_dir = install_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    for name in ["git", "git-receive-pack", "git-upload-pack"] {
        let system = which::which(name).ok();
        let target = bin_dir.join(name);
        let exe = match (&system, name) {
            (Some(p), _) => p.clone(),
            (None, "git") => system_git.to_path_buf(),
            (None, _) => continue,
        };
        std::fs::write(
            &target,
            format!("#!/usr/bin/env bash\nexec \"{}\" \"$@\"\n", exe.display()),
        )?;
        let mut perms = std::fs::metadata(&target)?.permissions();
        perms.set_mode(perms.mode() | 0o111);
        std::fs::set_permissions(&target, perms)?;
    }
    std::fs::write(
        install_dir.join(MARKER),
        format!("system-wrapper\n{}\n", system_git.display()),
    )?;
    Ok(())
}

pub struct GitPlugin;

impl Plugin for GitPlugin {
    fn id(&self) -> &'static str {
        "git"
    }

    fn name(&self) -> &'static str {
        "Git"
    }

    fn description(&self) -> &'static str {
        "Install MinGit (Git for Windows portable ZIP). \
         On macOS/Linux, registers system Git if already installed."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        if let Some(binary) = git_binary(ctx) {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail(binary.display().to_string());
        }
        if ctx.install_dir.join(MARKER).is_file() {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail("system git wrappers");
        }
        let has_contents = ctx.install_dir.is_dir()
            && std::fs::read_dir(&ctx.install_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_contents {
            return PluginStatus::new(InstallState::Partial, Some(ctx.install_dir.clone()))
                .with_detail("Install dir exists but git binary is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        match current_os() {
            HostOS::Windows => install_windows(ctx),
            HostOS::MacOS | HostOS::Linux => install_unix(ctx),
            HostOS::Other => bail!("Git is not supported on this OS: other"),
        }
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: path_dirs(ctx),
            vars: vec![(
                "GIT_HOME".to_string(),
                ctx.install_dir
                    .canonicalize()
                    .unwrap_or_else(|_| ctx.install_dir.clone())
                    .display()
                    .to_string(),
            )],
        }
    }
}

fn install_windows(ctx: &InstallContext) -> Result<InstallResult> {
    let (url, version) = resolve_mingit_url()?;
    println!("Git for Windows MinGit {version}");
    println!("URL: {url}");
    let mut progress = download_progress("Downloading Git");
    install_archive_from_url(
        &url,
        &ctx.install_dir,
        false,
        None,
        Some(&mut |d, t| progress.update(d, t)),
    )?;
    progress.done();
    let binary = git_binary(ctx);
    if binary.is_none() {
        bail!(
            "MinGit extracted but git.exe was not found under {}",
            ctx.install_dir.display()
        );
    }
    std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!(
            "Git {version} (MinGit) installed at {}",
            ctx.install_dir.display()
        ),
    ))
}

fn install_unix(ctx: &InstallContext) -> Result<InstallResult> {
    let system_git = which::which("git").map_err(|_| {
        anyhow::anyhow!(
            "Git does not ship a portable macOS/Linux archive for DevKit.\n\
             Install Git with your platform tools, then re-run this command:\n\
             \x20 macOS:          xcode-select --install\n\
             \x20                 (or: brew install git)\n\
             \x20 Debian/Ubuntu:  sudo apt install git\n\
             \x20 Fedora:         sudo dnf install git\n\
             \x20 Arch:           sudo pacman -S git\n\
             Then:  devkit install git"
        )
    })?;
    std::fs::create_dir_all(&ctx.install_dir)?;
    #[cfg(unix)]
    write_unix_wrappers(&ctx.install_dir, &system_git)?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!("Registered system Git at {}", system_git.display()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn git_binary_none_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = InstallContext {
            install_dir: tmp.path().join("git"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert!(git_binary(&ctx).is_none());
    }
}
