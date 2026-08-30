# Architecture

DevKit **0.8.0** is a CLI application with a plugin pipeline.

```text
target/release/devkit (binary) / devkit.bat / devkit.sh
        │
        ▼
   cli::main
        │
        ├─ PluginRegistry  (registry.rs, built from plugins::all())
        ├─ Plugin::install / uninstall / status / env_spec
        ├─ download / installers  (ZIP, tar, MSI, PKG)
        └─ EnvManager  (Windows registry | Unix env.sh)
```

## Install root

`paths::home()` resolves:

1. `DEVKIT_HOME` if set
2. Else Windows `C:\dev`, or `/opt/dev` / `~/dev` on Unix

Each plugin installs to `<home>/<plugin-id>`.

## Environment

After a successful install, the CLI calls `EnvManager::apply(&plugin.env_spec(&ctx))`:

- **Windows:** user `Path` and variables via the `winreg` crate
- **macOS / Linux:** writes `~/.devkit/env.sh` and ensures the primary shell profile sources it

Uninstall reverses those entries, then deletes the install directory.

## Plugins

Plugins are structs implementing the `Plugin` trait (`src/plugin.rs`). Rust
has no runtime module scan like Python's `pkgutil`, so `src/plugins/mod.rs`
explicitly lists every plugin — add new modules under `src/plugins/` and
register them there.
