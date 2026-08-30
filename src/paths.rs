//! DevKit install path helpers.
//!
//! Default install root is a `dev` folder chosen per OS:
//!
//! - Windows: `C:\dev` (system drive + `\dev`)
//! - macOS: `/opt/dev` if writable, else `~/dev`
//! - Linux: `/opt/dev` if writable, else `~/dev`
//!
//! Override with the `DEVKIT_HOME` environment variable.
//! Plugins install to `<dev_root>/<plugin-id>` (e.g. `C:\dev\flutter`).

use crate::platform::{current_os, HostOS};
use std::path::{Path, PathBuf};

/// Return true if `path` exists and is writable, or can be created.
fn is_creatable(path: &Path) -> bool {
    if path.exists() {
        return can_write(path);
    }
    let mut parent = path.parent();
    while let Some(p) = parent {
        if p.exists() {
            return can_write(p);
        }
        let next = p.parent();
        if next == parent {
            break;
        }
        parent = next;
    }
    false
}

#[cfg(unix)]
fn can_write(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o200 != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn can_write(path: &Path) -> bool {
    // No cheap ACL check on Windows; probe with a throwaway temp file.
    if !path.is_dir() {
        return false;
    }
    let probe = path.join(".devkit_write_probe");
    let ok = std::fs::write(&probe, b"").is_ok();
    let _ = std::fs::remove_file(&probe);
    ok
}

/// Ordered candidate `dev` roots for the current OS.
pub fn preferred_dev_roots() -> Vec<PathBuf> {
    match current_os() {
        HostOS::Windows => {
            let drive = std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string());
            vec![PathBuf::from(format!("{drive}\\dev"))]
        }
        // macOS + Linux (+ other Unix): prefer machine /opt/dev, fall back to ~/dev
        _ => {
            let home_dev = dirs::home_dir().unwrap_or_default().join("dev");
            vec![PathBuf::from("/opt/dev"), home_dev]
        }
    }
}

/// Return the best default `dev` folder for this machine.
pub fn default_dev_root() -> PathBuf {
    for candidate in preferred_dev_roots() {
        if is_creatable(&candidate) {
            return candidate;
        }
    }
    dirs::home_dir().unwrap_or_default().join("dev")
}

/// Return DEVKIT_HOME / the machine `dev` root.
pub fn home() -> PathBuf {
    if let Ok(override_path) = std::env::var("DEVKIT_HOME") {
        if !override_path.is_empty() {
            return expand_and_resolve(&PathBuf::from(override_path));
        }
    }
    let root = default_dev_root();
    to_absolute(&root)
}

fn expand_and_resolve(path: &Path) -> PathBuf {
    let expanded = if let Ok(stripped) = path.strip_prefix("~") {
        dirs::home_dir()
            .map(|h| h.join(stripped))
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    };
    to_absolute(&expanded)
}

/// Canonicalize `path`, then strip Windows' `\\?\` extended-length prefix.
/// `canonicalize()` always adds that prefix on success, but external tools
/// DevKit shells out to (notably `msiexec`, used by the mono plugin) don't
/// understand the verbatim form and fail with cryptic errors like
/// `ERROR_INSTALL_PACKAGE_OPEN_FAILED` when handed one. DevKit's install
/// paths are always short — that's the point of the machine `dev` folder —
/// so the >260-char case the prefix exists for never applies here.
pub fn to_absolute(path: &Path) -> PathBuf {
    let canon = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    #[cfg(windows)]
    {
        // `Path::strip_prefix` operates on parsed `Component`s, and Windows
        // treats the whole `\\?\C:` as one atomic prefix component — it
        // won't match against the bare `\\?\` marker. Strip it as a string
        // instead.
        if let Some(stripped) = canon.to_string_lossy().strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }
    canon
}

/// Return the download cache under the dev root.
pub fn cache_dir() -> PathBuf {
    home().join(".cache")
}

/// Return `<dev_root>/<plugin-id>`.
pub fn plugin_install_dir(plugin_id: &str) -> PathBuf {
    home().join(plugin_id)
}

/// Create the dev root and cache dirs if missing; return the root.
pub fn ensure_home() -> anyhow::Result<PathBuf> {
    let root = home();
    std::fs::create_dir_all(&root)?;
    std::fs::create_dir_all(cache_dir())?;
    Ok(root)
}

/// Return the directory that holds installed SDKs (the dev root).
pub fn sdks_dir() -> PathBuf {
    home()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn home_respects_devkit_home_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join("devkit_test_home_override");
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("DEVKIT_HOME", &tmp);
        let h = home();
        std::env::remove_var("DEVKIT_HOME");
        assert_eq!(h, to_absolute(&tmp));
    }

    #[test]
    fn plugin_install_dir_is_under_home() {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = std::env::temp_dir().join("devkit_test_plugin_dir");
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("DEVKIT_HOME", &tmp);
        let dir = plugin_install_dir("hello");
        std::env::remove_var("DEVKIT_HOME");
        assert_eq!(dir, to_absolute(&tmp).join("hello"));
    }

    #[test]
    fn to_absolute_strips_windows_extended_prefix() {
        let tmp = std::env::temp_dir().join("devkit_test_to_absolute");
        std::fs::create_dir_all(&tmp).unwrap();
        let abs = to_absolute(&tmp);
        assert!(!abs.display().to_string().starts_with(r"\\?\"));
    }
}
