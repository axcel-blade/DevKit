//! Go plugin — latest stable toolchain from go.dev.

use crate::download::{download_json_value, install_archive_from_url};
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const DL_JSON: &str = "https://go.dev/dl/?mode=json";
const MARKER: &str = ".devkit-go";

fn go_os_arch() -> Result<(&'static str, &'static str)> {
    let host = current_os();
    let goos = match host {
        HostOS::Windows => "windows",
        HostOS::MacOS => "darwin",
        HostOS::Linux => "linux",
        HostOS::Other => bail!("Go is not supported on this OS: other"),
    };
    let arch = cpu_arch();
    if arch != "x64" && arch != "aarch64" {
        bail!("Unsupported arch for Go: {arch}");
    }
    let goarch = if arch == "aarch64" { "arm64" } else { "amd64" };
    Ok((goos, goarch))
}

/// Return `(download_url, version)` for the latest stable Go archive.
fn resolve_go_download() -> Result<(String, String)> {
    let data = download_json_value(DL_JSON)?;
    let entries = data
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Unexpected go.dev/dl JSON"))?;
    let (goos, goarch) = go_os_arch()?;
    for entry in entries {
        let is_stable = entry
            .get("stable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !entry.is_object() || !is_stable {
            continue;
        }
        let version = entry
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let files = entry.get("files").and_then(|v| v.as_array());
        if let Some(files) = files {
            for f in files {
                if f.get("kind").and_then(|v| v.as_str()) != Some("archive") {
                    continue;
                }
                let f_os = f.get("os").and_then(|v| v.as_str()).unwrap_or("");
                let f_arch = f.get("arch").and_then(|v| v.as_str()).unwrap_or("");
                if f_os == goos && f_arch == goarch {
                    let name = f
                        .get("filename")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    return Ok((format!("https://go.dev/dl/{name}"), version));
                }
            }
        }
    }
    bail!("No stable Go archive for {goos}/{goarch}")
}

fn go_bin(ctx: &InstallContext) -> PathBuf {
    let name = if is_windows() { "go.exe" } else { "go" };
    ctx.install_dir.join("bin").join(name)
}

pub struct GoPlugin;

impl Plugin for GoPlugin {
    fn id(&self) -> &'static str {
        "go"
    }

    fn name(&self) -> &'static str {
        "Go"
    }

    fn description(&self) -> &'static str {
        "Download latest stable Go toolchain and set GOROOT / PATH."
    }

    fn status(&self, ctx: &InstallContext) -> crate::plugin::PluginStatus {
        binary_status(
            &go_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but go binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_go_download()?;
        println!("Go {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Go");
        // Archive root is go/; strip so bin/ lands in install_dir.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = go_bin(ctx);
        if !binary.is_file() {
            bail!("Go extracted but binary not found at {}", binary.display());
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("Go {version} installed at {}", ctx.install_dir.display()),
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
        Ok(Some(resolve_go_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let root = ctx
            .install_dir
            .canonicalize()
            .unwrap_or_else(|_| ctx.install_dir.clone())
            .display()
            .to_string();
        EnvSpec {
            paths: vec![ctx.install_dir.join("bin")],
            vars: vec![
                ("GOROOT".to_string(), root.clone()),
                ("GO_HOME".to_string(), root),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::InstallState;

    #[test]
    fn go_os_arch_returns_known_pair() {
        let (goos, goarch) = go_os_arch().unwrap();
        assert!(["windows", "darwin", "linux"].contains(&goos));
        assert!(["amd64", "arm64"].contains(&goarch));
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = GoPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("go"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
