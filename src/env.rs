//! User environment PATH and variable management.
//!
//! - Windows: user environment via the registry
//! - macOS / Linux: `~/.devkit/env.sh` plus a source line in the user shell profile

use crate::platform::{is_windows, os_label, primary_shell_profile};
use crate::plugin::EnvSpec;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const ENV_SH_BEGIN: &str = "# >>> DevKit >>>";
const ENV_SH_END: &str = "# <<< DevKit <<<";
const PROFILE_BEGIN: &str = "# >>> DevKit >>>";
const PROFILE_END: &str = "# <<< DevKit <<<";
const SOURCE_LINE: &str = r#"source "$HOME/.devkit/env.sh""#;

fn norm_path(value: &Path) -> String {
    let expanded = if let Ok(stripped) = value.strip_prefix("~") {
        dirs::home_dir()
            .map(|h| h.join(stripped))
            .unwrap_or_else(|| value.to_path_buf())
    } else {
        value.to_path_buf()
    };
    expanded
        .canonicalize()
        .unwrap_or(expanded)
        .display()
        .to_string()
}

fn path_list(value: &str) -> Vec<String> {
    if value.is_empty() {
        return Vec::new();
    }
    value
        .split(if is_windows() { ';' } else { ':' })
        .filter(|p| !p.is_empty())
        .map(|p| p.to_string())
        .collect()
}

fn join_paths(parts: &[String]) -> String {
    parts.join(if is_windows() { ";" } else { ":" })
}

pub struct EnvManager;

impl EnvManager {
    pub fn new() -> Self {
        Self
    }

    /// Prepend PATH entries and set env vars from `spec`.
    pub fn apply(&self, spec: &EnvSpec) -> Result<()> {
        if is_windows() {
            self.apply_windows(spec)
        } else {
            self.apply_unix(spec)
        }
    }

    /// Remove PATH entries and env vars declared by `spec`.
    pub fn revert(&self, spec: &EnvSpec) -> Result<()> {
        if is_windows() {
            self.revert_windows(spec)
        } else {
            self.revert_unix(spec)
        }
    }

    /// Return whether each expected PATH/var is present in the user env.
    pub fn check(&self, spec: &EnvSpec) -> Result<BTreeMap<String, bool>> {
        if is_windows() {
            self.check_windows(spec)
        } else {
            self.check_unix(spec)
        }
    }

    /// Short description of how env changes are persisted on this OS.
    pub fn backend_description(&self) -> String {
        if is_windows() {
            "Windows user environment (registry)".to_string()
        } else {
            let profile = primary_shell_profile();
            format!(
                "{} shell: ~/.devkit/env.sh (sourced from {})",
                os_label(),
                profile.display()
            )
        }
    }

    // --- Windows (user environment via registry) ---

    #[cfg(windows)]
    fn user_env_key(&self) -> Result<winreg::RegKey> {
        use winreg::enums::*;
        let hkcu = winreg::RegKey::predef(HKEY_CURRENT_USER);
        Ok(hkcu.open_subkey_with_flags("Environment", KEY_READ | KEY_SET_VALUE)?)
    }

    #[cfg(windows)]
    fn read_user_env(&self, name: &str) -> Option<String> {
        self.user_env_key().ok()?.get_value(name).ok()
    }

    #[cfg(windows)]
    fn write_user_env(&self, name: &str, value: &str) -> Result<()> {
        use winreg::enums::*;
        let key = self.user_env_key()?;
        if name.eq_ignore_ascii_case("path") {
            let wide: Vec<u16> = value.encode_utf16().chain(std::iter::once(0)).collect();
            let bytes: Vec<u8> = wide.iter().flat_map(|c| c.to_le_bytes()).collect();
            key.set_raw_value(
                name,
                &winreg::RegValue {
                    bytes,
                    vtype: REG_EXPAND_SZ,
                },
            )?;
        } else {
            key.set_value(name, &value)?;
        }
        Ok(())
    }

    #[cfg(windows)]
    fn delete_user_env(&self, name: &str) -> Result<()> {
        if let Ok(key) = self.user_env_key() {
            let _ = key.delete_value(name);
        }
        Ok(())
    }

    #[cfg(windows)]
    fn broadcast_env_change(&self) {
        use std::ffi::OsStr;
        use std::iter::once;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Foundation::{HWND, LPARAM, WPARAM};
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
        };

        let wide: Vec<u16> = OsStr::new("Environment")
            .encode_wide()
            .chain(once(0))
            .collect();
        unsafe {
            let mut result: usize = 0;
            SendMessageTimeoutW(
                HWND_BROADCAST as HWND,
                WM_SETTINGCHANGE,
                0 as WPARAM,
                wide.as_ptr() as LPARAM,
                SMTO_ABORTIFHUNG,
                5000,
                &mut result as *mut usize,
            );
        }
    }

    #[cfg(windows)]
    fn apply_windows(&self, spec: &EnvSpec) -> Result<()> {
        let path_entries: Vec<String> = spec.paths.iter().map(|p| norm_path(p)).collect();
        let current = self.read_user_env("Path").unwrap_or_default();
        let mut parts = path_list(&current);
        let mut lower: Vec<String> = parts.iter().map(|p| p.to_lowercase()).collect();
        for entry in path_entries.iter().rev() {
            let entry_lower = entry.to_lowercase();
            if !lower.contains(&entry_lower) {
                parts.insert(0, entry.clone());
                lower.push(entry_lower);
            }
        }
        self.write_user_env("Path", &join_paths(&parts))?;

        for (name, value) in &spec.vars {
            self.write_user_env(name, value)?;
        }
        self.broadcast_env_change();
        Ok(())
    }

    #[cfg(windows)]
    fn revert_windows(&self, spec: &EnvSpec) -> Result<()> {
        let path_entries: Vec<String> = spec
            .paths
            .iter()
            .map(|p| norm_path(p).to_lowercase())
            .collect();
        let current = self.read_user_env("Path").unwrap_or_default();
        let parts: Vec<String> = path_list(&current)
            .into_iter()
            .filter(|p| !path_entries.contains(&p.to_lowercase()))
            .collect();
        self.write_user_env("Path", &join_paths(&parts))?;

        for (name, _) in &spec.vars {
            self.delete_user_env(name)?;
        }
        self.broadcast_env_change();
        Ok(())
    }

    #[cfg(windows)]
    fn check_windows(&self, spec: &EnvSpec) -> Result<BTreeMap<String, bool>> {
        let mut result = BTreeMap::new();
        let current = self.read_user_env("Path").unwrap_or_default();
        let lower_parts: Vec<String> = path_list(&current)
            .into_iter()
            .map(|p| p.to_lowercase())
            .collect();
        for p in &spec.paths {
            let key = format!("PATH:{}", norm_path(p));
            result.insert(key, lower_parts.contains(&norm_path(p).to_lowercase()));
        }
        for (name, value) in &spec.vars {
            let actual = self.read_user_env(name);
            result.insert(name.clone(), actual.as_deref() == Some(value.as_str()));
        }
        Ok(result)
    }

    #[cfg(not(windows))]
    fn apply_windows(&self, _spec: &EnvSpec) -> Result<()> {
        unreachable!("apply_windows called on non-Windows target")
    }

    #[cfg(not(windows))]
    fn revert_windows(&self, _spec: &EnvSpec) -> Result<()> {
        unreachable!("revert_windows called on non-Windows target")
    }

    #[cfg(not(windows))]
    fn check_windows(&self, _spec: &EnvSpec) -> Result<BTreeMap<String, bool>> {
        unreachable!("check_windows called on non-Windows target")
    }

    // --- macOS / Linux (managed snippet in ~/.devkit/env.sh) ---

    fn env_sh_path(&self) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".devkit")
            .join("env.sh")
    }

    fn apply_unix(&self, spec: &EnvSpec) -> Result<()> {
        let env_sh = self.env_sh_path();
        if let Some(parent) = env_sh.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let block = self.build_unix_block(spec);
        let existing = std::fs::read_to_string(&env_sh).unwrap_or_default();
        let updated = self.replace_or_append_block(&existing, &block);
        std::fs::write(&env_sh, updated)?;
        self.ensure_profile_source()?;
        Ok(())
    }

    fn ensure_profile_source(&self) -> Result<PathBuf> {
        let profile = primary_shell_profile();
        if let Some(parent) = profile.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let block = format!("{PROFILE_BEGIN}\n{SOURCE_LINE}\n{PROFILE_END}\n");
        let existing = std::fs::read_to_string(&profile).unwrap_or_default();
        if existing.contains(SOURCE_LINE) || existing.contains("$HOME/.devkit/env.sh") {
            return Ok(profile);
        }
        let sep = if !existing.is_empty() && !existing.ends_with('\n') {
            "\n"
        } else {
            ""
        };
        std::fs::write(&profile, format!("{existing}{sep}{block}"))?;
        Ok(profile)
    }

    fn revert_unix(&self, spec: &EnvSpec) -> Result<()> {
        let env_sh = self.env_sh_path();
        if !env_sh.exists() {
            return Ok(());
        }
        let text = std::fs::read_to_string(&env_sh)?;
        let block = match self.extract_block(&text) {
            Some(b) => b,
            None => return Ok(()),
        };
        let path_targets: Vec<String> = spec.paths.iter().map(|p| norm_path(p)).collect();
        let var_names: Vec<&str> = spec.vars.iter().map(|(k, _)| k.as_str()).collect();
        let export_re = regex::Regex::new(r"^export\s+([A-Za-z_][A-Za-z0-9_]*)=").unwrap();

        let kept: Vec<&str> = block
            .lines()
            .filter(|line| {
                let path_hit = path_targets.iter().any(|p| {
                    line.contains(&format!("PATH=\"{p}:")) || line.contains(&format!("PATH={p}:"))
                });
                let var_hit = export_re
                    .captures(line)
                    .map(|c| var_names.contains(&c.get(1).unwrap().as_str()))
                    .unwrap_or(false);
                !(path_hit || var_hit)
            })
            .collect();
        let new_block = kept.join("\n").trim().to_string();

        let updated = if new_block.is_empty()
            || new_block == ENV_SH_BEGIN
            || new_block == format!("{ENV_SH_BEGIN}\n{ENV_SH_END}")
        {
            self.remove_block(&text)
        } else {
            let mut nb = new_block;
            if !nb.starts_with(ENV_SH_BEGIN) {
                nb = format!("{ENV_SH_BEGIN}\n{nb}\n{ENV_SH_END}");
            }
            if !nb.ends_with(ENV_SH_END) {
                nb = format!("{}\n{ENV_SH_END}", nb.trim_end());
            }
            self.replace_or_append_block(&text, &nb)
        };
        std::fs::write(&env_sh, updated)?;
        Ok(())
    }

    fn check_unix(&self, spec: &EnvSpec) -> Result<BTreeMap<String, bool>> {
        let mut result = BTreeMap::new();
        let env_sh = self.env_sh_path();
        let text = std::fs::read_to_string(&env_sh).unwrap_or_default();
        for p in &spec.paths {
            let np = norm_path(p);
            result.insert(format!("PATH:{np}"), text.contains(&np));
        }
        for (name, value) in &spec.vars {
            let pattern = regex::Regex::new(&format!(
                r#"(?m)^export\s+{}="{}"\s*$"#,
                regex::escape(name),
                regex::escape(value)
            ))
            .unwrap();
            result.insert(name.clone(), pattern.is_match(&text));
        }
        Ok(result)
    }

    fn build_unix_block(&self, spec: &EnvSpec) -> String {
        let mut lines = vec![ENV_SH_BEGIN.to_string()];
        for p in &spec.paths {
            let np = norm_path(p);
            lines.push(format!(r#"export PATH="{np}:$PATH""#));
        }
        for (name, value) in &spec.vars {
            let safe = value.replace('"', "\\\"");
            lines.push(format!(r#"export {name}="{safe}""#));
        }
        lines.push(ENV_SH_END.to_string());
        lines.join("\n") + "\n"
    }

    fn replace_or_append_block(&self, text: &str, block: &str) -> String {
        let existing_block = self.extract_block(text);
        let Some(existing_block) = existing_block else {
            let sep = if !text.is_empty() && !text.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            return format!("{text}{sep}{block}");
        };

        let old_lines: Vec<&str> = existing_block
            .lines()
            .filter(|l| *l != ENV_SH_BEGIN && *l != ENV_SH_END && !l.trim().is_empty())
            .collect();
        let new_inner: Vec<&str> = block
            .lines()
            .filter(|l| *l != ENV_SH_BEGIN && *l != ENV_SH_END && !l.trim().is_empty())
            .collect();

        let mut merged: Vec<&str> = Vec::new();
        let mut seen: Vec<&str> = Vec::new();
        for line in new_inner.into_iter().chain(old_lines) {
            if seen.contains(&line) {
                continue;
            }
            seen.push(line);
            merged.push(line);
        }
        let mut new_block = vec![ENV_SH_BEGIN];
        new_block.extend(merged);
        new_block.push(ENV_SH_END);
        let new_block = new_block.join("\n") + "\n";
        self.replace_block(text, &new_block)
    }

    fn extract_block<'a>(&self, text: &'a str) -> Option<&'a str> {
        let start = text.find(ENV_SH_BEGIN)?;
        let end = text.find(ENV_SH_END)?;
        if end < start {
            return None;
        }
        Some(&text[start..end + ENV_SH_END.len()])
    }

    fn replace_block(&self, text: &str, new_block: &str) -> String {
        let (Some(start), Some(end)) = (text.find(ENV_SH_BEGIN), text.find(ENV_SH_END)) else {
            return format!("{text}{new_block}");
        };
        format!(
            "{}{}{}",
            &text[..start],
            new_block,
            &text[end + ENV_SH_END.len()..]
        )
    }

    fn remove_block(&self, text: &str) -> String {
        let (Some(start), Some(end)) = (text.find(ENV_SH_BEGIN), text.find(ENV_SH_END)) else {
            return text.to_string();
        };
        let before = text[..start].trim_end_matches('\n');
        let after = text[end + ENV_SH_END.len()..].trim_start_matches('\n');
        if !before.is_empty() && !after.is_empty() {
            format!("{before}\n{after}")
        } else {
            format!("{before}{after}")
        }
    }
}

impl Default for EnvManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, not(windows)))]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static HOME_LOCK: Mutex<()> = Mutex::new(());

    fn with_home<T>(f: impl FnOnce(&Path) -> T) -> T {
        let _guard = HOME_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let prev = std::env::var("HOME").ok();
        std::env::set_var("HOME", tmp.path());
        let result = f(tmp.path());
        match prev {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
        result
    }

    #[test]
    fn unix_apply_check_revert_round_trips() {
        with_home(|home| {
            let install = home.join("sdk").join("bin");
            std::fs::create_dir_all(&install).unwrap();
            let spec = EnvSpec {
                paths: vec![install.clone()],
                vars: vec![("DEVKIT_TEST".to_string(), "yes".to_string())],
            };

            let mgr = EnvManager::new();
            mgr.apply(&spec).unwrap();

            let env_sh = home.join(".devkit").join("env.sh");
            assert!(env_sh.is_file());
            let text = std::fs::read_to_string(&env_sh).unwrap();
            assert!(text.contains(&norm_path(&install)));
            assert!(text.contains(r#"export DEVKIT_TEST="yes""#));

            let profile = std::fs::read_to_string(primary_shell_profile()).unwrap();
            assert!(profile.contains(".devkit/env.sh"));

            let checks = mgr.check(&spec).unwrap();
            assert!(checks.values().all(|v| *v));

            mgr.revert(&spec).unwrap();
            let text_after = std::fs::read_to_string(&env_sh).unwrap();
            assert!(!text_after.contains(&norm_path(&install)));
            assert!(!text_after.contains("DEVKIT_TEST"));
        });
    }

    #[test]
    fn unix_merge_multiple_applies() {
        with_home(|home| {
            let a = home.join("a");
            let b = home.join("b");
            std::fs::create_dir_all(&a).unwrap();
            std::fs::create_dir_all(&b).unwrap();

            let mgr = EnvManager::new();
            mgr.apply(&EnvSpec {
                paths: vec![a.clone()],
                vars: vec![("A".to_string(), "1".to_string())],
            })
            .unwrap();
            mgr.apply(&EnvSpec {
                paths: vec![b.clone()],
                vars: vec![("B".to_string(), "2".to_string())],
            })
            .unwrap();

            let text = std::fs::read_to_string(home.join(".devkit").join("env.sh")).unwrap();
            assert!(text.contains(&norm_path(&a)));
            assert!(text.contains(&norm_path(&b)));
            assert!(text.contains(r#"export A="1""#));
            assert!(text.contains(r#"export B="2""#));
        });
    }
}
