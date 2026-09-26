//! MySQL Community Server plugin (8.4 LTS portable archives).
//!
//! Downloads official noinstall / tar archives into the machine `dev` folder and
//! sets `MYSQL_HOME` + PATH. Does not initialize a data directory or start
//! `mysqld` — run server setup separately after install.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

// Pin MySQL 8.4 LTS; bump when DevKit cuts a release that tracks a newer LTS.
const MYSQL_VERSION: &str = "8.4.11";
const MYSQL_SERIES: &str = "8.4";
const MARKER: &str = ".devkit-mysql";

fn get_base() -> String {
    format!("https://dev.mysql.com/get/Downloads/MySQL-{MYSQL_SERIES}/")
}

/// Return `(download_url, version)` for this OS/arch.
fn resolve_mysql_url() -> Result<(String, String)> {
    let host = current_os();
    let arch = cpu_arch();
    let name = match host {
        HostOS::Windows => {
            if arch != "x64" && arch != "aarch64" {
                bail!("Unsupported Windows arch for MySQL: {arch}");
            }
            // Official Windows community ZIP is x64 only.
            format!("mysql-{MYSQL_VERSION}-winx64.zip")
        }
        HostOS::Linux => {
            let arch_slug = if arch == "aarch64" {
                "aarch64"
            } else {
                "x86_64"
            };
            format!("mysql-{MYSQL_VERSION}-linux-glibc2.28-{arch_slug}.tar.xz")
        }
        HostOS::MacOS => {
            let arch_slug = if arch == "aarch64" { "arm64" } else { "x86_64" };
            format!("mysql-{MYSQL_VERSION}-macos15-{arch_slug}.tar.gz")
        }
        HostOS::Other => bail!("MySQL is not supported on this OS: other"),
    };
    Ok((format!("{}{name}", get_base()), MYSQL_VERSION.to_string()))
}

fn mysql_client(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join("bin")
        .join(if is_windows() { "mysql.exe" } else { "mysql" })
}

pub struct MysqlPlugin;

impl Plugin for MysqlPlugin {
    fn id(&self) -> &'static str {
        "mysql"
    }

    fn name(&self) -> &'static str {
        "MySQL"
    }

    fn description(&self) -> &'static str {
        "Download MySQL Community Server 8.4.11 (LTS) portable archive and set MYSQL_HOME / PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &mysql_client(ctx),
            &ctx.install_dir,
            "Install dir exists but mysql client is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_mysql_url()?;
        println!("MySQL Community Server {version}");
        println!("URL: {url}");
        println!(
            "Note: this installs binaries only. Initialize/start the server separately \
             (e.g. mysqld --initialize-insecure)."
        );
        let mut progress = download_progress("Downloading MySQL");
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let client = mysql_client(ctx);
        if !client.is_file() {
            bail!(
                "MySQL archive extracted but mysql client missing under {}",
                ctx.install_dir.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("MySQL {version} installed at {}", ctx.install_dir.display()),
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
        Ok(Some(resolve_mysql_url()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.join("bin")],
            vars: vec![(
                "MYSQL_HOME".to_string(),
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
            install_dir: tmp.join("mysql"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = MysqlPlugin;
        let c = ctx(tmp.path());
        assert_eq!(plugin.status(&c).state, InstallState::NotInstalled);
    }

    #[test]
    fn status_partial_when_dir_has_contents_without_client() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = MysqlPlugin;
        let c = ctx(tmp.path());
        std::fs::create_dir_all(&c.install_dir).unwrap();
        std::fs::write(c.install_dir.join("README"), b"x").unwrap();
        assert_eq!(plugin.status(&c).state, InstallState::Partial);
    }
}
