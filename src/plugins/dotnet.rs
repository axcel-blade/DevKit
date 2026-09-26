//! .NET SDK plugin — latest LTS SDK ZIP from Microsoft release metadata.

use crate::download::{download_json, install_archive_from_url};
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

// Track .NET LTS channel (10.0). Bump when DevKit moves to a newer LTS.
const DOTNET_CHANNEL: &str = "10.0";
const MARKER: &str = ".devkit-dotnet";

fn meta_url() -> String {
    format!(
        "https://dotnetcli.blob.core.windows.net/dotnet/release-metadata/{DOTNET_CHANNEL}/releases.json"
    )
}

fn rid() -> Result<String> {
    let host = current_os();
    let arch = cpu_arch();
    let cpu = match arch.as_str() {
        "aarch64" => "arm64",
        "x64" => "x64",
        other => bail!("Unsupported arch for .NET SDK: {other}"),
    };
    Ok(match host {
        HostOS::Windows => format!("win-{cpu}"),
        HostOS::Linux => format!("linux-{cpu}"),
        HostOS::MacOS => format!("osx-{cpu}"),
        HostOS::Other => bail!(".NET SDK is not supported on this OS: other"),
    })
}

/// Return `(sdk_zip_url, version)` for the latest GA SDK on this RID.
fn resolve_dotnet_download() -> Result<(String, String)> {
    let meta = download_json(&meta_url())?;
    let rid = rid()?;
    let releases = meta
        .get("releases")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for release in &releases {
        let Some(sdk) = release.get("sdk") else {
            continue;
        };
        let version = sdk
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let files = sdk
            .get("files")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        for f in &files {
            let name = f.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let url = f.get("url").and_then(|v| v.as_str()).unwrap_or("");
            if name.contains(&rid) && name.ends_with(".zip") && !url.is_empty() {
                return Ok((url.to_string(), version));
            }
        }
    }
    bail!("No .NET SDK ZIP found for RID {rid} in channel {DOTNET_CHANNEL}")
}

fn dotnet_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join(if is_windows() { "dotnet.exe" } else { "dotnet" })
}

pub struct DotnetPlugin;

impl Plugin for DotnetPlugin {
    fn id(&self) -> &'static str {
        "dotnet"
    }

    fn name(&self) -> &'static str {
        ".NET SDK"
    }

    fn description(&self) -> &'static str {
        "Download .NET SDK 10.0 LTS ZIP and set DOTNET_ROOT / PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &dotnet_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but dotnet binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_dotnet_download()?;
        println!(".NET SDK {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading .NET SDK");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            false,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        if !dotnet_bin(ctx).is_file() {
            bail!(
                ".NET SDK extracted but dotnet not found at {}",
                dotnet_bin(ctx).display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                ".NET SDK {version} installed at {}",
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
        Ok(Some(resolve_dotnet_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.clone()],
            vars: vec![(
                "DOTNET_ROOT".to_string(),
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
            install_dir: tmp.join("dotnet"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = DotnetPlugin;
        let c = ctx(tmp.path());
        assert_eq!(plugin.status(&c).state, InstallState::NotInstalled);
    }
}
