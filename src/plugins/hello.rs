//! Hello stub plugin — download-style ZIP install into the machine `dev` folder.

use crate::download::install_zip_file;
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use anyhow::Result;
use std::io::Write;
use std::path::Path;

const MARKER: &str = ".devkit-hello";
const BIN_NAME: &str = "bin";

/// Create a tiny SDK-like ZIP (top-level folder + bin/) for the stub.
fn build_demo_zip(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("hello-sdk/bin/hello.txt", options)?;
    zip.write_all(b"hello from DevKit\n")?;
    zip.start_file("hello-sdk/README.txt", options)?;
    zip.write_all(b"DevKit hello stub SDK\n")?;
    zip.finish()?;
    Ok(())
}

pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn id(&self) -> &'static str {
        "hello"
    }

    fn name(&self) -> &'static str {
        "Hello"
    }

    fn description(&self) -> &'static str {
        "Stub plugin: ZIP → extract into machine dev folder → set PATH/env \
         (same flow as Flutter-style SDK installs)."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        let marker = ctx.install_dir.join(MARKER);
        if marker.is_file() {
            return PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
                .with_detail("Hello marker present");
        }
        let has_contents = ctx.install_dir.is_dir()
            && std::fs::read_dir(&ctx.install_dir)
                .map(|mut d| d.next().is_some())
                .unwrap_or(false);
        if has_contents {
            return PluginStatus::new(InstallState::Partial, Some(ctx.install_dir.clone()))
                .with_detail("Install dir exists without marker");
        }
        PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        let cache_zip = ctx.home.join(".cache").join("hello-sdk.zip");
        build_demo_zip(&cache_zip)?;
        install_zip_file(&cache_zip, &ctx.install_dir, true)?;
        std::fs::write(ctx.install_dir.join(MARKER), "installed\n")?;
        Ok(InstallResult::new(
            ctx.install_dir.clone(),
            format!("extracted ZIP into {}", ctx.install_dir.display()),
        ))
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        let bin_dir = ctx.install_dir.join(BIN_NAME);
        EnvSpec {
            paths: vec![bin_dir],
            vars: vec![(
                "DEVKIT_HELLO_ROOT".to_string(),
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

    fn ctx(tmp: &Path) -> InstallContext {
        InstallContext {
            install_dir: tmp.join("hello"),
            home: tmp.to_path_buf(),
            version: None,
            channel: None,
        }
    }

    #[test]
    fn install_then_uninstall_round_trips() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = HelloPlugin;
        let ctx = ctx(tmp.path());

        assert_eq!(plugin.status(&ctx).state, InstallState::NotInstalled);

        let result = plugin.install(&ctx).unwrap();
        assert!(result.install_dir.join(MARKER).is_file());
        assert!(result.install_dir.join("bin").join("hello.txt").is_file());
        assert_eq!(plugin.status(&ctx).state, InstallState::Installed);

        plugin.uninstall(&ctx).unwrap();
        assert!(!ctx.install_dir.exists());
    }
}
