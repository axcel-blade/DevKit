//! PHP Composer plugin — install composer.phar into the machine `dev` folder.

use crate::download::download_file;
use crate::platform::is_windows;
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use anyhow::{bail, Result};
use std::path::{Path, PathBuf};

const COMPOSER_PHAR_URL: &str = "https://getcomposer.org/download/latest-stable/composer.phar";
const PHAR_NAME: &str = "composer.phar";

fn php_available() -> bool {
    which::which("php").is_ok()
}

fn wrapper_path(ctx: &InstallContext) -> PathBuf {
    if is_windows() {
        ctx.install_dir.join("composer.bat")
    } else {
        ctx.install_dir.join("composer")
    }
}

fn write_wrappers(install_dir: &Path) -> Result<()> {
    let phar = install_dir.join(PHAR_NAME);
    if is_windows() {
        let content = "@echo off\r\nphp \"%~dp0composer.phar\" %*\r\n";
        std::fs::write(install_dir.join("composer.bat"), content)?;
        // Also drop a composer.cmd for shells that prefer .cmd
        std::fs::write(install_dir.join("composer.cmd"), content)?;
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let script = install_dir.join("composer");
            std::fs::write(
                &script,
                "#!/usr/bin/env bash\nDIR=\"$(cd \"$(dirname \"$0\")\" && pwd)\"\nexec php \"$DIR/composer.phar\" \"$@\"\n",
            )?;
            let mut perms = std::fs::metadata(&script)?.permissions();
            perms.set_mode(perms.mode() | 0o111);
            std::fs::set_permissions(&script, perms)?;
        }
    }

    if !phar.is_file() {
        bail!("Missing {}", phar.display());
    }
    Ok(())
}

pub struct ComposerPlugin;

impl Plugin for ComposerPlugin {
    fn id(&self) -> &'static str {
        "composer"
    }

    fn name(&self) -> &'static str {
        "Composer"
    }

    fn description(&self) -> &'static str {
        "Download latest stable Composer (composer.phar) into the machine dev folder. \
         Requires PHP on PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        let phar = ctx.install_dir.join(PHAR_NAME);
        let wrapper = wrapper_path(ctx);
        if phar.is_file() && wrapper.is_file() {
            let mut detail = wrapper.display().to_string();
            if !php_available() {
                detail.push_str(" (warning: php not found on PATH)");
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
                .with_detail("Install dir exists but composer files are incomplete");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        if !php_available() {
            println!(
                "Warning: `php` was not found on PATH. \
                 Composer is installed, but you need PHP to run it."
            );
        }
        std::fs::create_dir_all(&ctx.install_dir)?;
        let dest = ctx.install_dir.join(PHAR_NAME);
        println!("Downloading Composer from {COMPOSER_PHAR_URL} ...");
        download_file(COMPOSER_PHAR_URL, Some(&dest), Some(PHAR_NAME), None)?;
        write_wrappers(&ctx.install_dir)?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("Composer installed at {}", ctx.install_dir.display()),
        ))
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let composer_home = ctx.install_dir.join("home");
        EnvSpec {
            paths: vec![ctx.install_dir.clone()],
            vars: vec![(
                "COMPOSER_HOME".to_string(),
                composer_home
                    .canonicalize()
                    .unwrap_or(composer_home)
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

    fn ctx(tmp: &Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("composer"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = ComposerPlugin;
        let ctx = ctx(tmp.path());
        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);
    }

    #[test]
    fn status_partial_when_dir_has_contents_but_no_phar() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = ComposerPlugin;
        let ctx = ctx(tmp.path());
        std::fs::create_dir_all(&ctx.install_dir).unwrap();
        std::fs::write(ctx.install_dir.join("stray.txt"), "x").unwrap();
        assert_eq!(plugin.status(&ctx).state, InstallState::Partial);
    }
}
