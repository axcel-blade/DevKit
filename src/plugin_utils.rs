//! Shared helpers for simple binary/archive plugins.
//!
//! Keeps GitHub release lookup and status checks consistent across the many
//! SDK plugins (Go, CMake, Ninja, Deno, Bun, etc.).

use crate::plugin::{InstallState, PluginStatus};
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Fetch the latest GitHub release JSON for `owner/repo`.
pub fn github_latest_release(owner: &str, repo: &str) -> Result<Value> {
    crate::download::ensure_internet_access()?;
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let resp = ureq::get(&url)
        .set("User-Agent", "DevKit")
        .set("Accept", "application/vnd.github+json")
        .call()
        .with_context(|| format!("Failed to query GitHub release {owner}/{repo}"))?;
    let value: Value = resp
        .into_json()
        .with_context(|| format!("Unexpected GitHub release payload for {owner}/{repo}"))?;
    Ok(value)
}

/// Fetch a GitHub release by tag (tag slashes are percent-encoded).
pub fn github_release_by_tag(owner: &str, repo: &str, tag: &str) -> Result<Value> {
    crate::download::ensure_internet_access()?;
    let encoded = tag.replace('/', "%2F");
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{encoded}");
    let resp = ureq::get(&url)
        .set("User-Agent", "DevKit")
        .set("Accept", "application/vnd.github+json")
        .call()
        .with_context(|| format!("Failed to query GitHub release {owner}/{repo} tag {tag}"))?;
    let value: Value = resp
        .into_json()
        .with_context(|| format!("Unexpected GitHub release payload for {owner}/{repo}@{tag}"))?;
    Ok(value)
}

/// Return `(browser_download_url, tag_name)` for the first matching asset name.
pub fn pick_release_asset(release: &Value, name_substrings: &[&str]) -> Result<(String, String)> {
    let tag = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let assets = release.get("assets").and_then(|v| v.as_array());
    if let Some(assets) = assets {
        for asset in assets {
            let name = asset.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let url = asset
                .get("browser_download_url")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if url.is_empty() {
                continue;
            }
            if name_substrings.iter().all(|s| name.contains(s)) {
                return Ok((url.to_string(), tag));
            }
        }
    }
    let html_url = release
        .get("html_url")
        .and_then(|v| v.as_str())
        .unwrap_or(&tag)
        .to_string();
    bail!(
        "No GitHub asset matching {:?} in {}",
        name_substrings,
        html_url
    )
}

/// Common installed / partial / not_installed status for a marker binary.
pub fn binary_status(binary: &Path, install_dir: &Path, missing_detail: &str) -> PluginStatus {
    if binary.is_file() {
        return PluginStatus::new(InstallState::Installed, Some(install_dir.to_path_buf()))
            .with_detail(binary.display().to_string());
    }
    let has_contents = install_dir.is_dir()
        && std::fs::read_dir(install_dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false);
    if has_contents {
        return PluginStatus::new(InstallState::Partial, Some(install_dir.to_path_buf()))
            .with_detail(missing_detail);
    }
    PluginStatus::new(InstallState::NotInstalled, Some(install_dir.to_path_buf()))
}

/// Recursively find every file under `root` whose file name equals `name`
/// (a small stand-in for Python's `Path.rglob`), sorted shallowest-first so
/// callers can take `.first()` to prefer e.g. `bin/tool` over a deeper copy.
pub fn find_files_named(root: &Path, name: &str) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                stack.push(path);
            } else if path.file_name().and_then(|n| n.to_str()) == Some(name) {
                out.push(path);
            }
        }
    }
    out.sort_by_key(|p| (p.components().count(), p.display().to_string()));
    out
}

/// Download a small text document (e.g. kubectl stable.txt).
pub fn read_text_url(url: &str) -> Result<String> {
    crate::download::ensure_internet_access()?;
    let resp = ureq::get(url)
        .set("User-Agent", "DevKit")
        .call()
        .with_context(|| format!("Failed to download {url}"))?;
    let text = resp.into_string()?;
    Ok(text.trim().to_string())
}

/// Pick the newest `X.Y.Z/` directory entry from Apache Maven listing HTML.
pub fn parse_maven_latest_version(html: &str) -> Result<String> {
    let re = regex::Regex::new(r#"href="(\d+\.\d+\.\d+)/""#).unwrap();
    let versions: Vec<String> = re.captures_iter(html).map(|c| c[1].to_string()).collect();
    if versions.is_empty() {
        bail!("Could not parse Apache Maven version listing");
    }
    let key = |v: &str| -> Vec<u32> { v.split('.').map(|p| p.parse().unwrap_or(0)).collect() };
    Ok(versions.into_iter().max_by_key(|v| key(v)).unwrap())
}

/// Read the version stored in a plugin's `.devkit-<id>` install marker.
///
/// Returns the first line of the marker when it looks like a version. Placeholder markers such as `installed`, `stable`, or
/// `system-wrapper` return `None` so the menu shows "-" instead of noise.
pub fn read_marker_version(install_dir: &Path, plugin_id: &str) -> Option<String> {
    let marker = install_dir.join(format!(".devkit-{plugin_id}"));
    let text = std::fs::read_to_string(marker).ok()?;
    let first = text.lines().next()?.trim();
    if looks_like_version(first) {
        Some(first.to_string())
    } else {
        None
    }
}

/// Normalize a version string for display and comparison: trim, drop tool
/// prefixes some release tags carry (`bun-`, `go`, `v`), and ignore case, so
/// `bun-v1.2.3`, `go1.2.3`, `v1.2.3`, and `1.2.3` all become `1.2.3`.
pub fn normalize_version(version: &str) -> String {
    let v = version.trim().to_lowercase();
    let v = v.strip_prefix("bun-").unwrap_or(&v);
    let v = v.strip_prefix("go").unwrap_or(v);
    v.strip_prefix('v').unwrap_or(v).to_string()
}

/// True when a (normalized) string is an actual version rather than a
/// placeholder tag like `latest` or `nightly-x86_64`.
pub fn looks_like_version(version: &str) -> bool {
    normalize_version(version)
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_marker_version_skips_placeholders() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(read_marker_version(tmp.path(), "x"), None);
        std::fs::write(
            tmp.path().join(".devkit-x"),
            "v1.2.3
",
        )
        .unwrap();
        assert_eq!(
            read_marker_version(tmp.path(), "x").as_deref(),
            Some("v1.2.3")
        );
        std::fs::write(
            tmp.path().join(".devkit-x"),
            "system-wrapper
/usr/bin/x
",
        )
        .unwrap();
        assert_eq!(read_marker_version(tmp.path(), "x"), None);
    }

    #[test]
    fn normalize_version_strips_prefixes() {
        assert_eq!(normalize_version("v1.2.3"), "1.2.3");
        assert_eq!(normalize_version("go1.23.0"), "1.23.0");
        assert_eq!(normalize_version(" 3.9.6 "), "3.9.6");
        assert_eq!(normalize_version("bun-v1.4.2"), "1.4.2");
    }

    #[test]
    fn looks_like_version_rejects_placeholders() {
        assert!(looks_like_version("v1.2.3"));
        assert!(!looks_like_version("latest"));
        assert!(!looks_like_version("nightly-x86_64"));
    }

    #[test]
    fn parse_maven_latest_version_picks_max() {
        let html = r#"<a href="3.9.6/">3.9.6/</a><a href="3.10.1/">3.10.1/</a>"#;
        assert_eq!(parse_maven_latest_version(html).unwrap(), "3.10.1");
    }

    #[test]
    fn parse_maven_latest_version_errors_when_empty() {
        assert!(parse_maven_latest_version("<html></html>").is_err());
    }

    #[test]
    fn find_files_named_locates_nested_file_shallowest_first() {
        let tmp = tempfile::tempdir().unwrap();
        let shallow = tmp.path().join("bin").join("tool");
        let deep = tmp.path().join("x").join("y").join("bin").join("tool");
        for path in [&shallow, &deep] {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, b"stub").unwrap();
        }
        let found = find_files_named(tmp.path(), "tool");
        assert_eq!(found.len(), 2);
        assert_eq!(found[0], shallow);
    }

    #[test]
    fn pick_release_asset_matches_all_substrings() {
        let release: Value = serde_json::json!({
            "tag_name": "v1.0",
            "assets": [
                {"name": "tool-linux-x64.tar.gz", "browser_download_url": "https://x/linux"},
                {"name": "tool-windows-x64.zip", "browser_download_url": "https://x/windows"},
            ]
        });
        let (url, tag) = pick_release_asset(&release, &["windows", "x64"]).unwrap();
        assert_eq!(url, "https://x/windows");
        assert_eq!(tag, "v1.0");
    }
}
