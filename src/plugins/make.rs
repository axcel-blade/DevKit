//! GNU Make plugin — ezwinports ZIP on Windows; system Make wrappers on macOS/Linux.
//!
//! Windows: GNU does not publish a portable Make binary, so DevKit downloads the
//! ezwinports `make-<ver>-without-guile-w32-bin.zip` (bin/make.exe) used by
//! common Windows package managers.
//!
//! macOS/Linux: there is no official portable archive, so DevKit registers thin
//! wrappers around an already-installed system `make` (or `gmake`).

use crate::download::install_archive_from_url;
use crate::platform::{current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::progress::download_progress;
use anyhow::{bail, Result};
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

const MAKE_VERSION: &str = "4.4.1";
const MARKER: &str = ".devkit-make";

/// `(url, version)` for the Windows ezwinports archive `install` extracts.
fn make_windows_archive() -> (String, String) {
    let url = format!(
        "https://downloads.sourceforge.net/project/ezwinports/make-{MAKE_VERSION}-without-guile-w32-bin.zip"
    );
    (url, MAKE_VERSION.to_string())
}

fn make_binary(ctx: &InstallContext) -> Option<PathBuf> {
    let name = if is_windows() { "make.exe" } else { "make" };
    let candidates = [
        ctx.install_dir.join("bin").join(name),
        ctx.install_dir.join(name),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

#[cfg(unix)]
fn write_unix_wrappers(install_dir: &Path, system_make: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let bin_dir = install_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    let mut names = vec!["make"];
    if which::which("gmake").is_ok() {
        names.push("gmake");
    }
    for name in names {
        let exe = which::which(name).unwrap_or_else(|_| system_make.to_path_buf());
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
        format!("system-wrapper\n{}\n", system_make.display()),
    )?;
    Ok(())
}

pub struct MakePlugin;

impl Plugin for MakePlugin {
    fn id(&self) -> &'static str {
        "make"
    }

    fn name(&self) -> &'static str {
        "GNU Make"
    }

    fn description(&self) -> &'static str {
        "Install GNU Make (ezwinports ZIP) on Windows. \
         On macOS/Linux, registers system make if already installed."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        if let Some(binary) = make_binary(ctx) {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail(binary.display().to_string());
        }
        if ctx.install_dir.join(MARKER).is_file() {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail("system make wrappers");
        }
        let has_contents = ctx.install_dir.is_dir()
            && std::fs::read_dir(&ctx.install_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_contents {
            return PluginStatus::new(InstallState::Partial, Some(ctx.install_dir.clone()))
                .with_detail("Install dir exists but make binary is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        match current_os() {
            HostOS::Windows => install_windows(ctx),
            HostOS::MacOS | HostOS::Linux => install_unix(ctx),
            HostOS::Other => bail!("GNU Make is not supported on this OS: other"),
        }
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    /// Pinned to the ezwinports archive `install` downloads on Windows.
    fn latest_version(&self, _ctx: &InstallContext) -> Result<Option<String>> {
        Ok(Some(MAKE_VERSION.to_string()))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let bin_dir = ctx.install_dir.join("bin");
        let paths = if bin_dir.is_dir() {
            vec![bin_dir]
        } else {
            vec![ctx.install_dir.clone()]
        };
        EnvSpec {
            paths,
            vars: vec![(
                "MAKE_HOME".to_string(),
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
    let (url, version) = make_windows_archive();
    println!("GNU Make {version}");
    println!("URL: {url}");
    let mut progress = download_progress("Downloading GNU Make");
    // Archive root is bin/make.exe (plus docs); do not strip a top-level folder.
    install_archive_from_url(
        &url,
        &ctx.install_dir,
        false,
        None,
        Some(&mut |d, t| progress.update(d, t)),
    )?;
    progress.done();
    if make_binary(ctx).is_none() {
        bail!(
            "GNU Make extracted but make.exe was not found under {}",
            ctx.install_dir.display()
        );
    }
    std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!(
            "GNU Make {version} installed at {}",
            ctx.install_dir.display()
        ),
    ))
}

fn install_unix(ctx: &InstallContext) -> Result<InstallResult> {
    let system_make = which::which("make")
        .or_else(|_| which::which("gmake"))
        .map_err(|_| {
            anyhow::anyhow!(
                "GNU Make does not ship a portable macOS/Linux archive for DevKit.\n\
                 Install Make with your platform tools, then re-run this command:\n\
                 \x20 macOS:          xcode-select --install\n\
                 \x20                 (or: brew install make)\n\
                 \x20 Debian/Ubuntu:  sudo apt install make\n\
                 \x20 Fedora:         sudo dnf install make\n\
                 \x20 Arch:           sudo pacman -S make\n\
                 Then:  devkit install make"
            )
        })?;
    std::fs::create_dir_all(&ctx.install_dir)?;
    #[cfg(unix)]
    write_unix_wrappers(&ctx.install_dir, &system_make)?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!("Registered system Make at {}", system_make.display()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(tmp: &std::path::Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("make"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn windows_archive_pins_ezwinports_zip() {
        let (url, version) = make_windows_archive();
        assert_eq!(version, "4.4.1");
        assert!(url.contains("make-4.4.1-without-guile-w32-bin.zip"));
        assert!(url.starts_with("https://downloads.sourceforge.net/"));
    }

    #[test]
    fn make_binary_none_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(make_binary(&ctx(tmp.path())).is_none());
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = MakePlugin;
        assert_eq!(
            plugin.status(&ctx(tmp.path())).state,
            InstallState::NotInstalled
        );
    }

    #[test]
    fn status_installed_when_binary_present() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = MakePlugin;
        let c = ctx(tmp.path());
        let bin = c.install_dir.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let name = if is_windows() { "make.exe" } else { "make" };
        std::fs::write(bin.join(name), b"stub").unwrap();
        assert_eq!(plugin.status(&c).state, InstallState::Installed);
    }

    #[test]
    fn latest_version_matches_pinned_archive() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = MakePlugin;
        assert_eq!(
            plugin.latest_version(&ctx(tmp.path())).unwrap().as_deref(),
            Some("4.4.1")
        );
    }

    #[test]
    fn uninstall_removes_install_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = MakePlugin;
        let c = ctx(tmp.path());
        std::fs::create_dir_all(&c.install_dir).unwrap();
        plugin.uninstall(&c).unwrap();
        assert!(!c.install_dir.exists());
    }
}
