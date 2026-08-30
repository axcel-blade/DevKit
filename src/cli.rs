//! DevKit command-line interface.
//!
//! Commands: list, install, uninstall, status, doctor.
//! Install flow: plugin.install() then EnvManager.apply(plugin.env_spec()).

use crate::env::EnvManager;
use crate::paths::{ensure_home, home, plugin_install_dir};
use crate::platform::os_label;
use crate::plugin::{InstallContext, InstallState};
use crate::registry::default_registry;
use clap::{Parser, Subcommand};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "devkit", about = "DevKit - developer environment setup application.", version = VERSION)]
struct Cli {
    #[command(subcommand)]
    command: Command,
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
        /// SDK version (flutter release, or JDK feature like 17/21)
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
    println!("{:<16} {:<20} STATUS", "ID", "NAME");
    println!("{}", "-".repeat(50));
    for plugin in plugins {
        let ctx = context(plugin.id(), None, None)?;
        let status = plugin.status(&ctx);
        println!(
            "{:<16} {:<20} {}",
            plugin.id(),
            plugin.name(),
            status.state.as_str()
        );
    }
    Ok(0)
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
    if ctx.version.is_some() && !matches!(plugin.id(), "flutter" | "jdk") {
        eprintln!(
            "Note: --version is used by flutter/jdk; ignored for {}.",
            plugin.id()
        );
    }

    println!(
        "Installing {} ({}) into {} ...",
        plugin.name(),
        plugin.id(),
        ctx.install_dir.display()
    );
    let result = plugin.install(&ctx)?;
    let spec = plugin.env_spec(&ctx);
    EnvManager::new().apply(&spec)?;
    let msg = if result.message.is_empty() {
        "done".to_string()
    } else {
        result.message
    };
    println!("Installed {}: {}", plugin.id(), msg);
    println!("Environment updated. Open a new terminal for PATH/env changes to take effect.");
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

    let spec = plugin.env_spec(&ctx);
    EnvManager::new().revert(&spec)?;
    plugin.uninstall(&ctx)?;
    println!("Uninstalled {}.", plugin.id());
    println!("Environment updated. Open a new terminal for PATH/env changes to take effect.");
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
    println!("State:       {}", status.state.as_str());
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
            let mark = if *ok { "ok" } else { "missing" };
            println!("  [{mark}] {key}");
        }
    }
    Ok(if status.state == InstallState::Installed {
        0
    } else {
        1
    })
}

fn cmd_doctor() -> anyhow::Result<i32> {
    let root = home();
    println!("DevKit {VERSION}");
    println!("Platform:     {} ({})", os_label(), std::env::consts::OS);
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
                    println!("Root writable: yes");
                }
                Err(e) => {
                    println!("Root writable: no ({e})");
                    return Ok(1);
                }
            }
        }
        Err(e) => {
            println!("Root writable: no ({e})");
            return Ok(1);
        }
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

pub fn main(argv: Option<Vec<String>>) -> anyhow::Result<i32> {
    let cli = match argv {
        Some(args) => Cli::parse_from(args),
        None => Cli::parse(),
    };
    match cli.command {
        Command::List => cmd_list(),
        Command::Install {
            plugin,
            force,
            sdk_version,
            channel,
        } => cmd_install(&plugin, force, sdk_version, channel),
        Command::Uninstall { plugin } => cmd_uninstall(&plugin),
        Command::Status { plugin } => cmd_status(&plugin),
        Command::Doctor => cmd_doctor(),
    }
}
