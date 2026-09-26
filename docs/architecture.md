# Architecture

DevKit **0.9.2** is a CLI application with a plugin pipeline.

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

## Launcher bootstrap

`devkit.bat` and `devkit.sh` run before the compiled binary:

1. Resolve the same `dev` root as `paths::home()` (`DEVKIT_HOME` or the OS default).
2. Look for `cargo` under `<dev>/rust/cargo/bin`. If missing, probe the network (`static.rust-lang.org:443`, failing with a connect-to-internet error if offline), download the installer as `rustup-init` (rustup chooses its mode from its file name) and install the stable toolchain into `<dev>/rust` (`CARGO_HOME` / `RUSTUP_HOME`, `--no-modify-path`).
3. Always run `cargo build --release` (a no-op when up to date) so the binary matches the checkout. If the build fails but an older binary exists, it is used with a warning.
4. Exec the binary. With no arguments the launcher runs `devkit menu`, the interactive plugin menu.

`devkit doctor` reports `Rustc` and `Cargo` the same way `where rustc` / `where cargo` would (PATH first, then `<dev>/rust/cargo/bin`).

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
