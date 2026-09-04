//! Docker CLI plugin — official static client binaries (engine/daemon not included).
//!
//! Downloads the Docker CLI (+ bundled tools from the static archive) into the machine
//! `dev` folder. Talking to a real daemon still requires Docker Engine or Desktop
//! separately (or `DOCKER_HOST` pointing at a remote engine).

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin};
use crate::plugin_utils::{binary_status, find_files_named, read_text_url};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const MARKER: &str = ".devkit-docker";
const BASE: &str = "https://download.docker.com";

/// Return `(os_slug, arch_slug, archive_ext)` for static Docker downloads.
fn channel_dir() -> Result<(&'static str, &'static str, &'static str)> {
    let host = current_os();
    let arch = cpu_arch();
    let arch_slug = match arch.as_str() {
        "aarch64" => "aarch64",
        "x64" => "x86_64",
        other => bail!("Unsupported arch for Docker CLI: {other}"),
    };
    match host {
        HostOS::Linux => Ok(("linux", arch_slug, "tgz")),
        HostOS::MacOS => Ok(("mac", arch_slug, "tgz")),
        HostOS::Windows => {
            if arch != "x64" {
                bail!("Docker static Windows builds are x86_64 only");
            }
            Ok(("win", "x86_64", "zip"))
        }
        HostOS::Other => bail!("Docker CLI is not supported on this OS: other"),
    }
}

/// Return `(download_url, version)` for the latest stable static Docker archive.
fn resolve_docker_download() -> Result<(String, String)> {
    let (os_slug, arch_slug, ext) = channel_dir()?;
    let index_url = format!("{BASE}/{os_slug}/static/stable/{arch_slug}/");
    let html = read_text_url(&index_url)?;
    let pattern = format!(r"docker-(\d+\.\d+\.\d+)\.{}", regex::escape(ext));
    let re = regex::Regex::new(&pattern).unwrap();
    let versions: Vec<String> = re.captures_iter(&html).map(|c| c[1].to_string()).collect();
    if versions.is_empty() {
        bail!("No Docker static archives found at {index_url}");
    }
    let key = |v: &str| -> Vec<u32> { v.split('.').map(|p| p.parse().unwrap_or(0)).collect() };
    let version = versions.into_iter().max_by_key(|v| key(v)).unwrap();
    let name = format!("docker-{version}.{ext}");
    Ok((format!("{index_url}{name}"), version))
}

fn docker_bin(ctx: &InstallContext) -> PathBuf {
    let name = if is_windows() { "docker.exe" } else { "docker" };
    let direct = ctx.install_dir.join(name);
    if direct.is_file() {
        return direct;
    }
    let nested = ctx.install_dir.join("docker").join(name);
    if nested.is_file() {
        return nested;
    }
    let matches = find_files_named(&ctx.install_dir, name);
    if !matches.is_empty() {
        return matches.into_iter().next().unwrap();
    }
    direct
}

/// Flatten `docker/` archive contents into `install_dir` when needed.
fn layout_docker(ctx: &InstallContext) -> Result<()> {
    let binary = docker_bin(ctx);
    if binary.is_file() && binary.parent() != Some(ctx.install_dir.as_path()) {
        if let Some(parent) = binary.parent() {
            for entry in std::fs::read_dir(parent)? {
                let entry = entry?;
                let dest = ctx.install_dir.join(entry.file_name());
                if entry.file_type()?.is_file() && !dest.exists() {
                    std::fs::copy(entry.path(), &dest)?;
                }
            }
        }
    }
    Ok(())
}

pub struct DockerPlugin;

impl Plugin for DockerPlugin {
    fn id(&self) -> &'static str {
        "docker"
    }

    fn name(&self) -> &'static str {
        "Docker CLI"
    }

    fn description(&self) -> &'static str {
        "Download official static Docker CLI binaries and add them to PATH. \
         Does not install the Docker engine/daemon."
    }

    fn status(&self, ctx: &InstallContext) -> crate::plugin::PluginStatus {
        binary_status(
            &docker_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but docker binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_docker_download()?;
        println!("Docker CLI {version}");
        println!("URL: {url}");
        println!(
            "Note: this installs the client only. Start Docker Engine/Desktop \
             (or set DOCKER_HOST) to talk to a daemon."
        );
        let mut progress = download_progress("Downloading Docker CLI");
        // Archives contain a top-level docker/ directory with binaries.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        layout_docker(ctx)?;
        let binary = docker_bin(ctx);
        if !binary.is_file() {
            bail!(
                "Docker CLI extracted but binary not found under {}",
                ctx.install_dir.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Docker CLI {version} installed at {}",
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
        let binary = docker_bin(ctx);
        let path_dir = if binary.is_file() {
            binary.parent().unwrap_or(&ctx.install_dir).to_path_buf()
        } else {
            ctx.install_dir.clone()
        };
        EnvSpec {
            paths: vec![path_dir],
            vars: vec![(
                "DOCKER_HOME".to_string(),
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
    fn channel_dir_returns_known_triple() {
        let (os_slug, arch_slug, ext) = channel_dir().unwrap();
        assert!(["linux", "mac", "win"].contains(&os_slug));
        assert!(["aarch64", "x86_64"].contains(&arch_slug));
        assert!(["tgz", "zip"].contains(&ext));
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = DockerPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("docker"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
