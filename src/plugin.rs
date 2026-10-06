//! Plugin contract for DevKit installers.

use crate::platform::{HostOS, DESKTOP_HOSTS};
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

/// Text `devkit install` prints when the plugin does not support this OS.
pub fn cannot_install_message(plugin: &dyn Plugin) -> String {
    format!(
        "Cannot install {} on {}.\nReason: {}",
        plugin.id(),
        crate::platform::os_label(),
        plugin.unavailable_reason()
    )
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

    /// Operating systems this plugin can be installed on.
    ///
    /// Defaults to Windows, macOS, and Linux. Override for plugins that only
    /// ship on a subset (Chocolatey and MSYS2 are Windows-only). The menu
    /// lists only plugins that include the host. `devkit install` refuses
    /// the rest and prints [`cannot_install_message`].
    fn supported_os(&self) -> &'static [HostOS] {
        DESKTOP_HOSTS
    }

    /// Whether `os` is one of [`supported_os`](Plugin::supported_os).
    fn supports_os(&self, os: HostOS) -> bool {
        self.supported_os().contains(&os)
    }

    /// Why this plugin cannot be installed on a host outside [`supported_os`](Plugin::supported_os).
    fn unavailable_reason(&self) -> String {
        format!(
            "{} is only supported on {}.",
            self.name(),
            crate::platform::format_os_list(self.supported_os())
        )
    }

    /// Return whether this plugin is installed.
    fn status(&self, ctx: &InstallContext) -> PluginStatus;

    /// Install the tool into `ctx.install_dir`.
    fn install(&self, ctx: &InstallContext) -> anyhow::Result<InstallResult>;

    /// Remove the tool from `ctx.install_dir`.
    fn uninstall(&self, ctx: &InstallContext) -> anyhow::Result<()>;

    /// Declare PATH entries and env vars for this install.
    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec;

    /// Version currently installed in `ctx.install_dir`, if known.
    ///
    /// Default: read the `.devkit-<id>` marker that most plugins write after a
    /// successful install. Markers that don't hold a version (e.g. `stable`,
    /// `system-wrapper`) yield `None`. Override when the SDK records its own
    /// version (JDK `release`, Flutter `version`, `rustc --version`, ...).
    fn installed_version(&self, ctx: &InstallContext) -> Option<String> {
        crate::plugin_utils::read_marker_version(&ctx.install_dir, self.id())
    }

    /// Plugins that must be present before this one is installed.
    ///
    /// Each entry is `(plugin_id, binary)`. A prerequisite counts as present
    /// when that plugin reports `Installed` or `binary` is already on PATH
    /// (e.g. a system JDK). Otherwise DevKit installs it first.
    fn prerequisites(&self) -> &'static [(&'static str, &'static str)] {
        &[]
    }

    /// Newest version DevKit would install right now, if it can be looked up.
    ///
    /// Default `Ok(None)` (unknown). Implementations usually reuse the same
    /// resolver `install` calls, so the string matches what `installed_version`
    /// later reports. May hit the network — callers should run it off the
    /// main thread and cache the result.
    fn latest_version(&self, _ctx: &InstallContext) -> anyhow::Result<Option<String>> {
        Ok(None)
    }
}
