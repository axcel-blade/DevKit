//! PHP plugin — Windows ZIP from downloads.php.net; system PHP wrappers on Unix.
//!
//! Pairs with the Composer plugin. On Windows, installs the latest NTS x64 build
//! (suitable for CLI / Composer). On macOS/Linux, registers wrappers around system PHP.

use crate::download::{download_json, install_archive_from_url};
use crate::platform::{current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::progress::download_progress;
use anyhow::{bail, Result};
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

const RELEASES_JSON: &str = "https://downloads.php.net/~windows/releases/releases.json";
const RELEASES_BASE: &str = "https://downloads.php.net/~windows/releases/";
const MARKER: &str = ".devkit-php";

/// Pick the newest NTS x64 Windows ZIP from the official releases index.
fn resolve_php_windows_url() -> Result<(String, String)> {
    let meta = download_json(RELEASES_JSON)?;
    let mut candidates: Vec<(Vec<u32>, String, String)> = Vec::new();

    for info in meta.values() {
        let info = match info.as_object() {
            Some(o) => o,
            None => continue,
        };
        let version = info
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Prefer NTS (non-thread-safe) CLI builds; fall back to TS.
        let mut build = None;
        for (key, value) in info.iter() {
            if key.starts_with("nts-") && key.ends_with("-x64") {
                build = Some(value);
                break;
            }
        }
        if build.is_none() {
            for (key, value) in info.iter() {
                if key.starts_with("ts-") && key.ends_with("-x64") {
                    build = Some(value);
                    break;
                }
            }
        }
        let build = match build.and_then(|b| b.as_object()) {
            Some(b) => b,
            None => continue,
        };
        let zip_info = match build.get("zip").and_then(|v| v.as_object()) {
            Some(z) => z,
            None => continue,
        };
        let path = match zip_info.get("path").and_then(|v| v.as_str()) {
            Some(p) if !p.is_empty() => p.to_string(),
            _ => continue,
        };

        let mut ver_tuple = Vec::new();
        let mut ok = true;
        for part in version.split('.') {
            match part.parse::<u32>() {
                Ok(n) => ver_tuple.push(n),
                Err(_) => {
                    ok = false;
                    break;
                }
            }
        }
        if !ok {
            continue;
        }
        candidates.push((ver_tuple, version, path));
    }

    if candidates.is_empty() {
        bail!("No suitable Windows PHP ZIP found in releases.json");
    }
    candidates.sort_by(|a, b| b.0.cmp(&a.0));
    let (_, version, path) = candidates.into_iter().next().unwrap();
    Ok((format!("{RELEASES_BASE}{path}"), version))
}

fn php_binary(ctx: &InstallContext) -> Option<PathBuf> {
    if is_windows() {
        let exe = ctx.install_dir.join("php.exe");
        return if exe.is_file() { Some(exe) } else { None };
    }
    for rel in ["bin/php", "php"] {
        let path = ctx.install_dir.join(rel);
        if path.is_file() {
            return Some(path);
        }
    }
    None
}

/// Copy php.ini-development to php.ini when missing (Windows builds).
fn ensure_php_ini(install_dir: &std::path::Path) -> Result<()> {
    let ini = install_dir.join("php.ini");
    if ini.is_file() {
        return Ok(());
    }
    for name in ["php.ini-development", "php.ini-production"] {
        let src = install_dir.join(name);
        if src.is_file() {
            std::fs::copy(&src, &ini)?;
            return Ok(());
        }
    }
    Ok(())
}

#[cfg(unix)]
fn write_unix_wrappers(install_dir: &Path, system_php: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let bin_dir = install_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    for name in ["php", "php-cgi", "phpdbg"] {
        let system = which::which(name).ok();
        if system.is_none() && name != "php" {
            continue;
        }
        let exe = system.unwrap_or_else(|| system_php.to_path_buf());
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
        format!("system-wrapper\n{}\n", system_php.display()),
    )?;
    Ok(())
}

pub struct PhpPlugin;

impl Plugin for PhpPlugin {
    fn id(&self) -> &'static str {
        "php"
    }

    fn name(&self) -> &'static str {
        "PHP"
    }

    fn description(&self) -> &'static str {
        "Install latest PHP NTS x64 ZIP on Windows. \
         On macOS/Linux, registers system PHP if already installed."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        if let Some(binary) = php_binary(ctx) {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail(binary.display().to_string());
        }
        if ctx.install_dir.join(MARKER).is_file() {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail("system php wrappers");
        }
        let has_contents = ctx.install_dir.is_dir()
            && std::fs::read_dir(&ctx.install_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_contents {
            return PluginStatus::new(InstallState::Partial, Some(ctx.install_dir.clone()))
                .with_detail("Install dir exists but php binary is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        match current_os() {
            HostOS::Windows => install_windows(ctx),
            HostOS::MacOS | HostOS::Linux => install_unix(ctx),
            HostOS::Other => bail!("PHP is not supported on this OS: other"),
        }
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    /// Same resolver `install` uses, so the string matches the marker it writes.
    fn latest_version(&self, _ctx: &InstallContext) -> Result<Option<String>> {
        Ok(Some(resolve_php_windows_url()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let mut paths = vec![ctx.install_dir.clone()];
        let bin_dir = ctx.install_dir.join("bin");
        if bin_dir.is_dir() {
            paths.push(bin_dir);
        }
        EnvSpec {
            paths,
            vars: vec![(
                "PHP_HOME".to_string(),
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
    let (url, version) = resolve_php_windows_url()?;
    println!("PHP {version} (Windows NTS/TS x64)");
    println!("URL: {url}");
    let mut progress = download_progress("Downloading PHP");
    install_archive_from_url(
        &url,
        &ctx.install_dir,
        false,
        None,
        Some(&mut |d, t| progress.update(d, t)),
    )?;
    progress.done();
    let binary = php_binary(ctx);
    if binary.is_none() {
        bail!(
            "PHP ZIP extracted but php.exe missing under {}",
            ctx.install_dir.display()
        );
    }
    ensure_php_ini(&ctx.install_dir)?;
    std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!("PHP {version} installed at {}", ctx.install_dir.display()),
    ))
}

fn install_unix(ctx: &InstallContext) -> Result<InstallResult> {
    let system_php = which::which("php").map_err(|_| {
        anyhow::anyhow!(
            "PHP does not ship a portable macOS/Linux archive for DevKit.\n\
             Install PHP with your platform tools, then re-run this command:\n\
             \x20 macOS:          brew install php\n\
             \x20 Debian/Ubuntu:  sudo apt install php-cli\n\
             \x20 Fedora:         sudo dnf install php-cli\n\
             Then:  devkit install php"
        )
    })?;
    std::fs::create_dir_all(&ctx.install_dir)?;
    #[cfg(unix)]
    write_unix_wrappers(&ctx.install_dir, &system_php)?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!("Registered system PHP at {}", system_php.display()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn php_binary_none_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let ctx = InstallContext {
            install_dir: tmp.path().join("php"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert!(php_binary(&ctx).is_none());
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = PhpPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("php"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
