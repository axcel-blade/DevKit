# Architecture

DevKit **0.15.4** is a CLI application with a plugin pipeline.

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
3. `devkit.sh` only: make sure the system C linker `cc` exists, since Rust links build scripts through it. If missing (common on fresh Linux / WSL), install the C toolchain with the detected package manager (`apt-get install build-essential`, `dnf`/`yum`/`zypper` gcc, `pacman` base-devel, `apk` build-base; `xcode-select --install` on macOS), using `sudo` when not root; otherwise print the command and exit.
4. Always run `cargo build --release` (a no-op when up to date) so the binary matches the checkout. On Windows, if `devkit.exe` is still running (`os error 5`), the launcher renames that copy aside and builds again. If the build fails but an older binary exists, it is used with a warning.
5. Exec the binary. With no arguments the launcher runs `devkit menu`, the interactive plugin menu.

## Makefile

`make bat` and `make sh` run the launchers. With no extra words they open the menu. Extra words are arguments: `make bat doctor` runs `devkit.bat doctor`. Before any target, the Makefile runs `scripts/ensure-makefile.bat` or `scripts/ensure-makefile.sh`, which installs GNU make, `cargo`, or a C compiler when they are missing. `make sh` also installs `bash` when it is missing. If `make` is not on `PATH`, run that script directly, then run `make` again.

`make` prints:

```text
DevKit
  make build          Debug build
  make release        Release build
  make test           Run tests
  make test-release   Run tests in release mode
  make fmt            Format Rust sources
  make fmt-check      Check formatting
  make clippy         Lint with warnings denied
  make doctor         Build and run devkit doctor
  make version        Print the DevKit version
  make run            Open the plugin menu via cargo
  make bat            Run devkit.bat and show the menu
  make sh             Run devkit.sh and show the menu
  make bat doctor     Run devkit.bat with arguments
  make sh list        Run devkit.sh with arguments
  make ci             fmt-check, clippy, release build, release tests, doctor
  make clean          Remove build artifacts
```

The menu itself prints:

```text
Checking for available versions ...

DevKit
OS: Windows
Version: 0.15.4
#    ID               NAME                     STATUS         INSTALLED        AVAILABLE
--------------------------------------------------------------------------------------------------------
1    anaconda         Anaconda                 not_installed  -                2026.07-1
5    chocolatey       Chocolatey               installed      2.7.4            2.7.4 (up to date)
32   rust             Rust                     installed      1.98.1           1.99.0 (update available)

Enter number(s) to install/uninstall (e.g. '1 3 5' or '2-4'), 'u <number(s)>' to update, 'a' to update all, 'r' to refresh versions, or 'q' to quit:
```

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

The interactive menu (`cmd_menu` in `src/cli.rs`) calls each plugin's
`latest_version` once on scoped threads when it opens (again on `r`), after a
single connectivity probe, and reads `installed_version` on every redraw.
`u <number>` reinstalls that plugin when its installed version differs from
the available one. `a` does that for every such plugin. Both sides go through
`plugin_utils::normalize_version` so tags like `go1.27.1`,
`bun-v1.4.2`, and `1.27.1` compare equal. Plugins that do not list the detected
OS in `supported_os` are omitted from the menu. The
header prints `DevKit`, `OS: <host>`, and `Version: <version>`. `devkit install`
refuses unsupported plugins and prints `Cannot install …` plus a `Reason:` line.
