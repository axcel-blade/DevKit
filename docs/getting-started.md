# Getting started

## Requirements

- An internet connection for the first run (rustup, crates.io) and for SDK
  downloads. Once Rust is installed in the `dev` folder the launchers start
  offline; they only exit with a connect-to-internet error when Rust still
  needs to be installed.
- A Rust toolchain in the machine `dev` folder (`C:\dev\rust`, `/opt/dev/rust`,
  or `~/dev/rust`; override the root with `DEVKIT_HOME`). `devkit.bat` /
  `devkit.sh` install rustup stable there when `cargo` is missing. Running
  `cargo run` yourself still needs a toolchain on PATH (the one in `dev/rust`
  works if you export `CARGO_HOME` / `RUSTUP_HOME` first).
- Write access to the `dev` install root

## Run DevKit

From the repository root:

```bash
cargo run --release -- --version
cargo run --release -- doctor
cargo run --release -- list
```

Windows shortcut (rustup into `C:\dev\rust` if needed, then build and run):

```bat
devkit.bat doctor
```

macOS / Linux (same bootstrap behavior):

```bash
chmod +x devkit.sh
./devkit.sh doctor
```

Either launcher run with **no arguments** (including double-clicking
`devkit.bat`) opens an interactive menu instead — lists every plugin with its
status, installed version, and newest available version (marked
`update available` or `up to date`), and lets you pick a number to install or
uninstall it (`r` re-checks available versions, `q` quits). The launchers
rebuild DevKit on every run (instant when nothing changed), so the menu is
always the one from your current checkout:

```bash
./devkit.sh        # or: devkit.bat
```

## Install a tool

```bash
cargo run --release -- install git
cargo run --release -- install gradle
cargo run --release -- install maven
cargo run --release -- install junit
cargo run --release -- install pmd
cargo run --release -- install node
cargo run --release -- install php
cargo run --release -- install mysql
cargo run --release -- install android
cargo run --release -- install jdk
cargo run --release -- install flutter
cargo run --release -- status gradle
```

After install, **open a new terminal** so PATH and env vars reload.

### Flutter / JDK / JUnit / PMD options

```bash
cargo run --release -- install flutter --channel beta
cargo run --release -- install flutter --channel stable --version 3.24
cargo run --release -- install jdk --version 17
cargo run --release -- install jdk --version 25
cargo run --release -- install junit --version 1.11.4
cargo run --release -- install pmd --version 7.26.0
```

### Environment backends

| OS | How env is stored |
|----|-------------------|
| Windows | Current-user registry |
| macOS | `~/.devkit/env.sh` sourced from `~/.zshrc` |
| Linux | `~/.devkit/env.sh` sourced from `~/.bashrc` |

## Custom install root

```bash
# Windows PowerShell
$env:DEVKIT_HOME = "D:\dev"
cargo run --release -- install hello

# Unix
export DEVKIT_HOME="$HOME/my-dev"
cargo run --release -- install hello
```

## Uninstall

```bash
cargo run --release -- uninstall hello
```

Removes files under the `dev` folder and reverses PATH/env entries DevKit added.
