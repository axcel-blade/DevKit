//! Node.js plugin — latest LTS binary from nodejs.org into the machine `dev` folder.
//!
//! Installs `node`, `npm`, and `npx`. Sets `NODE_HOME` and prepends `bin` to PATH.

use crate::download::{download_json_value, install_archive_from_url};
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use serde_json::Value;
use std::path::PathBuf;

const DIST_INDEX: &str = "https://nodejs.org/dist/index.json";
const DIST_BASE: &str = "https://nodejs.org/dist";
const MARKER: &str = ".devkit-node";

/// Match a key from Node's `files` list for this OS/arch.
fn archive_file_key() -> Result<&'static str> {
    let host = current_os();
    let arch = cpu_arch();
    match host {
        HostOS::Windows => match arch.as_str() {
            "aarch64" => Ok("win-arm64-zip"),
            "x64" => Ok("win-x64-zip"),
            other => bail!("Unsupported Windows arch for Node.js: {other}"),
        },
        HostOS::Linux => match arch.as_str() {
            "aarch64" => Ok("linux-arm64"),
            "x64" => Ok("linux-x64"),
            other => bail!("Unsupported Linux arch for Node.js: {other}"),
        },
        HostOS::MacOS => match arch.as_str() {
            "aarch64" => Ok("osx-arm64-tar"),
            "x64" => Ok("osx-x64-tar"),
            other => bail!("Unsupported macOS arch for Node.js: {other}"),
        },
        HostOS::Other => bail!("Node.js is not supported on this OS: other"),
    }
}

/// Build the dist archive filename for a Node version + file key.
fn archive_name(version: &str, file_key: &str) -> Result<String> {
    let ver = if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{version}")
    };
    let name = match file_key {
        "win-x64-zip" => format!("node-{ver}-win-x64.zip"),
        "win-arm64-zip" => format!("node-{ver}-win-arm64.zip"),
        "linux-x64" => format!("node-{ver}-linux-x64.tar.xz"),
        "linux-arm64" => format!("node-{ver}-linux-arm64.tar.xz"),
        "osx-x64-tar" => format!("node-{ver}-darwin-x64.tar.gz"),
        "osx-arm64-tar" => format!("node-{ver}-darwin-arm64.tar.gz"),
        other => bail!("No Node.js archive mapping for file key: {other}"),
    };
    Ok(name)
}

fn is_lts(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => !s.is_empty(),
        Some(_) => true,
    }
}

/// Return `(download_url, version)` for the newest LTS release on this OS.
fn resolve_node_lts_download() -> Result<(String, String)> {
    let data = download_json_value(DIST_INDEX)?;
    let entries = data
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Unexpected Node.js dist index (expected array)"))?;
    let file_key = archive_file_key()?;
    for entry in entries {
        if !entry.is_object() || !is_lts(entry.get("lts")) {
            continue;
        }
        let has_file = entry
            .get("files")
            .and_then(|v| v.as_array())
            .map(|files| {
                files
                    .iter()
                    .any(|f| f.as_str().map(|s| s == file_key).unwrap_or(false))
            })
            .unwrap_or(false);
        if !has_file {
            continue;
        }
        let version = entry
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if version.is_empty() {
            continue;
        }
        let name = archive_name(&version, file_key)?;
        return Ok((format!("{DIST_BASE}/{version}/{name}"), version));
    }
    bail!("No Node.js LTS release found with file key {file_key}")
}

fn node_bin(ctx: &InstallContext) -> PathBuf {
    if is_windows() {
        ctx.install_dir.join("node.exe")
    } else {
        ctx.install_dir.join("bin").join("node")
    }
}

pub struct NodePlugin;

impl Plugin for NodePlugin {
    fn id(&self) -> &'static str {
        "node"
    }

    fn name(&self) -> &'static str {
        "Node.js"
    }

    fn description(&self) -> &'static str {
        "Download latest Node.js LTS binary and set NODE_HOME / PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        let binary = node_bin(ctx);
        if binary.is_file() {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail(binary.display().to_string());
        }
        let has_contents = ctx.install_dir.is_dir()
            && std::fs::read_dir(&ctx.install_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_contents {
            return PluginStatus::new(InstallState::Partial, Some(ctx.install_dir.clone()))
                .with_detail("Install dir exists but node binary is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_node_lts_download()?;
        println!("Node.js LTS {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Node.js");
        // Archive root is node-vX.Y.Z-<plat>/; strip so node.exe or bin/ lands in install_dir.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = node_bin(ctx);
        if !binary.is_file() {
            bail!(
                "Node.js extracted but binary not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Node.js {version} installed at {}",
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
        Ok(Some(resolve_node_lts_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        // Windows ZIP lays binaries at the root; Unix uses bin/.
        let path_dir = if is_windows() {
            ctx.install_dir.clone()
        } else {
            ctx.install_dir.join("bin")
        };
        EnvSpec {
            paths: vec![path_dir],
            vars: vec![(
                "NODE_HOME".to_string(),
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

    #[test]
    fn archive_name_maps_known_keys() {
        assert_eq!(
            archive_name("v20.11.0", "win-x64-zip").unwrap(),
            "node-v20.11.0-win-x64.zip"
        );
        assert_eq!(
            archive_name("20.11.0", "linux-arm64").unwrap(),
            "node-v20.11.0-linux-arm64.tar.xz"
        );
    }

    #[test]
    fn archive_name_rejects_unknown_key() {
        assert!(archive_name("20.11.0", "bogus").is_err());
    }

    #[test]
    fn is_lts_handles_variants() {
        assert!(!is_lts(None));
        assert!(!is_lts(Some(&Value::Bool(false))));
        assert!(!is_lts(Some(&Value::Null)));
        assert!(is_lts(Some(&Value::String("Hydrogen".to_string()))));
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = NodePlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("node"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
