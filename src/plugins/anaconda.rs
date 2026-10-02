//! Anaconda plugin — newest Anaconda3 distribution from repo.anaconda.com.
//!
//! Runs the official installer silently into the plugin folder: the NSIS
//! `.exe` on Windows (`/S ... /D=`) and the shell installer on macOS/Linux
//! (`bash <installer> -b -p <dir>`). Never touches the registry, PATH or
//! shell profiles itself — DevKit's env step handles PATH.

use crate::download::download_file;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, read_text_url};
use crate::progress::download_progress;
use anyhow::{bail, Context, Result};
use std::path::PathBuf;
use std::process::Command;

const ARCHIVE: &str = "https://repo.anaconda.com/archive";
const MARKER: &str = ".devkit-anaconda";

/// Installer file suffix (after `Anaconda3-<version>-`) for a host/arch pair.
fn installer_suffix(host: HostOS, arch: &str) -> Result<&'static str> {
    let suffix = match host {
        HostOS::Windows => "Windows-x86_64.exe",
        HostOS::Linux if arch == "aarch64" => "Linux-aarch64.sh",
        HostOS::Linux => "Linux-x86_64.sh",
        HostOS::MacOS if arch == "aarch64" => "MacOSX-arm64.sh",
        HostOS::MacOS => "MacOSX-x86_64.sh",
        HostOS::Other => bail!("Anaconda is not supported on this OS: other"),
    };
    Ok(suffix)
}

/// Find the first `Anaconda3-<version>-<suffix>` in the archive listing.
///
/// The listing is sorted newest first, so the first hit is the latest release.
/// Returns `(file_name, version)`, e.g. `("Anaconda3-2026.07-1-Linux-x86_64.sh", "2026.07-1")`.
fn parse_latest_installer(html: &str, suffix: &str) -> Option<(String, String)> {
    const PREFIX: &str = "Anaconda3-";
    let mut rest = html;
    while let Some(start) = rest.find(PREFIX) {
        let tail = &rest[start + PREFIX.len()..];
        let end = tail
            .find(|c: char| c == '"' || c == '<' || c == '>' || c.is_whitespace())
            .unwrap_or(tail.len());
        let candidate = &tail[..end];
        if let Some(version) = candidate.strip_suffix(&format!("-{suffix}")) {
            // Guard against odd names: versions look like `2026.07-1`.
            if !version.is_empty()
                && version
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
            {
                return Some((format!("{PREFIX}{candidate}"), version.to_string()));
            }
        }
        rest = &tail[end..];
    }
    None
}

/// Return `(download_url, file_name, version)` for this host.
fn resolve_anaconda_download() -> Result<(String, String, String)> {
    let suffix = installer_suffix(current_os(), &cpu_arch())?;
    let html = read_text_url(&format!("{ARCHIVE}/"))?;
    let (file, version) = parse_latest_installer(&html, suffix)
        .with_context(|| format!("No Anaconda3 installer for {suffix} in {ARCHIVE}/"))?;
    Ok((format!("{ARCHIVE}/{file}"), file, version))
}

/// Folder holding the `python` / `conda` binaries.
fn bin_dir(ctx: &InstallContext) -> PathBuf {
    if is_windows() {
        ctx.install_dir.join("Scripts")
    } else {
        ctx.install_dir.join("bin")
    }
}

fn conda_bin(ctx: &InstallContext) -> PathBuf {
    bin_dir(ctx).join(if is_windows() { "conda.exe" } else { "conda" })
}

/// Run the downloaded installer silently into `ctx.install_dir`.
fn run_installer(installer: &std::path::Path, ctx: &InstallContext) -> Result<()> {
    // Both installers refuse a non-empty prefix, so start clean.
    if ctx.install_dir.exists() {
        std::fs::remove_dir_all(&ctx.install_dir)?;
    }
    let dest = crate::paths::to_absolute(&ctx.install_dir);
    let status = if is_windows() {
        // NSIS: /D must be last, unquoted, with no trailing slash. Keep the
        // install private: no registry Python, no PATH edits, no shortcuts.
        let dir_arg = format!("/D={}", dest.display().to_string().trim_end_matches('\\'));
        Command::new(installer)
            .args([
                "/S",
                "/InstallationType=JustMe",
                "/RegisterPython=0",
                "/AddToPath=0",
                "/NoShortcuts=1",
                "/NoRegistry=1",
            ])
            .arg(dir_arg)
            .status()
    } else {
        // -b batch mode (accepts license, no profile edits), -p prefix.
        Command::new("bash")
            .arg(installer)
            .arg("-b")
            .arg("-p")
            .arg(&dest)
            .status()
    }
    .context("failed to launch Anaconda installer")?;
    if !status.success() {
        bail!("Anaconda installer exited with {:?}", status.code());
    }
    Ok(())
}

pub struct AnacondaPlugin;

impl Plugin for AnacondaPlugin {
    fn id(&self) -> &'static str {
        "anaconda"
    }

    fn name(&self) -> &'static str {
        "Anaconda"
    }

    fn description(&self) -> &'static str {
        "Silently install the latest Anaconda3 distribution (conda + Python) and add it to PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &conda_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but conda binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, file, version) = resolve_anaconda_download()?;
        println!("Anaconda3 {version}");
        println!("URL: {url}");
        let mut progress = download_progress("Downloading Anaconda");
        let installer = download_file(
            &url,
            None,
            Some(&file),
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        println!(
            "Running silent install into {} (this takes a few minutes) ...",
            ctx.install_dir.display()
        );
        run_installer(&installer, ctx)?;
        let binary = conda_bin(ctx);
        if !binary.is_file() {
            bail!(
                "Anaconda installed but conda not found at {}",
                binary.display()
            );
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "Anaconda3 {version} installed at {}",
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

    /// Same resolver `install` uses, so the string matches the marker it writes.
    fn latest_version(&self, _ctx: &InstallContext) -> Result<Option<String>> {
        Ok(Some(resolve_anaconda_download()?.2))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        // Windows keeps python.exe at the prefix root and conda in Scripts/;
        // Unix keeps both in bin/.
        let mut paths = vec![bin_dir(ctx)];
        if is_windows() {
            paths.insert(0, ctx.install_dir.clone());
            paths.push(ctx.install_dir.join("Library").join("bin"));
        }
        EnvSpec {
            paths,
            vars: vec![(
                "CONDA_HOME".to_string(),
                ctx.install_dir
                    .canonicalize()
                    .unwrap_or_else(|_| ctx.install_dir.clone())
                    .display()
                    .to_string(),
            )],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::InstallState;

    fn ctx(tmp: &std::path::Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("anaconda"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    const LISTING: &str = r#"
<tr><td><a href="Anaconda3-2026.07-1-Windows-x86_64.exe">Anaconda3-2026.07-1-Windows-x86_64.exe</a></td></tr>
<tr><td><a href="Anaconda3-2026.07-1-Linux-x86_64.sh">Anaconda3-2026.07-1-Linux-x86_64.sh</a></td></tr>
<tr><td><a href="Anaconda3-2025.12-2-Windows-x86_64.exe">Anaconda3-2025.12-2-Windows-x86_64.exe</a></td></tr>
<tr><td><a href="Anaconda3-2025.12-2-MacOSX-x86_64.sh">Anaconda3-2025.12-2-MacOSX-x86_64.sh</a></td></tr>
"#;

    #[test]
    fn parse_latest_picks_first_match() {
        let (file, version) = parse_latest_installer(LISTING, "Windows-x86_64.exe").unwrap();
        assert_eq!(file, "Anaconda3-2026.07-1-Windows-x86_64.exe");
        assert_eq!(version, "2026.07-1");
    }

    #[test]
    fn parse_latest_falls_back_to_older_release() {
        let (_, version) = parse_latest_installer(LISTING, "MacOSX-x86_64.sh").unwrap();
        assert_eq!(version, "2025.12-2");
        assert!(parse_latest_installer(LISTING, "Linux-aarch64.sh").is_none());
    }

    #[test]
    fn installer_suffix_matches_host() {
        assert_eq!(
            installer_suffix(HostOS::Windows, "x86_64").unwrap(),
            "Windows-x86_64.exe"
        );
        assert_eq!(
            installer_suffix(HostOS::Linux, "aarch64").unwrap(),
            "Linux-aarch64.sh"
        );
        assert_eq!(
            installer_suffix(HostOS::MacOS, "aarch64").unwrap(),
            "MacOSX-arm64.sh"
        );
        assert!(installer_suffix(HostOS::Other, "x86_64").is_err());
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let c = ctx(tmp.path());
        assert_eq!(AnacondaPlugin.status(&c).state, InstallState::NotInstalled);
    }

    #[test]
    fn status_installed_when_conda_present() {
        let tmp = tempfile::tempdir().unwrap();
        let c = ctx(tmp.path());
        std::fs::create_dir_all(bin_dir(&c)).unwrap();
        std::fs::write(conda_bin(&c), b"stub").unwrap();
        assert_eq!(AnacondaPlugin.status(&c).state, InstallState::Installed);
    }

    #[test]
    fn env_spec_sets_conda_home() {
        let tmp = tempfile::tempdir().unwrap();
        let c = ctx(tmp.path());
        let spec = AnacondaPlugin.env_spec(&c);
        assert!(spec.paths.contains(&bin_dir(&c)));
        assert_eq!(spec.vars[0].0, "CONDA_HOME");
    }
}
