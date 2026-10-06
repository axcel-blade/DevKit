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
- On Linux / WSL / macOS, a system C linker (`cc`). `devkit.sh` and the
  Makefile install the distro toolchain (`build-essential`, `base-devel`, ...)
  when it is missing.
- Write access to the `dev` install root
- Optional: GNU make. If `make` is not installed, run
  `scripts\ensure-makefile.bat` (Windows) or `sh scripts/ensure-makefile.sh`
  (macOS / Linux). That script installs make, then you can run `make`.

## Run DevKit

From the repository root, with GNU make:

```bash
make bat              # Windows launcher, plugin menu
make sh               # macOS / Linux launcher, plugin menu
make bat doctor       # devkit.bat doctor
make sh list          # devkit.sh list
make version
```

`make` checks for GNU make, `cargo`, and a C compiler before the target runs,
and installs them when they are missing. `make sh` also checks for `bash`.
With no extra words, `make bat` and `make sh` open the plugin menu. Extra
words are passed to the launcher.

Without make:

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
`devkit.bat`, or `make bat` / `make sh`) opens an interactive menu. On Windows the terminal prints:

```text
Checking for available versions ...

DevKit
OS: Windows
Version: 0.15.1
#    ID               NAME                     STATUS         INSTALLED        AVAILABLE
--------------------------------------------------------------------------------------------------------
1    anaconda         Anaconda                 not_installed  -                2026.07-1
2    android          Android SDK              not_installed  -                16111833
3    android-ndk      Android NDK              not_installed  -                -
4    bun              Bun                      not_installed  -                1.4.2
5    chocolatey       Chocolatey               installed      2.7.4            2.7.4 (up to date)
...
32   rust             Rust                     installed      1.98.1           1.99.0 (update available)

Enter number(s) to install/uninstall (e.g. '1 3 5' or '2-4'), 'u <number(s)>' to update, 'a' to update all, 'r' to refresh versions, or 'q' to quit:
```

On macOS and Linux the OS line is `OS: macOS` or `OS: Linux`. Chocolatey and MSYS2 are Windows-only and those rows are omitted there. An installed plugin that matches the newest release prints `up to date` in the AVAILABLE column; a newer release prints `update available`. Pick one or more numbers (`1 3 5`, `1,3`, or `2-4`) to install or uninstall each, `u <numbers>` to update those, or `a` to update every plugin that shows `update available` (`r` re-checks available versions, `q` quits). The launchers
rebuild DevKit on every run (instant when nothing changed), so the menu is
always the one from your current checkout. On Windows, if a previous DevKit
window still has `devkit.exe` open, the launcher renames that copy aside and
builds a new one:

```bash
./devkit.sh        # or: devkit.bat
```

## Makefile

Run these from the repository root. If `make` is not installed, run `scripts\ensure-makefile.bat` (Windows) or `sh scripts/ensure-makefile.sh` (macOS / Linux) first.

Before a target runs, `make` checks for GNU make, `cargo`, and a C compiler, and installs them when they are missing. `make sh` also checks for `bash`. `make` alone prints:

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

`make bat` and `make sh` with no extra words open the plugin menu. Extra words are passed through: `make bat install git` runs `devkit.bat install git`.

## Install a tool

```bash
make bat install git
make sh install git
cargo run --release -- install git
cargo run --release -- install make
cargo run --release -- install chocolatey
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
cargo run --release -- install git

# Unix
export DEVKIT_HOME="$HOME/my-dev"
cargo run --release -- install git
```

## Uninstall

```bash
cargo run --release -- uninstall git
cargo run --release -- uninstall go node     # several at once
```

Removes files under the `dev` folder and reverses PATH/env entries DevKit added.

## Troubleshooting: GitHub rate limit

Many plugins look up releases through the GitHub API, which allows 60
unauthenticated requests per hour. If you see "GitHub API rate limit
exceeded", wait for the reset or set `GITHUB_TOKEN` (or `GH_TOKEN`) to a
personal access token to raise the limit to 5,000/hour. Git on Windows falls
back to a non-API lookup automatically.
