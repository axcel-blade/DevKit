# Architecture

DevKit **0.6.1** is a CLI application with a plugin pipeline.

```text
main.py / devkit.bat / devkit.sh
        │
        ▼
   devkit.cli
        │
        ├─ PluginRegistry  (loads devkit.plugins.*)
        ├─ Plugin.install / uninstall / status / env_spec
        ├─ download / installers  (ZIP, tar, MSI, PKG)
        └─ EnvManager  (Windows registry | Unix env.sh)
```

## Install root

`devkit.paths.home()` resolves:

1. `DEVKIT_HOME` if set
2. Else Windows `C:\dev`, or `/opt/dev` / `~/dev` on Unix

Each plugin installs to `<home>/<plugin-id>`.

## Environment

After a successful install, the CLI calls `EnvManager.apply(plugin.env_spec())`:

- **Windows:** user `Path` and variables via `winreg`
- **macOS / Linux:** writes `~/.devkit/env.sh` and ensures the primary shell profile sources it

Uninstall reverses those entries, then deletes the install directory.

## Plugins

Plugins are plain Python classes. No separate package index in 0.6.1 — add modules under `src/devkit/plugins/`.
