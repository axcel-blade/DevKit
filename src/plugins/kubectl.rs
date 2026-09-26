//! kubectl plugin — latest stable binary from dl.k8s.io.

use crate::download::download_file;
use crate::platform::{cpu_arch, current_os, is_windows, HostOS};
use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::{binary_status, read_text_url};
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::PathBuf;

const MARKER: &str = ".devkit-kubectl";

#[cfg(unix)]
fn make_executable(path: &std::path::Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(path)?.permissions();
    perms.set_mode(perms.mode() | 0o111);
    std::fs::set_permissions(path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &std::path::Path) -> Result<()> {
    Ok(())
}

fn resolve_kubectl_download() -> Result<(String, String)> {
    // stable.txt is a one-line text file (e.g. "v1.31.0"), not JSON.
    let version = read_text_url("https://dl.k8s.io/release/stable.txt")?;
    let host = current_os();
    let arch = cpu_arch();
    let (goos, name) = match host {
        HostOS::Windows => ("windows", "kubectl.exe"),
        HostOS::Linux => ("linux", "kubectl"),
        HostOS::MacOS => ("darwin", "kubectl"),
        HostOS::Other => bail!("kubectl is not supported on this OS: other"),
    };
    let goarch = if arch == "aarch64" { "arm64" } else { "amd64" };
    let url = format!("https://dl.k8s.io/release/{version}/bin/{goos}/{goarch}/{name}");
    Ok((url, version))
}

fn kubectl_bin(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir.join(if is_windows() {
        "kubectl.exe"
    } else {
        "kubectl"
    })
}

pub struct KubectlPlugin;

impl Plugin for KubectlPlugin {
    fn id(&self) -> &'static str {
        "kubectl"
    }

    fn name(&self) -> &'static str {
        "kubectl"
    }

    fn description(&self) -> &'static str {
        "Download latest stable kubectl binary and add it to PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &kubectl_bin(ctx),
            &ctx.install_dir,
            "Install dir exists but kubectl binary is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let (url, version) = resolve_kubectl_download()?;
        println!("kubectl {version}");
        println!("URL: {url}");
        std::fs::create_dir_all(&ctx.install_dir)?;
        // dl.k8s.io serves the bare kubectl binary directly, not an archive,
        // so this downloads straight to the final path (no extract step).
        let dest = kubectl_bin(ctx);
        let mut progress = download_progress("Downloading kubectl");
        download_file(
            &url,
            Some(&dest),
            None,
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        if !is_windows() {
            make_executable(&dest)?;
        }
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!(
                "kubectl {version} installed at {}",
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
        Ok(Some(resolve_kubectl_download()?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.clone()],
            vars: vec![(
                "KUBECTL_HOME".to_string(),
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
    use std::path::Path;

    fn ctx(tmp: &Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("kubectl"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = KubectlPlugin;
        let c = ctx(tmp.path());
        assert_eq!(plugin.status(&c).state, InstallState::NotInstalled);
    }
}
