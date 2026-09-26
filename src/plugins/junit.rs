//! JUnit plugin — JUnit Platform Console Standalone JAR (all platforms).
//!
//! Installs the official `junit-platform-console-standalone` artifact from
//! Maven Central and a `junit` wrapper that runs `java -jar`. Pair with the
//! existing `jdk`, `gradle`, and `maven` plugins for a Java test toolchain.

use crate::download::download_file;
use crate::platform::is_windows;
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use crate::plugin_utils::read_text_url;
use crate::progress::download_progress;
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

const METADATA_URL: &str =
    "https://repo1.maven.org/maven2/org/junit/platform/junit-platform-console-standalone/maven-metadata.xml";
const ARTIFACT_BASE: &str =
    "https://repo1.maven.org/maven2/org/junit/platform/junit-platform-console-standalone";
const JAR_NAME: &str = "junit-platform-console-standalone.jar";
const MARKER: &str = ".devkit-junit";

/// Parse `<release>` (preferred) or `<latest>` from Maven metadata XML.
fn parse_junit_release_version(xml: &str) -> Result<String> {
    let release = regex::Regex::new(r"<release>([^<]+)</release>")
        .unwrap()
        .captures(xml)
        .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()));
    if let Some(v) = release {
        if !v.is_empty() {
            return Ok(v);
        }
    }
    let latest = regex::Regex::new(r"<latest>([^<]+)</latest>")
        .unwrap()
        .captures(xml)
        .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()));
    match latest {
        Some(v) if !v.is_empty() => Ok(v),
        _ => bail!("Could not parse JUnit version from Maven metadata"),
    }
}

/// Return `(jar_url, version)` — `--version` pins a release, otherwise latest.
fn resolve_junit_download(requested: Option<&str>) -> Result<(String, String)> {
    let version = if let Some(v) = requested {
        let v = v.trim();
        if v.is_empty() {
            bail!("Invalid JUnit version. Use a Maven version like 1.11.4.");
        }
        v.to_string()
    } else {
        let xml = read_text_url(METADATA_URL)?;
        parse_junit_release_version(&xml)?
    };
    let url = format!("{ARTIFACT_BASE}/{version}/junit-platform-console-standalone-{version}.jar");
    Ok((url, version))
}

fn jar_path(ctx: &InstallContext) -> PathBuf {
    ctx.install_dir.join(JAR_NAME)
}

fn wrapper_path(ctx: &InstallContext) -> PathBuf {
    if is_windows() {
        ctx.install_dir.join("junit.cmd")
    } else {
        ctx.install_dir.join("junit")
    }
}

fn java_available() -> bool {
    which::which("java").is_ok()
}

fn write_wrappers(install_dir: &Path) -> Result<()> {
    if is_windows() {
        let content =
            "@echo off\r\njava -jar \"%~dp0junit-platform-console-standalone.jar\" %*\r\n";
        std::fs::write(install_dir.join("junit.cmd"), content)?;
        std::fs::write(install_dir.join("junit.bat"), content)?;
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let script = install_dir.join("junit");
            std::fs::write(
                &script,
                "#!/usr/bin/env bash\nDIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nexec java -jar \"$DIR/junit-platform-console-standalone.jar\" \"$@\"\n",
            )?;
            let mut perms = std::fs::metadata(&script)?.permissions();
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&script, perms)?;
        }
        #[cfg(not(unix))]
        {
            let _ = install_dir;
        }
    }
    Ok(())
}

pub struct JunitPlugin;

impl Plugin for JunitPlugin {
    fn id(&self) -> &'static str {
        "junit"
    }

    fn name(&self) -> &'static str {
        "JUnit"
    }

    fn description(&self) -> &'static str {
        "Download JUnit Platform Console Standalone from Maven Central \
         (optional --version) and set JUNIT_HOME / PATH. Needs Java to run; \
         use the jdk plugin. Works alongside gradle and maven."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        let jar = jar_path(ctx);
        let wrapper = wrapper_path(ctx);
        if jar.is_file() && wrapper.is_file() {
            let mut detail = wrapper.display().to_string();
            if !java_available() {
                detail.push_str(" (warning: java not found on PATH)");
            }
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail(detail);
        }
        let has_contents = ctx.install_dir.is_dir()
            && std::fs::read_dir(&ctx.install_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_contents {
            return PluginStatus::new(InstallState::Partial, Some(ctx.install_dir.clone()))
                .with_detail("Install dir exists but JUnit jar/wrapper is missing");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        if !java_available() {
            println!(
                "Warning: `java` was not found on PATH. \
                 Install the jdk plugin (or another JDK) to run JUnit."
            );
        }
        let (url, version) = resolve_junit_download(ctx.version.as_deref())?;
        println!("JUnit Platform Console Standalone {version}");
        println!("URL: {url}");
        std::fs::create_dir_all(&ctx.install_dir)?;
        let dest = jar_path(ctx);
        let mut progress = download_progress("Downloading JUnit");
        download_file(
            &url,
            Some(&dest),
            Some(JAR_NAME),
            Some(&mut |d, t| progress.update(d, t)),
        )?;
        progress.done();
        if !dest.is_file() {
            bail!(
                "JUnit download finished but jar not found at {}",
                dest.display()
            );
        }
        write_wrappers(&ctx.install_dir)?;
        std::fs::write(ctx.install_dir.join(MARKER), format!("{version}\n"))?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("JUnit {version} installed at {}", ctx.install_dir.display()),
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
        Ok(Some(resolve_junit_download(None)?.1))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.clone()],
            vars: vec![(
                "JUNIT_HOME".to_string(),
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

    #[test]
    fn parse_junit_release_prefers_release_tag() {
        let xml = r#"
            <metadata>
              <versioning>
                <latest>1.11.3</latest>
                <release>1.11.4</release>
              </versioning>
            </metadata>
        "#;
        assert_eq!(parse_junit_release_version(xml).unwrap(), "1.11.4");
    }

    #[test]
    fn resolve_junit_download_honors_requested_version() {
        let (url, version) = resolve_junit_download(Some("1.10.2")).unwrap();
        assert_eq!(version, "1.10.2");
        assert!(url.ends_with("/1.10.2/junit-platform-console-standalone-1.10.2.jar"));
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = JunitPlugin;
        let ctx = InstallContext {
            install_dir: tmp.path().join("junit"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }
}
