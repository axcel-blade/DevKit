//! Shared helpers for Node.js package managers shipped as npm registry
//! tarballs (npm, Yarn). Each one is unpacked into its install dir and run
//! through small `node` wrapper scripts, so Node.js must be on PATH.

use crate::download::{download_json_value, install_archive_from_url};
use crate::platform::is_windows;
use crate::plugin::{InstallContext, InstallResult};
use crate::progress::download_progress;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// Return `(version, tarball_url)` for the `latest` dist-tag of `package`.
pub fn registry_latest(package: &str) -> Result<(String, String)> {
    let url = format!("https://registry.npmjs.org/{package}/latest");
    let data = download_json_value(&url)
        .with_context(|| format!("Failed to query npm registry for {package}"))?;
    let version = data
        .get("version")
        .and_then(|v| v.as_str())
        .context("npm registry response has no version")?
        .to_string();
    let tarball = data
        .get("dist")
        .and_then(|d| d.get("tarball"))
        .and_then(|v| v.as_str())
        .context("npm registry response has no dist.tarball")?
        .to_string();
    Ok((version, tarball))
}

/// Path of the launcher DevKit writes for `command` (`.cmd` on Windows).
pub fn wrapper_path(install_dir: &Path, command: &str) -> PathBuf {
    if is_windows() {
        install_dir.join(format!("{command}.cmd"))
    } else {
        install_dir.join(command)
    }
}

/// Write a launcher `command` that runs `node <install_dir>/<script>`.
fn write_wrapper(install_dir: &Path, command: &str, script: &str) -> Result<()> {
    if is_windows() {
        let script = script.replace('/', "\\");
        let content = format!("@echo off\r\nnode \"%~dp0{script}\" %*\r\n");
        std::fs::write(install_dir.join(format!("{command}.cmd")), content)?;
    } else {
        let path = install_dir.join(command);
        std::fs::write(
            &path,
            format!(
                "#!/usr/bin/env bash\nDIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nexec node \"$DIR/{script}\" \"$@\"\n"
            ),
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&path)?.permissions();
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&path, perms)?;
        }
    }
    Ok(())
}

/// Download the latest `package` tarball, unpack it into the install dir,
/// write one wrapper per `(command, script)` pair, and record the version
/// in the `.devkit-<plugin_id>` marker.
pub fn install_package(
    ctx: &InstallContext,
    plugin_id: &str,
    package: &str,
    display: &str,
    commands: &[(&str, &str)],
) -> Result<InstallResult> {
    if which::which("node").is_err() {
        println!("Warning: `node` was not found on PATH. {display} needs Node.js to run.");
    }
    let (version, url) = registry_latest(package)?;
    println!("{display} {version}");
    println!("URL: {url}");

    // Start clean so files removed upstream do not linger across updates.
    if ctx.install_dir.exists() {
        std::fs::remove_dir_all(&ctx.install_dir)?;
    }
    std::fs::create_dir_all(&ctx.install_dir)?;
    let mut progress = download_progress(&format!("Downloading {display}"));
    // Registry tarballs nest everything under `package/`; strip it.
    install_archive_from_url(
        &url,
        &ctx.install_dir,
        true,
        Some(&format!("{package}-{version}.tgz")),
        Some(&mut |d, t| progress.update(d, t)),
    )?;
    progress.done();

    for (command, script) in commands {
        if !ctx.install_dir.join(script).is_file() {
            bail!("{display} archive extracted but {script} is missing");
        }
        write_wrapper(&ctx.install_dir, command, script)?;
    }
    std::fs::write(
        ctx.install_dir.join(format!(".devkit-{plugin_id}")),
        format!("{version}\n"),
    )?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!(
            "{display} {version} installed at {}",
            ctx.install_dir.display()
        ),
    ))
}
