//! DevKit command-line interface.
//!
//! Commands: list, install, uninstall, status, doctor.
//! Install flow: plugin.install() then EnvManager.apply(plugin.env_spec()).

use crate::env::EnvManager;
use crate::paths::{ensure_home, home, plugin_install_dir};
use crate::platform::{is_windows, os_label};
use crate::plugin::{InstallContext, InstallState, Plugin};
use crate::registry::default_registry;
use crate::theme;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "devkit", about = "DevKit - developer environment setup application.", version = VERSION)]
struct Cli {
    // Optional so a bare `devkit` (e.g. double-clicked from Explorer, or run
    // via devkit.bat/devkit.sh with no arguments) falls through to the
    // interactive menu instead of clap erroring "a subcommand is required".
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// List available plugins
    List,
    /// Install a plugin
    Install {
        /// Plugin id (e.g. hello)
        plugin: String,
        /// Reinstall even if already installed
        #[arg(long)]
        force: bool,
        /// SDK version (flutter, JDK feature, JUnit, or PMD release)
        #[arg(long = "version", value_name = "VER")]
        sdk_version: Option<String>,
        /// Release channel for flutter (stable, beta, or dev)
        #[arg(long, value_name = "NAME")]
        channel: Option<String>,
    },
    /// Uninstall a plugin
    Uninstall {
        /// Plugin id
        plugin: String,
    },
    /// Show plugin install and env status
    Status {
        /// Plugin id
        plugin: String,
    },
    /// Check DevKit environment health
    Doctor,
    /// Interactive menu to install/uninstall plugins
    Menu,
}

fn context(
    plugin_id: &str,
    version: Option<String>,
    channel: Option<String>,
) -> anyhow::Result<InstallContext> {
    ensure_home()?;
    Ok(InstallContext {
        install_dir: plugin_install_dir(plugin_id),
        home: home(),
        version,
        channel,
    })
}

fn cmd_list() -> anyhow::Result<i32> {
    let registry = default_registry();
    let plugins = registry.all();
    if plugins.is_empty() {
        println!("No plugins registered.");
        return Ok(0);
    }
    println!(
        "{}",
        theme::bold(&format!("{:<16} {:<20} STATUS", "ID", "NAME"))
    );
    println!("{}", theme::dim(&"-".repeat(50)));
    for plugin in plugins {
        let ctx = context(plugin.id(), None, None)?;
        let status = plugin.status(&ctx);
        println!(
            "{:<16} {:<20} {}",
            plugin.id(),
            plugin.name(),
            theme::status_label(status.state.as_str())
        );
    }
    Ok(0)
}

/// Run a plugin's install step and apply its env_spec. Shared by `cmd_install`
/// (explicit `devkit install <id>`) and `cmd_menu` (interactive picker) so
/// both go through one code path.
fn perform_install(plugin: &dyn Plugin, ctx: &InstallContext) -> anyhow::Result<()> {
    println!(
        "{} {} ({}) into {} ...",
        theme::cyan("Installing"),
        plugin.name(),
        plugin.id(),
        ctx.install_dir.display()
    );
    let result = plugin.install(ctx)?;
    let spec = plugin.env_spec(ctx);
    EnvManager::new().apply(&spec)?;
    let msg = if result.message.is_empty() {
        "done".to_string()
    } else {
        result.message
    };
    println!("{} {}: {}", theme::green("Installed"), plugin.id(), msg);
    println!(
        "{}",
        theme::dim("Environment updated. Open a new terminal for PATH/env changes to take effect.")
    );
    Ok(())
}

/// Revert a plugin's env_spec and run its uninstall step. Shared by
/// `cmd_uninstall` and `cmd_menu`.
fn perform_uninstall(plugin: &dyn Plugin, ctx: &InstallContext) -> anyhow::Result<()> {
    let spec = plugin.env_spec(ctx);
    EnvManager::new().revert(&spec)?;
    plugin.uninstall(ctx)?;
    println!("{} {}.", theme::yellow("Uninstalled"), plugin.id());
    println!(
        "{}",
        theme::dim("Environment updated. Open a new terminal for PATH/env changes to take effect.")
    );
    Ok(())
}

fn cmd_install(
    plugin_id: &str,
    force: bool,
    sdk_version: Option<String>,
    channel: Option<String>,
) -> anyhow::Result<i32> {
    let registry = default_registry();
    let plugin = match registry.require(plugin_id) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return Ok(1);
        }
    };

    let ctx = context(plugin.id(), sdk_version, channel)?;
    let status = plugin.status(&ctx);
    if status.state == InstallState::Installed && !force {
        println!(
            "{} is already installed at {}",
            plugin.id(),
            status
                .install_dir
                .map(|p| p.display().to_string())
                .unwrap_or_default()
        );
        println!("Use --force to reinstall.");
        return Ok(0);
    }

    if ctx.channel.is_some() && plugin.id() != "flutter" {
        eprintln!(
            "Note: --channel is used by the flutter plugin; ignored for {}.",
            plugin.id()
        );
    }
    if ctx.version.is_some() && !matches!(plugin.id(), "flutter" | "jdk" | "junit" | "pmd") {
        eprintln!(
            "Note: --version is used by flutter/jdk/junit/pmd; ignored for {}.",
            plugin.id()
        );
    }

    perform_install(plugin, &ctx)?;
    Ok(0)
}

fn cmd_uninstall(plugin_id: &str) -> anyhow::Result<i32> {
    let registry = default_registry();
    let plugin = match registry.require(plugin_id) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return Ok(1);
        }
    };

    let ctx = context(plugin.id(), None, None)?;
    let status = plugin.status(&ctx);
    if status.state == InstallState::NotInstalled {
        println!("{} is not installed.", plugin.id());
        return Ok(0);
    }

    perform_uninstall(plugin, &ctx)?;
    Ok(0)
}

fn cmd_status(plugin_id: &str) -> anyhow::Result<i32> {
    let registry = default_registry();
    let plugin = match registry.require(plugin_id) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}");
            return Ok(1);
        }
    };

    let ctx = context(plugin.id(), None, None)?;
    let status = plugin.status(&ctx);
    println!("Plugin:      {} ({})", plugin.id(), plugin.name());
    println!(
        "State:       {}",
        theme::status_label(status.state.as_str())
    );
    if let Some(dir) = &status.install_dir {
        println!("Install dir: {}", dir.display());
    }
    if !status.detail.is_empty() {
        println!("Detail:      {}", status.detail);
    }

    let spec = plugin.env_spec(&ctx);
    let checks = EnvManager::new().check(&spec)?;
    if !checks.is_empty() {
        println!("Environment:");
        for (key, ok) in &checks {
            println!("  {} {key}", theme::check_mark(*ok));
        }
    }
    Ok(if status.state == InstallState::Installed {
        0
    } else {
        1
    })
}

/// Locate `cargo`/`rustc` the way `where` (Windows) / `which` (Unix) would,
/// then fall back to the machine `dev` folder toolchain (`<dev>/rust/cargo/bin`).
/// Returns a single doctor line: version plus the resolved path.
fn rust_toolchain_line(bin_name: &str) -> String {
    let from_path = which::which(bin_name).ok();
    let from_dev = {
        let exe = if is_windows() {
            format!("{bin_name}.exe")
        } else {
            bin_name.to_string()
        };
        let candidate = plugin_install_dir("rust")
            .join("cargo")
            .join("bin")
            .join(exe);
        candidate.is_file().then_some(candidate)
    };
    let path: Option<PathBuf> = from_path.or(from_dev);
    match path {
        Some(path) => {
            let ver = std::process::Command::new(&path)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| format!("{bin_name} (version unknown)"));
            format!("{ver}  ({})", path.display())
        }
        None => {
            "not found  (run the launcher to install Rust into the machine dev folder)".to_string()
        }
    }
}

fn cmd_doctor() -> anyhow::Result<i32> {
    let root = home();
    println!("{}", theme::bold(&format!("DevKit {VERSION}")));
    println!("Platform:     {} ({})", os_label(), std::env::consts::OS);
    // Replaces the old Python-app `Python: <sys.version>` / `where python` line.
    println!("Rustc:        {}", rust_toolchain_line("rustc"));
    println!("Cargo:        {}", rust_toolchain_line("cargo"));
    println!(
        "Dev root:     {}  (override with DEVKIT_HOME)",
        root.display()
    );
    println!("Example path: {}", plugin_install_dir("flutter").display());

    match ensure_home() {
        Ok(root) => {
            let probe = root.join(".write_probe");
            match std::fs::write(&probe, "ok") {
                Ok(()) => {
                    let _ = std::fs::remove_file(&probe);
                    println!("Root writable: {}", theme::green("yes"));
                }
                Err(e) => {
                    println!("Root writable: {} ({e})", theme::red("no"));
                    return Ok(1);
                }
            }
        }
        Err(e) => {
            println!("Root writable: {} ({e})", theme::red("no"));
            return Ok(1);
        }
    }

    // Advisory only — doesn't fail the command, since doctor is still useful
    // offline (e.g. checking the install root or env backend).
    if crate::download::has_internet_access() {
        println!("Internet:     {}", theme::green("yes"));
    } else {
        println!(
            "Internet:     {} (SDK downloads need network access)",
            theme::yellow("no")
        );
    }

    println!("Env backend:  {}", EnvManager::new().backend_description());
    if !crate::platform::is_windows() {
        println!("             Open a new terminal (or `source ~/.devkit/env.sh`) after install.");
    }

    let registry = default_registry();
    let ids = registry.ids();
    println!(
        "Plugins:      {}",
        if ids.is_empty() {
            "(none)".to_string()
        } else {
            ids.join(", ")
        }
    );
    Ok(0)
}

/// Look up the newest available version of every plugin in parallel.
///
/// Each lookup may hit a release API, so they run on scoped threads rather
/// than one after another (30+ sequential HTTP calls would stall the menu).
/// Returns one entry per plugin, in the same order; `None` means unknown
/// (offline, no resolver, or the lookup failed).
fn fetch_latest_versions(plugins: &[&dyn Plugin]) -> Vec<Option<String>> {
    use crate::plugin_utils::{looks_like_version, normalize_version};
    // One connectivity probe up front so offline users don't wait on every
    // plugin's own probe timing out.
    if !crate::download::has_internet_access() {
        return vec![None; plugins.len()];
    }
    std::thread::scope(|scope| {
        let handles: Vec<_> = plugins
            .iter()
            .map(|plugin| {
                scope.spawn(move || {
                    let ctx = context(plugin.id(), None, None).ok()?;
                    plugin
                        .latest_version(&ctx)
                        .ok()
                        .flatten()
                        .filter(|v| looks_like_version(v))
                        .map(|v| normalize_version(&v))
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().ok().flatten())
            .collect()
    })
}

/// Pad an already-colored cell to `width` using its plain-text length, since
/// ANSI escape codes would otherwise throw off `{:<width}` alignment.
fn pad_cell(colored: String, plain_len: usize, width: usize) -> String {
    format!("{colored}{}", " ".repeat(width.saturating_sub(plain_len)))
}

/// Render the AVAILABLE column: the latest version, flagged in yellow when it
/// differs from what's installed, green when the install is current.
fn available_cell(installed: Option<&str>, latest: Option<&str>) -> String {
    use crate::plugin_utils::normalize_version;
    match (installed, latest) {
        (_, None) => theme::dim("-"),
        (Some(i), Some(l)) if normalize_version(i) == normalize_version(l) => {
            theme::green(&format!("{l} (up to date)"))
        }
        (Some(_), Some(l)) => theme::yellow(&format!("{l} (update available)")),
        (None, Some(l)) => l.to_string(),
    }
}

/// Interactive text menu: lists every plugin with its current status, the
/// installed version, and the newest available version, lets the user pick
/// one by number, and toggles install/uninstall on it — installs if
/// missing/partial, uninstalls if already installed. Loops until the user
/// quits. This is what a bare `devkit` (no subcommand) runs, so
/// double-clicking `devkit.bat`/`devkit.sh` gives a usable menu instead of a
/// clap usage error.
fn cmd_menu() -> anyhow::Result<i32> {
    use std::io::{self, Write};

    let registry = default_registry();
    let plugins = registry.all();
    if plugins.is_empty() {
        println!("No plugins registered.");
        return Ok(0);
    }

    // Available versions are fetched once (network) and reused across loop
    // iterations; 'r' re-fetches. Installed versions are local reads, so they
    // are refreshed every loop alongside status.
    println!("{}", theme::dim("Checking for available versions ..."));
    let mut latest = fetch_latest_versions(&plugins);
    if latest.iter().all(Option::is_none) {
        println!(
            "{}",
            theme::yellow("Could not look up available versions (offline?). Press 'r' to retry.")
        );
    }

    loop {
        println!();
        println!(
            "{}",
            theme::bold(&format!("DevKit {VERSION} — plugin menu"))
        );
        println!(
            "{}",
            theme::bold(&format!(
                "{:<4} {:<16} {:<24} {:<14} {:<16} AVAILABLE",
                "#", "ID", "NAME", "STATUS", "INSTALLED"
            ))
        );
        println!("{}", theme::dim(&"-".repeat(104)));

        // Re-check status every loop so the menu reflects what the last
        // action actually did, and remember which are installed so the
        // chosen action (install vs uninstall) doesn't need a second lookup.
        let mut installed = Vec::with_capacity(plugins.len());
        for (i, plugin) in plugins.iter().enumerate() {
            let ctx = context(plugin.id(), None, None)?;
            let status = plugin.status(&ctx);
            let state = status.state.as_str();
            // Only report a version for a complete install; a partial dir may
            // hold a stale marker from an interrupted run.
            let current = if status.state == InstallState::Installed {
                plugin
                    .installed_version(&ctx)
                    .map(|v| crate::plugin_utils::normalize_version(&v))
            } else {
                None
            };
            let current_cell = match &current {
                Some(v) => pad_cell(v.clone(), v.chars().count(), 16),
                None => pad_cell(theme::dim("-"), 1, 16),
            };
            println!(
                "{:<4} {:<16} {:<24} {} {} {}",
                i + 1,
                plugin.id(),
                plugin.name(),
                pad_cell(theme::status_label(state), state.len(), 14),
                current_cell,
                available_cell(current.as_deref(), latest[i].as_deref())
            );
            installed.push(status.state == InstallState::Installed);
        }

        println!();
        print!(
            "{}",
            theme::cyan(
                "Enter a number to install/uninstall, 'r' to refresh versions, or 'q' to quit: "
            )
        );
        io::stdout().flush()?;

        let mut line = String::new();
        // read_line returns Ok(0) on EOF (piped/closed stdin) — exit instead
        // of spinning forever on empty reads in a non-interactive run.
        if io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        let input = line.trim();
        if input.is_empty() || input.eq_ignore_ascii_case("q") {
            break;
        }
        if input.eq_ignore_ascii_case("r") {
            println!("{}", theme::dim("Checking for available versions ..."));
            latest = fetch_latest_versions(&plugins);
            continue;
        }

        let choice: usize = match input.parse() {
            Ok(n) if n >= 1 && n <= plugins.len() => n,
            _ => {
                println!("{} {input}", theme::red("Invalid choice:"));
                continue;
            }
        };
        let plugin = plugins[choice - 1];
        let ctx = context(plugin.id(), None, None)?;

        let outcome = if installed[choice - 1] {
            perform_uninstall(plugin, &ctx)
        } else {
            perform_install(plugin, &ctx)
        };
        if let Err(e) = outcome {
            eprintln!("{} {e}", theme::red("Error:"));
        }
    }
    Ok(0)
}

pub fn main(argv: Option<Vec<String>>) -> anyhow::Result<i32> {
    let cli = match argv {
        Some(args) => Cli::parse_from(args),
        None => Cli::parse(),
    };
    match cli.command {
        Some(Command::List) => cmd_list(),
        Some(Command::Install {
            plugin,
            force,
            sdk_version,
            channel,
        }) => cmd_install(&plugin, force, sdk_version, channel),
        Some(Command::Uninstall { plugin }) => cmd_uninstall(&plugin),
        Some(Command::Status { plugin }) => cmd_status(&plugin),
        Some(Command::Doctor) => cmd_doctor(),
        Some(Command::Menu) | None => cmd_menu(),
    }
}
