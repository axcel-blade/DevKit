//! PostgreSQL plugin — EDB Windows binaries; system wrappers on Unix.

use crate::download::install_archive_from_url;
use crate::platform::{current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Result};
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

// Pin a known EDB Windows binaries ZIP; bump with DevKit releases.
const PG_VERSION: &str = "16.8-1";
const MARKER: &str = ".devkit-postgresql";

fn windows_url() -> String {
    format!(
        "https://get.enterprisedb.com/postgresql/postgresql-{PG_VERSION}-windows-x64-binaries.zip"
    )
}

fn psql_name() -> &'static str {
    if is_windows() {
        "psql.exe"
    } else {
        "psql"
    }
}

fn pgsql_bin(ctx: &InstallContext) -> Option<PathBuf> {
    let candidates = [
        ctx.install_dir.join("bin").join(psql_name()),
        ctx.install_dir.join("pgsql").join("bin").join(psql_name()),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

#[cfg(unix)]
fn write_unix_wrappers(install_dir: &Path, system_psql: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let bin_dir = install_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    for name in ["psql", "pg_dump", "pg_restore", "createdb", "dropdb"] {
        let system = which::which(name).ok();
        if system.is_none() && name != "psql" {
            continue;
        }
        let exe = system.unwrap_or_else(|| system_psql.to_path_buf());
        let target = bin_dir.join(name);
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
        format!("system-wrapper\n{}\n", system_psql.display()),
    )?;
    Ok(())
}

pub struct PostgresqlPlugin;

impl Plugin for PostgresqlPlugin {
    fn id(&self) -> &'static str {
        "postgresql"
    }

    fn name(&self) -> &'static str {
        "PostgreSQL"
    }

    fn description(&self) -> &'static str {
        "Download PostgreSQL 16.8-1 Windows binaries from EDB. \
         On macOS/Linux, registers system PostgreSQL clients if installed."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        if let Some(binary) = pgsql_bin(ctx) {
            return binary_status(
                &binary,
                &ctx.install_dir,
                "Install dir exists but psql is missing",
            );
        }
        if ctx.install_dir.join(MARKER).is_file() {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail("system postgresql wrappers");
        }
        binary_status(
            &ctx.install_dir.join("bin").join("psql"),
            &ctx.install_dir,
            "Install dir exists but psql is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        match current_os() {
            HostOS::Windows => install_windows(ctx),
            HostOS::MacOS | HostOS::Linux => install_unix(ctx),
            HostOS::Other => bail!("PostgreSQL is not supported on this OS: other"),
        }
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let binary = pgsql_bin(ctx);
        let mut paths: Vec<PathBuf> = Vec::new();
        if let Some(b) = &binary {
            if let Some(parent) = b.parent() {
                paths.push(parent.to_path_buf());
            }
        }
        let wrapper = ctx.install_dir.join("bin");
        if wrapper.is_dir() && !paths.contains(&wrapper) {
            paths.push(wrapper);
        }
        EnvSpec {
            paths,
            vars: vec![(
                "POSTGRESQL_HOME".to_string(),
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
    let url = windows_url();
    println!("PostgreSQL {PG_VERSION} (Windows x64 binaries)");
    println!("URL: {url}");
    let mut progress = download_progress("Downloading PostgreSQL");
    install_archive_from_url(
        &url,
        &ctx.install_dir,
        true,
        None,
        Some(&mut |d, t| progress.update(d, t)),
    )?;
    progress.done();
    if pgsql_bin(ctx).is_none() {
        bail!(
            "PostgreSQL extracted but psql missing under {}",
            ctx.install_dir.display()
        );
    }
    std::fs::write(ctx.install_dir.join(MARKER), format!("{PG_VERSION}\n"))?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!(
            "PostgreSQL {PG_VERSION} installed at {}",
            ctx.install_dir.display()
        ),
    ))
}

fn install_unix(ctx: &InstallContext) -> Result<InstallResult> {
    let system = which::which("psql").map_err(|_| {
        anyhow::anyhow!(
            "PostgreSQL has no portable Unix archive in DevKit.\n\
             Install clients with your package manager, then re-run:\n\
             \x20 Debian/Ubuntu:  sudo apt install postgresql-client\n\
             \x20 Fedora:         sudo dnf install postgresql\n\
             \x20 macOS:          brew install libpq && brew link --force libpq\n\
             Then:  devkit install postgresql"
        )
    })?;
    std::fs::create_dir_all(&ctx.install_dir)?;
    #[cfg(unix)]
    write_unix_wrappers(&ctx.install_dir, &system)?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!(
            "Registered system PostgreSQL client at {}",
            system.display()
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pgsql_bin_none_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = InstallContext {
            install_dir: tmp.path().join("postgresql"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert!(pgsql_bin(&ctx).is_none());
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = PostgresqlPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("postgresql"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
