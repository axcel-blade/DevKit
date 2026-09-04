//! Python plugin — portable CPython from python-build-standalone (all platforms).

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin};
use crate::plugin_utils::binary_status;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

// Pin a known-good CPython + python-build-standalone release pair.
const PYTHON_VERSION: &str = "3.12.13";
const PBS_TAG: &str = "20260728";
const MARKER: &str = ".devkit-python";

fn pbs_triple() -> Result<String> {
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
        HostOS::Other => bail!("Python is not supported on this OS: other"),
    }
}

/// Return `(download_url, version)` for this OS/arch.
fn resolve_python_download() -> Result<(String, String)> {
    let arch = cpu_arch();
    if arch != "x64" && arch != "aarch64" {
        bail!("Unsupported arch for Python: {arch}");
    }
    let triple = pbs_triple()?;
    // '+' must be URL-encoded in the asset name.
    let asset = format!("cpython-{PYTHON_VERSION}%2B{PBS_TAG}-{triple}-install_only.tar.gz");
    let url = format!(
        "https://github.com/astral-sh/python-build-standalone/releases/download/{PBS_TAG}/{asset}"
    );
    Ok((url, PYTHON_VERSION.to_string()))
}

fn python_bin(ctx: &InstallContext) -> PathBuf {
    if is_windows() {
        for candidate in [
            ctx.install_dir.join("python.exe"),
            ctx.install_dir.join("bin").join("python.exe"),
        ] {
            if candidate.is_file() {
                return candidate;
            }
        }
        return ctx.install_dir.join("python.exe");
    }
    for candidate in [
        ctx.install_dir.join("bin").join("python3"),
        ctx.install_dir.join("bin").join("python"),
    ] {
        if candidate.is_file() {
            return candidate;
        }
    }
    ctx.install_dir.join("bin").join("python3")
}

pub struct PythonPlugin;

impl Plugin for PythonPlugin {
    fn id(&self) -> &'static str {
        "python"
    }

    fn name(&self) -> &'static str {
        "Python"
    }

    fn description(&self) -> &'static str {
        "Download portable Python 3.12.13 (python-build-standalone) and set PYTHON_HOME / PATH."
    }

    fn status(&self, ctx: &InstallContext) -> crate::plugin::PluginStatus {
        binary_status(
            &python_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but python binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_python_download()?;
        println!("Python {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Python");
        // Archive root is python/; strip so bin/ lands in install_dir.
        install_archive_from_url(
            &url,
            &ctx.install_dir,
            true,
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let binary = python_bin(ctx);
        if !binary.is_file() {
            bail!(
                "Python extracted but binary not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Python {version} installed at {}",
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
        let home = ctx
            .install_dir
            .canonicalize()
            .unwrap_or_else(|_| ctx.install_dir.clone())
            .display()
            .to_string();
        let binary = python_bin(ctx);
        let bin_dir = if binary.is_file() {
            binary.parent().unwrap_or(&ctx.install_dir).to_path_buf()
        } else {
            ctx.install_dir.join("bin")
        };
        EnvSpec {
            paths: vec![bin_dir],
            vars: vec![("PYTHON_HOME".to_string(), home)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::InstallState;

    #[test]
    fn resolve_python_download_builds_expected_url() {
        let (url, version) = resolve_python_download().unwrap();
        assert_eq!(version, PYTHON_VERSION);
        assert!(url.contains("cpython-3.12.13%2B20260728-"));
        assert!(url.starts_with(
            "https://github.com/astral-sh/python-build-standalone/releases/download/"
        ));
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = PythonPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("python"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
