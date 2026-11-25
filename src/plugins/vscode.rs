//! Visual Studio Code plugin — portable stable build from Microsoft's update service.
//!
//! Windows / Linux: archive build extracted under `<dev>/vscode` with a `data`
//! folder so VS Code runs in portable mode. macOS: the universal `.app` ZIP,
//! unpacked with `ditto` so framework symlinks and permissions survive.

use crate::download::{download_file, download_json_value, extract_archive};
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;

const MARKER: &str = ".devkit-vscode";
const RELEASES: &str = "https://update.code.visualstudio.com/api/releases/stable";
const MAC_APP: &str = "Visual Studio Code.app";

/// Update-service platform id for this host.
fn platform_id() -> Result<&'static str> {
    let arm = cpu_arch() == "aarch64";
    Ok(match current_os() {
        HostOS::Windows if arm => "win32-arm64-archive",
        HostOS::Windows => "win32-x64-archive",
        HostOS::Linux if arm => "linux-arm64",
        HostOS::Linux => "linux-x64",
        HostOS::MacOS => "darwin-universal",
        HostOS::Other => bail!("Visual Studio Code is not supported on this OS: other"),
    })
}

fn latest_stable() -> Result<String> {
    let releases = download_json_value(RELEASES)?;
    releases
        .as_array()
        .and_then(|a| a.first())
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .context("Could not read the latest VS Code stable version")
}

/// Directory holding the `code` CLI launcher.
fn bin_dir(ctx: &InstallContext) -> PathBuf {
    if current_os() == HostOS::MacOS {
        ctx.install_dir
            .join(MAC_APP)
            .join("Contents")
            .join("Resources")
            .join("app")
            .join("bin")
    } else {
        ctx.install_dir.join("bin")
    }
}

fn code_bin(ctx: &InstallContext) -> PathBuf {
    bin_dir(ctx).join(if is_windows() { "code.cmd" } else { "code" })
}

pub struct VscodePlugin;

impl Plugin for VscodePlugin {
    fn id(&self) -> &'static str {
        "vscode"
    }

    fn name(&self) -> &'static str {
        "Visual Studio Code"
    }

    fn description(&self) -> &'static str {
        "Download portable Visual Studio Code (stable) and add the `code` CLI to PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &code_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but the code launcher is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let version = latest_stable()?;
        let platform = platform_id()?;
        let url = format!("https://update.code.visualstudio.com/{version}/{platform}/stable");
        println!("Visual Studio Code {version}");
        println!("URL: {url}");
        let filename = if current_os() == HostOS::Linux {
            format!("vscode-{version}-{platform}.tar.gz")
        } else {
            format!("vscode-{version}-{platform}.zip")
        };
        let mut progress = download_progress("Downloading Visual Studio Code");
        let archive = download_file(
            &url,
            None,
            Some(&filename),
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();

        match current_os() {
            HostOS::MacOS => {
                std::fs::create_dir_all(&ctx.install_dir)?;
                let status = std::process::Command::new("ditto")
                    .args(["-x", "-k"])
                    .arg(&archive)
                    .arg(&ctx.install_dir)
                    .status()
                    .context("Failed to run ditto to unpack Visual Studio Code")?;
                if !status.success() {
                    bail!("ditto failed to unpack {}", archive.display());
                }
            }
            // Windows ZIP has files at the root; Linux tarball nests under VSCode-linux-*/.
            HostOS::Linux => {
                extract_archive(&archive, &ctx.install_dir, true)?;
            }
            _ => {
                extract_archive(&archive, &ctx.install_dir, false)?;
            }
        }

        let binary = code_bin(ctx);
        if !binary.is_file() {
            bail!(
                "Visual Studio Code extracted but launcher not found at {}",
                binary.display()
            );
        }
        if current_os() != HostOS::MacOS {
            // A `data` folder next to the executable enables portable mode.
            std::fs::create_dir_all(ctx.install_dir.join("data"))?;
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Visual Studio Code {version} installed at {}",
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

    fn latest_version(&self, _ctx: &InstallContext) -> Result<Option<String>> {
        Ok(Some(latest_stable()?))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![bin_dir(ctx)],
            vars: vec![(
                "VSCODE_HOME".to_string(),
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
            install_dir: tmp.path().join("vscode"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(VscodePlugin.status(&ctx).state, InstallState::NotInstalled);
    }

    #[test]
    fn platform_id_is_known_for_host() {
        assert!(platform_id().is_ok());
    }
}
