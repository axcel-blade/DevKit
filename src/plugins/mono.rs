//! Mono runtime / MDK plugin.

use crate::download::download_file;
use crate::installers::{extract_msi_admin, extract_pkg};
use crate::platform::{current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::plugin_utils::find_files_named;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

const MONO_VERSION: &str = "6.12.0.206";
const MARKER: &str = ".devkit-mono";

fn mono_windows_url() -> String {
    format!(
        "https://download.mono-project.com/archive/6.12.0/windows-installer/mono-{MONO_VERSION}-x64-0.msi"
    )
}

fn mono_macos_url() -> String {
    format!(
        "https://download.mono-project.com/archive/6.12.0/macos-10-universal/MonoFramework-MDK-{MONO_VERSION}.macos10.xamarin.universal.pkg"
    )
}

/// Locate a directory that contains the `mono` executable.
fn find_mono_bin_dir(root: &Path) -> Option<PathBuf> {
    for name in ["mono.exe", "mono"] {
        let files = find_files_named(root, name);
        if !files.is_empty() {
            // Prefer .../bin/mono over deeper copies (find_files_named sorts shallowest-first).
            return files
                .into_iter()
                .next()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()));
        }
    }
    // Known Mono.framework layout on macOS.
    let commands = root
        .join("Library")
        .join("Frameworks")
        .join("Mono.framework")
        .join("Commands");
    if commands.join("mono").is_file() {
        return Some(commands);
    }
    None
}

fn mono_binary(ctx: &InstallContext) -> Option<PathBuf> {
    let bin_dir = find_mono_bin_dir(&ctx.install_dir)?;
    if is_windows() {
        let exe = bin_dir.join("mono.exe");
        if exe.is_file() {
            Some(exe)
        } else {
            None
        }
    } else {
        let mono = bin_dir.join("mono");
        if mono.is_file() {
            Some(mono)
        } else {
            None
        }
    }
}

#[cfg(unix)]
fn write_linux_wrappers(install_dir: &Path, system_mono: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let bin_dir = install_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)?;
    for name in ["mono", "mcs", "csharp"] {
        let system = which::which(name).ok();
        let target = bin_dir.join(name);
        let exe = match (&system, name) {
            (Some(p), _) => p.clone(),
            (None, "mono") => system_mono.to_path_buf(),
            (None, _) => continue,
        };
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
        format!("system-wrapper\n{}\n", system_mono.display()),
    )?;
    Ok(())
}

pub struct MonoPlugin;

impl Plugin for MonoPlugin {
    fn id(&self) -> &'static str {
        "mono"
    }

    fn name(&self) -> &'static str {
        "Mono"
    }

    fn description(&self) -> &'static str {
        "Install Mono 6.12.0.206 (Windows MSI / macOS PKG). \
         On Linux, registers system Mono if already installed via apt/dnf."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        if let Some(binary) = mono_binary(ctx) {
            if binary.is_file() {
                return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                    .with_detail(binary.display().to_string());
            }
        }
        if ctx.install_dir.join(MARKER).is_file() {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail("system mono wrappers");
        }
        let has_contents = ctx.install_dir.is_dir()
            && std::fs::read_dir(&ctx.install_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_contents {
            return PluginStatus::new(InstallState::Partial, Some(ctx.install_dir.clone()))
                .with_detail("Install dir exists but mono binary is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        match current_os() {
            HostOS::Windows => install_windows(ctx),
            HostOS::MacOS => install_macos(ctx),
            HostOS::Linux => install_linux(ctx),
            HostOS::Other => bail!("Mono is not supported on this OS: other"),
        }
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let mut paths = Vec::new();
        if let Some(bin_dir) = find_mono_bin_dir(&ctx.install_dir) {
            paths.push(bin_dir);
        }
        // Fallback for Linux wrappers.
        let wrapper_bin = ctx.install_dir.join("bin");
        if wrapper_bin.is_dir() && !paths.contains(&wrapper_bin) {
            paths.push(wrapper_bin);
        }
        EnvSpec {
            paths,
            vars: vec![(
                "MONO_HOME".to_string(),
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
    println!("Downloading Mono {MONO_VERSION} (Windows x64 MSI) ...");
    let mut progress = download_progress("Downloading Mono");
    let msi = download_file(
        &mono_windows_url(),
        None,
        Some(&format!("mono-{MONO_VERSION}-x64.msi")),
        Some(&mut |d, t| progress.update(d, t)),
    )?;
    progress.done();
    println!("Extracting MSI into {} ...", ctx.install_dir.display());
    extract_msi_admin(&msi, &ctx.install_dir)?;
    let binary = mono_binary(ctx);
    if binary.is_none() {
        bail!(
            "Mono MSI extracted but mono.exe was not found under {}",
            ctx.install_dir.display()
        );
    }
    std::fs::write(ctx.install_dir.join(MARKER), format!("{MONO_VERSION}\n"))?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!(
            "Mono {MONO_VERSION} installed at {}",
            ctx.install_dir.display()
        ),
    ))
}

fn install_macos(ctx: &InstallContext) -> Result<InstallResult> {
    println!("Downloading Mono {MONO_VERSION} (macOS PKG) ...");
    let mut progress = download_progress("Downloading Mono");
    let pkg = download_file(
        &mono_macos_url(),
        None,
        Some(&format!("MonoFramework-MDK-{MONO_VERSION}.pkg")),
        Some(&mut |d, t| progress.update(d, t)),
    )?;
    progress.done();
    println!("Extracting PKG into {} ...", ctx.install_dir.display());
    extract_pkg(&pkg, &ctx.install_dir)?;
    let binary = mono_binary(ctx);
    if binary.is_none() {
        bail!(
            "Mono PKG extracted but mono was not found under {}",
            ctx.install_dir.display()
        );
    }
    std::fs::write(ctx.install_dir.join(MARKER), format!("{MONO_VERSION}\n"))?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!(
            "Mono {MONO_VERSION} installed at {}",
            ctx.install_dir.display()
        ),
    ))
}

fn install_linux(ctx: &InstallContext) -> Result<InstallResult> {
    let system_mono = which::which("mono").map_err(|_| {
        anyhow::anyhow!(
            "Mono does not ship a portable Linux archive for DevKit.\n\
             Install it with your package manager, then re-run this command:\n\
             \x20 Debian/Ubuntu:  sudo apt install mono-complete\n\
             \x20 Fedora:         sudo dnf install mono-complete\n\
             \x20 Arch:           sudo pacman -S mono\n\
             Then:  devkit install mono"
        )
    })?;
    std::fs::create_dir_all(&ctx.install_dir)?;
    #[cfg(unix)]
    write_linux_wrappers(&ctx.install_dir, &system_mono)?;
    Ok(InstallResult::new(
        ctx.install_dir.clone(),
        format!("Registered system Mono at {}", system_mono.display()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_mono_bin_dir_none_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(find_mono_bin_dir(tmp.path()).is_none());
    }

    #[test]
    fn find_mono_bin_dir_prefers_shallowest() {
        let tmp = tempfile::tempdir().unwrap();
        let shallow = tmp.path().join("bin");
        let deep = tmp.path().join("a").join("b").join("bin");
        std::fs::create_dir_all(&shallow).unwrap();
        std::fs::create_dir_all(&deep).unwrap();
        std::fs::write(shallow.join("mono"), b"stub").unwrap();
        std::fs::write(deep.join("mono"), b"stub").unwrap();
        let found = find_mono_bin_dir(tmp.path()).unwrap();
        assert_eq!(found, shallow);
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = MonoPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("mono"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }

    /// Windows CI smoke: download the real Mono MSI, admin-extract it, verify
    /// the mono.exe layout. Set `DEVKIT_MONO_MSI_SMOKE=1` to run — mirrors the
    /// Python `-m smoke` marker gate in `tests/test_mono_msi_smoke.py`.
    ///
    /// Uses a short path under `RUNNER_TEMP` (or `TEMP`) because long CI temp
    /// paths under AppData have caused msiexec exit 1603 on GitHub Actions.
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn mono_msi_admin_extract_layout() {
        if std::env::var("DEVKIT_MONO_MSI_SMOKE").as_deref() != Ok("1") {
            eprintln!("skipping: set DEVKIT_MONO_MSI_SMOKE=1 to run");
            return;
        }

        let root = std::env::var("RUNNER_TEMP")
            .or_else(|_| std::env::var("TEMP"))
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir());
        let work = root.join("devkit-mono-msi-smoke");
        if work.exists() {
            std::fs::remove_dir_all(&work).unwrap();
        }
        std::fs::create_dir_all(&work).unwrap();

        let mut progress = download_progress("Downloading Mono MSI (smoke)");
        let msi = crate::download::download_file(
            &mono_windows_url(),
            None,
            Some(&format!("mono-{MONO_VERSION}-x64.msi")),
            Some(&mut |d, t| progress.update(d, t)),
        )
        .unwrap();
        progress.done();
        assert!(msi.is_file());
        assert!(std::fs::metadata(&msi).unwrap().len() > 1_000_000);

        let dest = work.join("extract");
        extract_msi_admin(&msi, &dest).unwrap();
        let bin_dir = find_mono_bin_dir(&dest);
        assert!(
            bin_dir.is_some(),
            "mono.exe not found under {}",
            dest.display()
        );
        assert!(bin_dir.unwrap().join("mono.exe").is_file());
    }
}
