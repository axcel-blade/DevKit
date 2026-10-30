//! Yarn plugin — latest `yarn` tarball from the npm registry, run through
//! `node` wrapper scripts. Needs Node.js (the `node` plugin).

use crate::plugin::{EnvSpec, InstallContext, InstallResult, Plugin, PluginStatus};
use crate::plugin_utils::binary_status;
use crate::plugins::node_pm::{install_package, registry_latest, wrapper_path};
use anyhow::Result;

/// `(command, script)` launchers written into the install dir.
const COMMANDS: &[(&str, &str)] = &[("yarn", "bin/yarn.js"), ("yarnpkg", "bin/yarn.js")];

pub struct YarnPlugin;

impl Plugin for YarnPlugin {
    fn id(&self) -> &'static str {
        "yarn"
    }

    fn name(&self) -> &'static str {
        "Yarn"
    }

    // Runs on Node.js; install the node plugin first if it is missing.
    fn prerequisites(&self) -> &'static [(&'static str, &'static str)] {
        &[("node", "node")]
    }

    fn description(&self) -> &'static str {
        "Download latest Yarn classic from the npm registry (yarn / yarnpkg wrappers). Requires Node.js on PATH."
    }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        binary_status(
            &wrapper_path(&ctx.install_dir, "yarn"),
            &ctx.install_dir,
            "Install dir exists but the yarn launcher is missing",
        )
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        install_package(ctx, self.id(), "yarn", self.name(), COMMANDS)
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    /// Same registry lookup `install` uses, so it matches the marker.
    fn latest_version(&self, _ctx: &InstallContext) -> Result<Option<String>> {
        Ok(Some(registry_latest("yarn")?.0))
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.clone()],
            vars: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::InstallState;

    #[test]
    fn status_not_installed_on_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let c = InstallContext {
            install_dir: tmp.path().join("yarn"),
            home: tmp.path().to_path_buf(),
            version: None,
            channel: None,
        };
        assert_eq!(YarnPlugin.status(&c).state, InstallState::NotInstalled);
    }
}
