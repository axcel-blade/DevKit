# Getting started

## Requirements

- An internet connection. The launchers exit with an error if the network is
  unreachable (needed for rustup, crates.io, and later SDK downloads).
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

Windows shortcut (internet check, then rustup into `C:\dev\rust` if needed):

```bat
devkit.bat doctor
```

macOS / Linux (same bootstrap behavior):

```bash
chmod +x devkit.sh
./devkit.sh doctor
```

Either launcher run with **no arguments** opens an interactive menu instead —
lists every plugin with its status and lets you pick a number to install or
uninstall it:

```bash
./devkit.sh        # or: devkit.bat
```

## Install a tool

```bash
cargo run --release -- install git
cargo run --release -- install gradle
cargo run --release -- install node
cargo run --release -- install php
cargo run --release -- install mysql
cargo run --release -- install android
cargo run --release -- install jdk
cargo run --release -- install flutter
cargo run --release -- status gradle
```

After install, **open a new terminal** so PATH and env vars reload.

### Flutter / JDK options

```bash
cargo run --release -- install flutter --channel beta
cargo run --release -- install flutter --channel stable --version 3.24
cargo run --release -- install jdk --version 17
cargo run --release -- install jdk --version 21
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
