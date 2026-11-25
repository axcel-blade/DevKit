//! NASM plugin — official builds from nasm.us on Windows/macOS; system wrappers on Linux.
//!
//! nasm.us publishes `win64` / `win32` and `macosx` ZIPs per release. There is
//! no portable Linux build, so DevKit wraps an already-installed `nasm` there.

use crate::download::install_archive_from_url;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::plugin_utils::read_text_url;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

const MARKER: &str = ".devkit-nasm";
const RELEASES: &str = "https://www.nasm.us/pub/nasm/releasebuilds/";

/// Newest stable `X.Y[.Z]/` entry in the release listing (skips rc builds).
fn parse_latest_version(html: &str) -> Result<String> {
    let re = regex::Regex::new(r#"href="(\d+\.\d+(?:\.\d+)?)/""#).unwrap();
    let key = |v: &str| -> Vec<u32> { v.split('.').map(|p| p.parse().unwrap_or(0)).collect() };
    re.captures_iter(html)
        .map(|c| c[1].to_string())
        .max_by_key(|v| key(v))
        .ok_or_else(|| anyhow::anyhow!("Could not parse NASM release listing"))
}

fn latest_version() -> Result<String> {
    parse_latest_version(&read_text_url(RELEASES)?)
}

fn download_url(version: &str) -> Result<String> {
    let flavor = match current_os() {
        // No ARM64 build; win64 runs under emulation on Windows on ARM.
        HostOS::Windows if cpu_arch() == "x86" => "win32",
        HostOS::Windows => "win64",
        HostOS::MacOS => "macosx",
        _ => bail!("NASM has no official archive for this OS"),
    };
    Ok(format!(
        "{RELEASES}{version}/{flavor}/nasm-{version}-{flavor}.zip"
    ))
}

fn nasm_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir
        .join(if is_windows() { "nasm.exe" } else { "nasm" })
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

pub struct NasmPlugin;

impl Plugin for NasmPlugin {
    fn id(&self) -> &'static str {
        "nasm"
    }

    fn name(&self) -> &'static str {
        "NASM"
    }

    fn description(&self) -> &'static str {
        "Download the latest NASM assembler and add it to PATH. \
         On Linux, registers system NASM if already installed."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        let binary = nasm_bin(ctx);
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
                .with_detail("Install dir exists but nasm binary is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        match current_os() {
            HostOS::Windows | HostOS::MacOS => install_archive(ctx),
            HostOS::Linux => install_linux(ctx),
            HostOS::Other => bail!("NASM is not supported on this OS: other"),
        }
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn latest_version(&self, _ctx: &InstallContext) -> Result<Option<String>> {
        if current_os() == HostOS::Linux {
            return Ok(None);
        }
        Ok(Some(latest_version()?))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.clone()],
            vars: vec![(
                "NASM_HOME".to_string(),
                ctx.install_dir
                    .canonicalize()
                    .unwrap_or_else(|_| ctx.install_dir.clone())
                    .display()
                    .to_string(),
            )],
        }
    }
}

fn install_archive(ctx: &InstallContext) -> Result<InstallResult> {
    let version = latest_version()?;
    let url = download_url(&version)?;
    println!("NASM {version}");
    println!("URL: {url}");
    let mut progress = download_progress("Downloading NASM");
    // ZIP nests nasm(.exe) / ndisasm(.exe) under nasm-<ver>/.
    install_archive_from_url(
        &url,
        &ctx.install_dir,
        true,
        None,
        Some(&mut |d, t| progress.update(d, t)),
    )?;
    progress.done();
    let binary = nasm_bin(ctx);
    if !binary.is_file() {
        bail!(
            "NASM extracted but binary not found at {}",
            binary.display()
        );
    }
    if !is_windows() {
        for name in ["nasm", "ndisasm"] {
            let path = ctx.install_dir.join(name);
            if path.is_file() {
                make_executable(&path)?;
            }
        }
    }
    std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!("NASM {version} installed at {}", ctx.install_dir.display()),
    ))
}

fn install_linux(ctx: &InstallContext) -> Result<InstallResult> {
    let system_nasm = which::which("nasm").map_err(|_| {
        anyhow::anyhow!(
            "NASM does not ship a portable Linux archive for DevKit.\n\
             Install NASM with your package manager, then re-run this command:\n\
             \x20 Debian/Ubuntu:  sudo apt install nasm\n\
             \x20 Fedora:         sudo dnf install nasm\n\
             \x20 Arch:           sudo pacman -S nasm\n\
             Then:  devkit install nasm"
        )
    })?;
    std::fs::create_dir_all(&ctx.install_dir)?;
    for name in ["nasm", "ndisasm"] {
        let Ok(exe) = which::which(name) else {
            continue;
        };
        let target = ctx.install_dir.join(name);
        std::fs::write(
            &target,
            format!("#!/usr/bin/env bash\nexec \"{}\" \"$@\"\n", exe.display()),
        )?;
        make_executable(&target)?;
    }
    std::fs::write(
        ctx.install_dir.join(MARKER),
        format!("system-wrapper\n{}\n", system_nasm.display()),
    )?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!("Registered system NASM at {}", system_nasm.display()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = InstallContext {
            install_dir: tmp.path().join("nasm"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(NasmPlugin.status(&ctx).state, InstallState::NotInstalled);
    }

    #[test]
    fn parse_latest_version_skips_rc() {
        let html = r#"<a href="2.16.03/">2.16.03/</a> <a href="3.01/">3.01/</a>
                      <a href="3.02rc1/">3.02rc1/</a> <a href="2.9.1/">2.9.1/</a>"#;
        assert_eq!(parse_latest_version(html).unwrap(), "3.01");
    }
}
