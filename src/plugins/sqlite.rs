//! SQLite tools plugin — official sqlite-tools ZIP from sqlite.org.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, read_text_url};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const DOWNLOAD_PAGE: &str = "https://www.sqlite.org/download.html";
const MARKER: &str = ".devkit-sqlite";

/// Parse the sqlite.org download page for the matching sqlite-tools ZIP.
fn resolve_sqlite_download() -> Result<(String, String)> {
    let host = current_os();
    let arch = cpu_arch();
    let key = match host {
        HostOS::Windows => {
            if arch == "aarch64" {
                "sqlite-tools-win-arm64"
            } else {
                "sqlite-tools-win-x64"
            }
        }
        HostOS::Linux => {
            if arch == "aarch64" {
                bail!("sqlite.org does not publish linux-arm64 tools yet");
            }
            // official page currently ships x64 only
            "sqlite-tools-linux-x64"
        }
        HostOS::MacOS => {
            if arch == "aarch64" {
                "sqlite-tools-osx-arm64"
            } else {
                "sqlite-tools-osx-x64"
            }
        }
        HostOS::Other => bail!("SQLite tools are not supported on this OS: other"),
    };

    let html = read_text_url(DOWNLOAD_PAGE)?;
    // Product rows look like: 2026/sqlite-tools-win-x64-3530400.zip,size,sha
    let pattern = format!(r"(\d{{4}}/{}-\d+\.zip)", regex::escape(key));
    let re = regex::Regex::new(&pattern).unwrap();
    let Some(caps) = re.captures(&html) else {
        bail!("Could not find {key} on sqlite.org download page");
    };
    let rel = caps[1].to_string();
    let url = format!("https://www.sqlite.org/{rel}");
    let version_re = regex::Regex::new(r"-(\d+)\.zip$").unwrap();
    let version = version_re
        .captures(&rel)
        .map(|c| c[1].to_string())
        .unwrap_or_else(|| "unknown".to_string());
    Ok((url, version))
}

fn sqlite3_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir.join(if is_windows() {
        "sqlite3.exe"
    } else {
        "sqlite3"
    })
}

pub struct SqlitePlugin;

impl Plugin for SqlitePlugin {
    fn id(&self) -> &'static str {
        "sqlite"
    }

    fn name(&self) -> &'static str {
        "SQLite"
    }

    fn description(&self) -> &'static str {
        "Download official sqlite-tools ZIP (sqlite3 CLI) and add it to PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &sqlite3_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but sqlite3 binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_sqlite_download()?;
        println!("SQLite tools {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading SQLite tools");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            false,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        if !sqlite3_bin(ctx).is_file() {
            bail!(
                "SQLite tools extracted but sqlite3 not found at {}",
                sqlite3_bin(ctx).display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "SQLite tools {version} installed at {}",
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
                "SQLITE_HOME".to_string(),
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
            install_dir: tmp.join("sqlite"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = SqlitePlugin;
        let c = ctx(tmp.path());
        assert_eq!(plugin.status(&c).state, InstallState::NotInstalled);
    }

    #[test]
    fn parses_version_and_url_pattern() {
        let html = "blah 2026/sqlite-tools-win-x64-3530400.zip,12345,abcdef blah";
        let pattern = format!(
            r"(\d{{4}}/{}-\d+\.zip)",
            regex::escape("sqlite-tools-win-x64")
        );
        let re = regex::Regex::new(&pattern).unwrap();
        let caps = re.captures(html).unwrap();
        let rel = caps[1].to_string();
        assert_eq!(rel, "2026/sqlite-tools-win-x64-3530400.zip");
        let version_re = regex::Regex::new(r"-(\d+)\.zip$").unwrap();
        let version = version_re.captures(&rel).map(|c| c[1].to_string()).unwrap();
        assert_eq!(version, "3530400");
    }
}
