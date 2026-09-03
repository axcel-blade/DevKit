//! Rust plugin — install stable toolchain via rustup into the machine `dev` folder.

use crate::download::download_file;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

const MARKER: &str = ".devkit-rust";

fn rust_host_triple() -> Result<String> {
    let host = current_os();
    let arch = cpu_arch();
    let cpu = if arch == "aarch64" {
        "aarch64"
    } else {
        "x86_64"
    };
    match host {
        HostOS::Windows => Ok(format!("{cpu}-pc-windows-msvc")),
        HostOS::Linux => Ok(format!("{cpu}-unknown-linux-gnu")),
        HostOS::MacOS => Ok(format!("{cpu}-apple-darwin")),
        HostOS::Other => bail!("Rust is not supported on this OS: other"),
    }
}

fn resolve_rustup_init_url() -> Result<String> {
    let triple = rust_host_triple()?;
    let name = if is_windows() {
        "rustup-init.exe"
    } else {
        "rustup-init"
    };
    Ok(format!(
        "https://static.rust-lang.org/rustup/dist/{triple}/{name}"
    ))
}

fn cargo_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join("cargo")
        .join("bin")
        .join(if is_windows() { "cargo.exe" } else { "cargo" })
}

pub struct RustPlugin;

impl Plugin for RustPlugin {
    fn id(&self) -> &'static str {
        "rust"
    }

    fn name(&self) -> &'static str {
        "Rust"
    }

    fn description(&self) -> &'static str {
        "Install Rust stable via rustup into the machine dev folder \
         (sets CARGO_HOME / RUSTUP_HOME / PATH)."
    }

    fn status(&self, ctx: &InstallContext) -> crate::plugin::PluginStatus {
        binary_status(
            &cargo_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but cargo binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let url = resolve_rustup_init_url()?;
        println!("Rust (rustup stable)");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading rustup-init");
        let init_name = if is_windows() {
            "rustup-init.exe"
        } else {
            "rustup-init"
        };
        let init = download_file(
            &url,
            None,
            Some(init_name),
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&init)?.permissions();
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&init, perms)?;
        }

        let cargo_home = ctx.install_dir.join("cargo");
        let rustup_home = ctx.install_dir.join("rustup");
        std::fs::create_dir_all(&ctx.install_dir)?;

        println!("Running rustup-init ...");
        let output = Command::new(&init)
            .args([
                "-y",
                "--no-modify-path",
                "--default-toolchain",
                "stable",
                "--profile",
                "default",
            ])
            .env("CARGO_HOME", &cargo_home)
            .env("RUSTUP_HOME", &rustup_home)
            .output()
            .context("failed to launch rustup-init")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if !stderr.trim().is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            bail!(
                "rustup-init failed (exit {:?}): {}",
                output.status.code(),
                detail
            );
        }
        if !cargo_bin(ctx).is_file() {
            bail!(
                "rustup finished but cargo not found at {}",
                cargo_bin(ctx).display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), "stable\n")?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("Rust stable installed at {}", ctx.install_dir.display()),
        ))
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let cargo_home = ctx.install_dir.join("cargo");
        let rustup_home = ctx.install_dir.join("rustup");
        EnvSpec {
            paths: vec![cargo_home.join("bin")],
            vars: vec![
                (
                    "CARGO_HOME".to_string(),
                    cargo_home
                        .canonicalize()
                        .unwrap_or_else(|_| cargo_home.clone())
                        .display()
                        .to_string(),
                ),
                (
                    "RUSTUP_HOME".to_string(),
                    rustup_home
                        .canonicalize()
                        .unwrap_or_else(|_| rustup_home.clone())
                        .display()
                        .to_string(),
                ),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::InstallState;

    #[test]
    fn resolve_rustup_init_url_contains_triple() {
        let url = resolve_rustup_init_url().unwrap();
        assert!(url.starts_with("https://static.rust-lang.org/rustup/dist/"));
        assert!(url.ends_with("rustup-init.exe") || url.ends_with("rustup-init"));
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = RustPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("rust"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
