//! Helpers to extract Windows MSI and macOS PKG installers into a folder.
//!
//! Used by plugins such as Mono that ship platform installers instead of plain ZIP/tar.

use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
// Only the Windows (extract_msi_admin) and macOS (extract_pkg) real
// implementations shell out or touch the filesystem beyond what `Path`
// itself provides — on other platforms only the `bail!`-only stubs below
// compile, which don't need these.
#[cfg(any(windows, target_os = "macos"))]
use anyhow::Context;
#[cfg(any(windows, target_os = "macos"))]
use std::fs;
#[cfg(any(windows, target_os = "macos"))]
use std::process::Command;

/// Extract a Windows MSI into `dest` via `msiexec /a` (no full install).
///
/// GitHub Actions and other CI hosts often return opaque exit 1603 when paths use
/// forward slashes or when `TARGETDIR` is poorly quoted. We normalize paths,
/// write a verbose log, and wait for msiexec via PowerShell `Start-Process`.
#[cfg(windows)]
pub fn extract_msi_admin(msi_path: &Path, dest: &Path) -> Result<PathBuf> {
    if !msi_path.is_file() {
        bail!("MSI not found: {}", msi_path.display());
    }
    // `Path::canonicalize()` always returns Windows' `\\?\`-prefixed
    // extended-length form, which msiexec doesn't understand and rejects
    // with ERROR_INSTALL_PACKAGE_OPEN_FAILED (1619) — use the stripped form
    // both tools and msiexec agree on (see `paths::to_absolute`'s doc).
    let msi_path = crate::paths::to_absolute(msi_path);
    let dest = dest.to_path_buf();

    if dest.exists() {
        fs::remove_dir_all(&dest)?;
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    let msi_arg = msi_path.display().to_string();
    let dest_arg = format!("{}\\", dest.display().to_string().trim_end_matches('\\'));
    let log_path = dest.parent().unwrap_or(Path::new(".")).join(format!(
        "{}.msiexec.log",
        dest.file_name().unwrap_or_default().to_string_lossy()
    ));
    if log_path.exists() {
        let _ = fs::remove_file(&log_path);
    }
    let log_arg = log_path.display().to_string();

    let ps_script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$msi = '{msi}'
$dest = '{dest}'
$log = '{log}'
$argList = @('/a', $msi, '/qn', '/norestart', ('TARGETDIR=' + $dest), '/l*v', $log)
$p = Start-Process -FilePath 'msiexec.exe' -ArgumentList $argList -Wait -PassThru
if ($null -eq $p) {{ exit 1 }}
exit $p.ExitCode
"#,
        msi = msi_arg.replace('\'', "''"),
        dest = dest_arg
            .trim_end_matches('\\')
            .replace('\'', "''")
            .to_string()
            + "\\",
        log = log_arg.replace('\'', "''"),
    );

    let output = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &ps_script,
        ])
        .output()
        .context("failed to launch powershell.exe")?;

    if !output.status.success() {
        let log_tail = fs::read_to_string(&log_path)
            .map(|s| {
                let bytes = s.as_bytes();
                let start = bytes.len().saturating_sub(4000);
                String::from_utf8_lossy(&bytes[start..]).to_string()
            })
            .unwrap_or_else(|_| "(could not read msiexec log)".to_string());
        bail!(
            "msiexec failed to extract MSI (exit {:?}): {}\nLog ({}):\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr),
            log_path.display(),
            log_tail
        );
    }
    if !dest.exists() || fs::read_dir(&dest)?.next().is_none() {
        bail!(
            "msiexec reported success but TARGETDIR is empty: {}",
            dest.display()
        );
    }
    Ok(dest)
}

#[cfg(not(windows))]
pub fn extract_msi_admin(_msi_path: &Path, _dest: &Path) -> Result<PathBuf> {
    bail!("MSI extraction is only supported on Windows")
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn extract_msi_admin_missing_file() {
        let dir = tempfile::tempdir().unwrap();
        let err = extract_msi_admin(&dir.path().join("missing.msi"), &dir.path().join("out"))
            .unwrap_err();
        assert!(err.to_string().contains("MSI not found"));
    }
}

/// Extract a macOS .pkg into `dest` using `pkgutil` + `cpio`.
#[cfg(target_os = "macos")]
pub fn extract_pkg(pkg_path: &Path, dest: &Path) -> Result<PathBuf> {
    if dest.exists() {
        fs::remove_dir_all(dest)?;
    }
    fs::create_dir_all(dest)?;

    let tmp = tempfile::tempdir()?;
    let expanded = tmp.path().join("expanded");
    let expand = Command::new("pkgutil")
        .args([
            "--expand",
            &pkg_path.display().to_string(),
            &expanded.display().to_string(),
        ])
        .output()
        .context("failed to launch pkgutil")?;
    if !expand.status.success() {
        bail!(
            "pkgutil --expand failed: {}",
            String::from_utf8_lossy(&expand.stderr)
        );
    }

    let mut payloads = Vec::new();
    for entry in walkdir(&expanded)? {
        if entry.file_name().map(|n| n == "Payload").unwrap_or(false) {
            payloads.push(entry);
        }
    }
    let payload = payloads
        .into_iter()
        .max_by_key(|p| fs::metadata(p).map(|m| m.len()).unwrap_or(0))
        .with_context(|| format!("No Payload found inside {}", pkg_path.display()))?;

    let extract_dir = tmp.path().join("root");
    fs::create_dir_all(&extract_dir)?;

    let payload_bytes = fs::read(&payload)?;
    let gunzip = Command::new("gunzip")
        .arg("-dc")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn();
    let data = match gunzip {
        Ok(mut child) => {
            use std::io::Write;
            child.stdin.take().unwrap().write_all(&payload_bytes)?;
            let out = child.wait_with_output()?;
            if out.status.success() {
                out.stdout
            } else {
                payload_bytes
            }
        }
        Err(_) => payload_bytes,
    };

    let mut cpio = Command::new("cpio")
        .arg("-id")
        .current_dir(&extract_dir)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("failed to launch cpio")?;
    {
        use std::io::Write;
        cpio.stdin.take().unwrap().write_all(&data)?;
    }
    let cpio_out = cpio.wait_with_output()?;
    if !cpio_out.status.success() {
        bail!(
            "cpio extract failed: {}",
            String::from_utf8_lossy(&cpio_out.stderr)
        );
    }

    for entry in fs::read_dir(&extract_dir)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }

    Ok(dest.to_path_buf())
}

#[cfg(target_os = "macos")]
fn copy_dir_all(source: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn walkdir(root: &Path) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_dir() {
                stack.push(path.clone());
            }
            out.push(path);
        }
    }
    Ok(out)
}

#[cfg(not(target_os = "macos"))]
pub fn extract_pkg(_pkg_path: &Path, _dest: &Path) -> Result<PathBuf> {
    bail!("PKG extraction is only supported on macOS")
}
