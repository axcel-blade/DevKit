//! Plugin contract for DevKit installers.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallState {
    Installed,
    NotInstalled,
    Partial,
}

impl InstallState {
    pub fn as_str(&self) -> &'static str {
        match self {
            InstallState::Installed => "installed",
            InstallState::NotInstalled => "not_installed",
            InstallState::Partial => "partial",
        }
    }
}

/// Environment changes a plugin needs after install.
#[derive(Debug, Clone, Default)]
pub struct EnvSpec {
    /// Directories to prepend to the user PATH.
    pub paths: Vec<PathBuf>,
    /// Environment variable name -> value mappings.
    pub vars: Vec<(String, String)>,
}

impl EnvSpec {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Runtime context passed into plugin install/uninstall.
#[derive(Debug, Clone)]
pub struct InstallContext {
    /// Target folder, e.g. `C:\dev\flutter`.
    pub install_dir: PathBuf,
    /// Machine `dev` root, e.g. `C:\dev` (see `DEVKIT_HOME`).
    pub home: PathBuf,
    /// Optional SDK version / feature request from `install --version`.
    pub version: Option<String>,
    /// Optional release channel from `install --channel` (e.g. Flutter).
    pub channel: Option<String>,
}

/// Outcome of a successful plugin install.
#[derive(Debug, Clone)]
pub struct InstallResult {
    pub install_dir: PathBuf,
    pub message: String,
}

impl InstallResult {
    pub fn new(install_dir: PathBuf, message: impl Into<String>) -> Self {
        Self {
            install_dir,
            message: message.into(),
        }
    }
}

/// Reported status for a plugin.
#[derive(Debug, Clone)]
pub struct PluginStatus {
    pub state: InstallState,
    pub install_dir: Option<PathBuf>,
    pub detail: String,
}

impl PluginStatus {
    pub fn new(state: InstallState, install_dir: Option<PathBuf>) -> Self {
        Self {
            state,
            install_dir,
            detail: String::new(),
        }
    }

    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = detail.into();
        self
    }
}

/// Abstract installer plugin.
///
/// Implement this and register it in `plugins::all()` to expose
/// `devkit install <id>`.
pub trait Plugin: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str {
        ""
    }

    /// Return whether this plugin is installed.
    fn status(&self, ctx: &InstallContext) -> PluginStatus;

    /// Install the tool into `ctx.install_dir`.
    fn install(&self, ctx: &InstallContext) -> anyhow::Result<InstallResult>;

    /// Remove the tool from `ctx.install_dir`.
    fn uninstall(&self, ctx: &InstallContext) -> anyhow::Result<()>;

    /// Declare PATH entries and env vars for this install.
    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec;
}
