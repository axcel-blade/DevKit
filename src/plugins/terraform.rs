//! Terraform plugin — latest ZIP from HashiCorp releases.

use crate::download::{download_json, install_archive_from_url};
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const INDEX: &str = "https://releases.hashicorp.com/terraform/index.json";
const MARKER: &str = ".devkit-terraform";

/// True for plain `X.Y.Z` versions; rejects pre-releases like `1.9.0-rc1` or
/// `1.9.0-beta1`, which HashiCorp's index otherwise mixes in with stable ones.
fn is_release(v: &str) -> bool {
    !v.is_empty()
        && v.split('.')
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn version_key(v: &str) -> Vec<u32> {
    v.split('.').map(|p| p.parse().unwrap_or(0)).collect()
}

fn resolve_terraform_download() -> Result<(String, String)> {
    let meta = download_json(INDEX)?;
    let versions = meta
        .get("versions")
        .and_then(|v| v.as_object())
        .cloned()
        .unwrap_or_default();
    if versions.is_empty() {
        bail!("Unexpected Terraform releases index");
    }
    let release_versions: Vec<&String> = versions.keys().filter(|v| is_release(v)).collect();
    if release_versions.is_empty() {
        bail!("No stable Terraform versions found");
    }
    let version = release_versions
        .into_iter()
        .max_by_key(|v| version_key(v))
        .unwrap()
        .clone();

    let host = current_os();
    let goos = match host {
        HostOS::Windows => "windows",
        HostOS::Linux => "linux",
        HostOS::MacOS => "darwin",
        HostOS::Other => bail!("Terraform is not supported on this OS: other"),
    };
    let arch = cpu_arch();
    let goarch = if arch == "aarch64" { "arm64" } else { "amd64" };

    let builds = versions
        .get(&version)
        .and_then(|v| v.get("builds"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for build in &builds {
        let os = build.get("os").and_then(|v| v.as_str()).unwrap_or("");
        let arch_field = build.get("arch").and_then(|v| v.as_str()).unwrap_or("");
        if os == goos && arch_field == goarch {
            let url = build.get("url").and_then(|v| v.as_str()).unwrap_or("");
            if !url.is_empty() {
                return Ok((url.to_string(), version));
            }
        }
    }
    bail!("No Terraform build for {goos}/{goarch} version {version}")
}

fn terraform_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir.join(if is_windows() {
        "terraform.exe"
    } else {
        "terraform"
    })
}

pub struct TerraformPlugin;

impl Plugin for TerraformPlugin {
    fn id(&self) -> &'static str {
        "terraform"
    }

    fn name(&self) -> &'static str {
        "Terraform"
    }

    fn description(&self) -> &'static str {
        "Download latest Terraform ZIP and add it to PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &terraform_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but terraform binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_terraform_download()?;
        println!("Terraform {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Terraform");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            false,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        if !terraform_bin(ctx).is_file() {
            bail!(
                "Terraform extracted but binary not found at {}",
                terraform_bin(ctx).display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Terraform {version} installed at {}",
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
            paths: vec![ctx.install_dir.clone()],
            vars: vec![(
                "TERRAFORM_HOME".to_string(),
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
    use std::path::Path;

    fn ctx(tmp: &Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("terraform"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = TerraformPlugin;
        let c = ctx(tmp.path());
        assert_eq!(plugin.status(&c).state, InstallState::NotInstalled);
    }

    #[test]
    fn is_release_filters_prereleases() {
        assert!(is_release("1.9.5"));
        assert!(!is_release("1.9.5-beta1"));
        assert!(!is_release(""));
    }

    #[test]
    fn version_key_picks_max_numeric() {
        let versions = vec!["1.9.5", "1.10.0", "1.2.0"];
        let max = versions.into_iter().max_by_key(|v| version_key(v)).unwrap();
        assert_eq!(max, "1.10.0");
    }
}
