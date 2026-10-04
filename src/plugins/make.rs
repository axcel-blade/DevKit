//! GNU Make plugin — Chocolatey on Windows; system Make wrappers on macOS/Linux.
//!
//! Windows: GNU does not publish a portable Make binary. DevKit runs
//! `choco install make`, then copies `make.exe` (and sibling DLLs) out of the
//! Chocolatey lib folder into the DevKit install directory.
//!
//! macOS/Linux: there is no official portable archive, so DevKit registers thin
//! wrappers around an already-installed system `make` (or `gmake`).

use crate::platform::{current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use anyhow::{bail, Context, Result};
#[cfg(unix)]
use std::path::Path;
use std::path::PathBuf;

const MAKE_VERSION: &str = "4.4.1";
const MARKER: &str = ".devkit-make";

/// Non-interactive form of `choco install make`, pinned to [`MAKE_VERSION`].
fn choco_make_args() -> &'static [&'static str] {
    &["install", "make", "--version", MAKE_VERSION, "-y"]
}

fn chocolatey_make_bin(root: &std::path::Path) -> PathBuf {
    root.join("lib")
        .join("make")
        .join("tools")
        .join("install")
        .join("bin")
}

fn chocolatey_root() -> PathBuf {
    if let Some(root) = std::env::var_os("ChocolateyInstall") {
        return PathBuf::from(root);
    }
    if let Ok(choco) = which::which("choco") {
        if let Some(root) = choco.parent().and_then(|bin| bin.parent()) {
            return root.to_path_buf();
        }
    }
    PathBuf::from(r"C:\ProgramData\chocolatey")
}

/// Copy `make.exe` and sibling files from Chocolatey's package `bin` into `install_dir/bin`.
fn stage_make_bin(src_bin: &std::path::Path, install_dir: &std::path::Path) -> Result<PathBuf> {
    let dest = install_dir.join("bin");
    std::fs::create_dir_all(&dest)?;
    let mut copied_exe = None;
    for entry in std::fs::read_dir(src_bin)
        .with_context(|| format!("Chocolatey Make bin not found at {}", src_bin.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let to = dest.join(entry.file_name());
        std::fs::copy(entry.path(), &to)?;
        if entry.file_name() == "make.exe" {
            copied_exe = Some(to);
        }
    }
    copied_exe.ok_or_else(|| {
        anyhow::anyhow!(
            "choco install make finished but make.exe was not found under {}",
            src_bin.display()
        )
    })
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
        "Install GNU Make with Chocolatey (`choco install make`) on Windows. \
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

    /// Pinned to the Chocolatey `make` package `install` requests on Windows.
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
    use std::process::Command;

    let choco = which::which("choco").map_err(|_| {
        anyhow::anyhow!(
            "GNU Make on Windows is installed with Chocolatey.\n\
             Install Chocolatey, then re-run this command.\n\
             DevKit runs:  choco install make"
        )
    })?;
    let args = choco_make_args();
    println!("Running: choco {}", args.join(" "));
    let status = Command::new(&choco)
        .args(args)
        .status()
        .context("failed to launch choco")?;
    let code = status.code();
    // 3010: success, reboot requested.
    if code != Some(0) && code != Some(3010) {
        bail!("choco install make exited with {code:?}");
    }
    let src_bin = chocolatey_make_bin(&chocolatey_root());
    let dest_exe = stage_make_bin(&src_bin, &ctx.install_dir)?;
    std::fs::write(ctx.install_dir.join(MARKER), format!("{MAKE_VERSION}\n"))?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!(
            "GNU Make {MAKE_VERSION} installed at {}",
            dest_exe.display()
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
    fn choco_install_args_pin_make() {
        assert_eq!(
            choco_make_args(),
            ["install", "make", "--version", "4.4.1", "-y"]
        );
    }

    #[test]
    fn chocolatey_make_bin_uses_package_layout() {
        let root = std::path::Path::new(r"C:\ProgramData\chocolatey");
        assert_eq!(
            chocolatey_make_bin(root),
            root.join("lib")
                .join("make")
                .join("tools")
                .join("install")
                .join("bin")
        );
    }

    #[test]
    fn stage_make_bin_copies_exe_and_siblings() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src-bin");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(src.join("make.exe"), b"make").unwrap();
        std::fs::write(src.join("libintl-8.dll"), b"dll").unwrap();
        let install = tmp.path().join("make");
        let dest = stage_make_bin(&src, &install).unwrap();
        assert_eq!(dest, install.join("bin").join("make.exe"));
        assert_eq!(std::fs::read(dest).unwrap(), b"make");
        assert_eq!(
            std::fs::read(install.join("bin").join("libintl-8.dll")).unwrap(),
            b"dll"
        );
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
    fn latest_version_matches_chocolatey_package() {
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
