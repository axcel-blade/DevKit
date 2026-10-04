//! Chocolatey plugin — portable CLI into the DevKit folder (Windows only).
//!
//! The official `choco install` path locks `C:\ProgramData\chocolatey` and needs
//! an elevated shell. DevKit downloads the Chocolatey nupkg, unpacks
//! `tools/chocolateyInstall` into the DevKit directory, and sets the user
//! `ChocolateyInstall` variable so later `choco` commands use that folder.

use crate::download::{download_file, extract_zip};
use crate::platform::{current_os, HostOS};
use crate::plugin::{
    cannot_install_message, EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus,
};
use crate::plugin_utils::{binary_status, read_text_url};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

const MARKER: &str = ".devkit-chocolatey";
const FEED_URL: &str = "https://community.chocolatey.org/api/v2/Packages()?$filter=((Id%20eq%20%27chocolatey%27)%20and%20(not%20IsPrerelease))%20and%20IsLatestVersion";

fn parse_chocolatey_version(xml: &str) -> Result<String> {
    let re = regex::Regex::new(r"<d:Version>([^<]+)</d:Version>").unwrap();
    let version = re
        .captures(xml)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .filter(|v| !v.is_empty());
    version.ok_or_else(|| anyhow::anyhow!("Could not parse the Chocolatey package version"))
}

fn chocolatey_package_url(version: &str) -> String {
    format!("https://community.chocolatey.org/api/v2/package/chocolatey/{version}")
}

fn resolve_chocolatey_download() -> Result<(String, String)> {
    let xml = read_text_url(FEED_URL)?;
    let version = parse_chocolatey_version(&xml)?;
    Ok((chocolatey_package_url(&version), version))
}

fn choco_binary(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir.join("bin").join("choco.exe")
}

fn copy_tree(src: &Path, dest: &Path) -> Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let to = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &to)?;
        } else {
            std::fs::copy(entry.path(), &to)?;
        }
    }
    Ok(())
}

/// Copy the nupkg's `tools/chocolateyInstall` tree into `install_dir` and add `bin/choco.exe`.
fn stage_chocolatey(extracted: &Path, install_dir: &Path, nupkg: &Path) -> Result<PathBuf> {
    let src = extracted.join("tools").join("chocolateyInstall");
    if !src.join("choco.exe").is_file() {
        bail!(
            "Chocolatey package did not contain {}",
            src.join("choco.exe").display()
        );
    }
    if install_dir.exists() {
        std::fs::remove_dir_all(install_dir)?;
    }
    copy_tree(&src, install_dir)?;

    let bin = install_dir.join("bin");
    std::fs::create_dir_all(&bin)?;
    for name in ["choco.exe", "RefreshEnv.cmd"] {
        let from = install_dir.join("redirects").join(name);
        if from.is_file() {
            std::fs::copy(&from, bin.join(name))?;
        }
    }
    let bin_exe = bin.join("choco.exe");
    if !bin_exe.is_file() {
        std::fs::copy(install_dir.join("choco.exe"), &bin_exe)?;
    }

    let lib = install_dir.join("lib").join("chocolatey");
    std::fs::create_dir_all(&lib)?;
    std::fs::copy(nupkg, lib.join("chocolatey.nupkg"))?;
    Ok(bin_exe)
}

fn install_dir_text(dir: &Path) -> String {
    dir.display()
        .to_string()
        .trim_end_matches(['\\', '/'])
        .to_string()
}

#[cfg(windows)]
fn set_chocolatey_install(dir: &Path) -> Result<()> {
    use winreg::enums::*;
    let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu.open_subkey_with_flags("Environment", KEY_SET_VALUE)?;
    key.set_value("ChocolateyInstall", &install_dir_text(dir))?;
    Ok(())
}

#[cfg(not(windows))]
fn set_chocolatey_install(_dir: &Path) -> Result<()> {
    Ok(())
}

#[cfg(windows)]
fn clear_chocolatey_install(dir: &Path) {
    use winreg::enums::*;
    let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
    let Ok(key) = hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_SET_VALUE) else {
        return;
    };
    let current: String = key.get_value("ChocolateyInstall").unwrap_or_default();
    let current = current.trim().trim_end_matches(['\\', '/']);
    if current.eq_ignore_ascii_case(&install_dir_text(dir)) {
        let _ = key.delete_value("ChocolateyInstall");
    }
}

#[cfg(not(windows))]
fn clear_chocolatey_install(_dir: &Path) {}

pub struct ChocolateyPlugin;

impl Plugin for ChocolateyPlugin {
    fn id(&self) -> &'static str {
        "chocolatey"
    }

    fn name(&self) -> &'static str {
        "Chocolatey"
    }

    fn description(&self) -> &'static str {
        "Install the Chocolatey CLI into the DevKit folder (Windows only) and \
         set ChocolateyInstall so packages do not need an elevated shell."
    }

    fn supported_os(&self) -> &'static [HostOS] {
        &[HostOS::Windows]
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &choco_binary(ctx),
            &ctx.install_dir,
            "Install dir exists but bin/choco.exe is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        if !self.supports_os(current_os()) {
            bail!("{}", cannot_install_message(self));
        }
        let (url, version) = resolve_chocolatey_download()?;
        println!("Chocolatey {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Chocolatey");
        let nupkg = download_file(
            &url,
            None,
            Some(&format!("chocolatey.{version}.nupkg")),
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        let extracted = tempfile::tempdir()?;
        extract_zip(&nupkg, extracted.path(), false)?;
        let dest_exe = stage_chocolatey(extracted.path(), &ctx.install_dir, &nupkg)?;
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        set_chocolatey_install(&ctx.install_dir)?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("Chocolatey {version} installed at {}", dest_exe.display()),
        ))
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        clear_chocolatey_install(&ctx.install_dir);
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn latest_version(&self, _ctx: &InstallContext) -> Result<Option<String>> {
        if current_os() != HostOS::Windows {
            return Ok(None);
        }
        Ok(Some(resolve_chocolatey_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.join("bin")],
            vars: vec![(
                "ChocolateyInstall".to_string(),
                install_dir_text(&ctx.install_dir),
            )],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::InstallState;

    fn ctx(tmp: &Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("chocolatey"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn parses_odata_version() {
        let xml = r#"<entry><d:Version>2.7.4</d:Version></entry>"#;
        assert_eq!(parse_chocolatey_version(xml).unwrap(), "2.7.4");
    }

    #[test]
    fn package_url_uses_version() {
        assert_eq!(
            chocolatey_package_url("2.7.4"),
            "https://community.chocolatey.org/api/v2/package/chocolatey/2.7.4"
        );
    }

    #[test]
    fn stage_copies_redirect_shim_and_nupkg() {
        let tmp = tempfile::tempdir().unwrap();
        let extracted = tmp.path().join("nupkg");
        let src = extracted.join("tools").join("chocolateyInstall");
        let redirects = src.join("redirects");
        std::fs::create_dir_all(&redirects).unwrap();
        std::fs::write(src.join("choco.exe"), b"real").unwrap();
        std::fs::write(redirects.join("choco.exe"), b"shim").unwrap();
        std::fs::write(redirects.join("RefreshEnv.cmd"), b"refresh").unwrap();
        let nupkg = tmp.path().join("chocolatey.nupkg");
        std::fs::write(&nupkg, b"pkg").unwrap();

        let install = tmp.path().join("chocolatey");
        let dest = stage_chocolatey(&extracted, &install, &nupkg).unwrap();
        assert_eq!(dest, install.join("bin").join("choco.exe"));
        assert_eq!(std::fs::read(&dest).unwrap(), b"shim");
        assert_eq!(
            std::fs::read(install.join("bin").join("RefreshEnv.cmd")).unwrap(),
            b"refresh"
        );
        assert_eq!(
            std::fs::read(
                install
                    .join("lib")
                    .join("chocolatey")
                    .join("chocolatey.nupkg")
            )
            .unwrap(),
            b"pkg"
        );
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = ChocolateyPlugin;
        assert_eq!(
            plugin.status(&ctx(tmp.path())).state,
            InstallState::NotInstalled
        );
    }

    #[test]
    fn status_installed_when_shim_present() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = ChocolateyPlugin;
        let c = ctx(tmp.path());
        std::fs::create_dir_all(c.install_dir.join("bin")).unwrap();
        std::fs::write(c.install_dir.join("bin").join("choco.exe"), b"shim").unwrap();
        assert_eq!(plugin.status(&c).state, InstallState::Installed);
    }

    #[test]
    fn uninstall_removes_install_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = ChocolateyPlugin;
        let c = ctx(tmp.path());
        std::fs::create_dir_all(&c.install_dir).unwrap();
        plugin.uninstall(&c).unwrap();
        assert!(!c.install_dir.exists());
    }

    #[test]
    fn env_spec_sets_chocolatey_install() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = ChocolateyPlugin;
        let c = ctx(tmp.path());
        let spec = plugin.env_spec(&c);
        assert_eq!(spec.paths, vec![c.install_dir.join("bin")]);
        assert_eq!(spec.vars[0].0, "ChocolateyInstall");
        assert_eq!(spec.vars[0].1, install_dir_text(&c.install_dir));
    }
}
