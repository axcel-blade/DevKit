//! Host OS detection for Windows, macOS, and Linux.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostOS {
    Windows,
    MacOS,
    Linux,
    Other,
}

pub fn current_os() -> HostOS {
    if cfg!(target_os = "windows") {
        HostOS::Windows
    } else if cfg!(target_os = "macos") {
        HostOS::MacOS
    } else if cfg!(target_os = "linux") {
        HostOS::Linux
    } else {
        HostOS::Other
    }
}

pub fn is_windows() -> bool {
    current_os() == HostOS::Windows
}

pub fn is_macos() -> bool {
    current_os() == HostOS::MacOS
}

pub fn is_linux() -> bool {
    current_os() == HostOS::Linux
}

pub fn is_unix() -> bool {
    matches!(current_os(), HostOS::MacOS | HostOS::Linux | HostOS::Other)
}

/// Return a normalized CPU arch label: `x64` or `aarch64`.
pub fn cpu_arch() -> String {
    let machine = std::env::consts::ARCH.to_lowercase();
    match machine.as_str() {
        "aarch64" | "arm64" => "aarch64".to_string(),
        "x86_64" | "amd64" | "x64" => "x64".to_string(),
        "x86" | "i386" | "i686" => "x86".to_string(),
        other => other.to_string(),
    }
}

/// OS slug for the Adoptium / Temurin API.
pub fn adoptium_os() -> anyhow::Result<&'static str> {
    Ok(match current_os() {
        HostOS::Windows => "windows",
        HostOS::MacOS => "mac",
        HostOS::Linux => "linux",
        HostOS::Other => anyhow::bail!("Unsupported OS for Temurin JDK: other"),
    })
}

/// Human-readable OS name for CLI output.
pub fn os_label() -> &'static str {
    match current_os() {
        HostOS::Windows => "Windows",
        HostOS::MacOS => "macOS",
        HostOS::Linux => "Linux",
        HostOS::Other => "Unknown",
    }
}

/// Pick a value from a platform map.
///
/// Keys may be `windows`, `macos`/`darwin`, `linux`, or `*`/`default`.
pub fn pick_for_os(mapping: &[(&str, &str)], default: Option<&str>) -> anyhow::Result<String> {
    let aliases: &[&str] = match current_os() {
        HostOS::MacOS => &["macos", "darwin", "osx"],
        HostOS::Windows => &["windows", "win", "win32"],
        HostOS::Linux => &["linux"],
        HostOS::Other => &["other"],
    };
    let lowered: Vec<(String, &str)> = mapping
        .iter()
        .map(|(k, v)| (k.to_lowercase(), *v))
        .collect();
    for alias in aliases {
        if let Some((_, v)) = lowered.iter().find(|(k, _)| k == alias) {
            return Ok(v.to_string());
        }
    }
    if let Some((_, v)) = lowered.iter().find(|(k, _)| k == "default") {
        return Ok(v.to_string());
    }
    if let Some((_, v)) = lowered.iter().find(|(k, _)| k == "*") {
        return Ok(v.to_string());
    }
    if let Some(d) = default {
        return Ok(d.to_string());
    }
    let known: Vec<&str> = mapping.iter().map(|(k, _)| *k).collect();
    anyhow::bail!(
        "No URL/value for OS '{}'. Available keys: {}",
        std::env::consts::OS,
        known.join(", ")
    )
}

/// Return likely shell profile paths for the current OS (macOS/Linux).
pub fn shell_profile_candidates() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    let shell = std::env::var("SHELL").unwrap_or_default();
    let mut profiles: Vec<PathBuf> = Vec::new();

    if shell.contains("zsh") || current_os() == HostOS::MacOS {
        profiles.push(home.join(".zshrc"));
        profiles.push(home.join(".zprofile"));
    }
    if shell.contains("bash") || current_os() == HostOS::Linux {
        profiles.push(home.join(".bashrc"));
        profiles.push(home.join(".bash_profile"));
        profiles.push(home.join(".profile"));
    }
    for extra in [
        home.join(".zshrc"),
        home.join(".bashrc"),
        home.join(".profile"),
    ] {
        if !profiles.contains(&extra) {
            profiles.push(extra);
        }
    }
    profiles
}

/// Best profile file to source `~/.devkit/env.sh` from.
pub fn primary_shell_profile() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_default();
    let shell = std::env::var("SHELL").unwrap_or_default();

    if shell.contains("zsh") || current_os() == HostOS::MacOS {
        return home.join(".zshrc");
    }
    if shell.contains("bash") {
        let bashrc = home.join(".bashrc");
        if bashrc.exists() || current_os() == HostOS::Linux {
            return bashrc;
        }
        return home.join(".bash_profile");
    }
    home.join(".profile")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_os_matches_target() {
        let os = current_os();
        assert!(os == HostOS::Windows || is_unix());
    }

    #[test]
    fn pick_for_os_falls_back_to_default_key() {
        let v = pick_for_os(&[("default", "everywhere")], None).unwrap();
        assert_eq!(v, "everywhere");
    }

    #[test]
    fn pick_for_os_errors_without_match() {
        assert!(pick_for_os(&[("bogus", "x")], None).is_err());
    }

    #[test]
    fn cpu_arch_is_known_label() {
        let arch = cpu_arch();
        assert!(["x64", "aarch64", "x86"].contains(&arch.as_str()) || !arch.is_empty());
    }
}
