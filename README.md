# DevKit

[![Version](https://img.shields.io/badge/version-0.15.0-blue.svg)](CHANGELOG.md)
[![CI](https://github.com/axcel-blade/DevKit/actions/workflows/ci.yml/badge.svg)](https://github.com/axcel-blade/DevKit/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE.md)
[![Rust](https://img.shields.io/badge/rust-2021_edition-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Platforms](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)](docs/getting-started.md)
[![Plugins](https://img.shields.io/badge/plugins-35-blueviolet.svg)](docs/plugins.md)

DevKit is a **CLI application** that sets up developer environments on Windows, macOS, and Linux: download an SDK, install it under a machine `dev` folder, and configure PATH / environment variables through plugins.

It is **not** a published crate. Run it from this repository.

## Requirements

- Windows, macOS, or Linux
- An internet connection for the first run (to install Rust and fetch crates)
  and for SDK downloads; once Rust is installed the launchers start offline
- A Rust toolchain in the machine `dev` folder (`C:\dev\rust`, `/opt/dev/rust`,
  or `~/dev/rust`; override the root with `DEVKIT_HOME`). `devkit.bat` /
  `devkit.sh` install rustup stable there if `cargo` is missing.
- On Linux / WSL / macOS, a system C linker (`cc`). `devkit.sh` installs
  the distro toolchain (`build-essential`, `base-devel`, ...) when it is missing.

## Quick start

Running either launcher with **no arguments** (`make bat`, `make sh`, `devkit.bat`, or `./devkit.sh`) opens an interactive menu. On Windows it prints:

```text
Checking for available versions ...

DevKit
OS: Windows
Version: 0.15.0
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

On macOS and Linux the OS line is `OS: macOS` or `OS: Linux`. Chocolatey and MSYS2 are Windows-only, so those rows are left out there. Pick one or more numbers (`1 3 5`, `1,3`, or a range `2-4`) to install or uninstall each, `u <numbers>` to update those plugins, `a` to update every installed plugin that has a newer release, `r` to re-check versions, `q` to quit.

**Windows**

```bat
devkit.bat
devkit.bat doctor
devkit.bat list
devkit.bat install python
devkit.bat install go
devkit.bat install node
devkit.bat install jdk
devkit.bat install maven
devkit.bat install gradle
devkit.bat install junit
devkit.bat install pmd
devkit.bat install make
devkit.bat install chocolatey
devkit.bat install flutter
```

**macOS / Linux**

```bash
chmod +x devkit.sh
./devkit.sh
./devkit.sh doctor
./devkit.sh list
./devkit.sh install python
```

**Any OS**

```bash
cargo run --release -- doctor
cargo run --release -- install cmake
cargo run --release -- install kubectl
cargo run --release -- install docker
```

**Several plugins at once**

`install`, `update`, and `uninstall` take any number of plugin ids, space- or
comma-separated. Each plugin runs in turn; a failure doesn't stop the rest, and
a summary is printed at the end.

```bash
devkit install git go node
devkit install jdk,maven,gradle
devkit update git go          # reinstall only if a newer release exists
devkit update --all           # every installed plugin (--force to reinstall)
devkit uninstall go node
```

## Built-in plugins

Every plugin below is added to the same shared user `PATH` — DevKit does not
set per-tool `*_HOME` variables (`PYTHON_HOME`, `GOROOT`, etc.). The OS column
is where the menu lists that plugin and where `devkit install` accepts it.

| Plugin | OS | Installs |
|--------|----|----------|
| `anaconda` | Windows, macOS, Linux | Latest Anaconda3 distribution (conda + Python) |
| `android` | Windows, macOS, Linux | Android SDK cmdline-tools |
| `android-ndk` | Windows, macOS, Linux | Android NDK r29 native toolchain |
| `bun` | Windows, macOS, Linux | Latest Bun runtime |
| `chocolatey` | Windows | Chocolatey CLI into the DevKit folder; sets `ChocolateyInstall` |
| `cmake` | Windows, macOS, Linux | Latest CMake binary |
| `composer` | Windows, macOS, Linux | Composer PHAR + wrapper |
| `deno` | Windows, macOS, Linux | Latest Deno runtime |
| `docker` | Windows, macOS, Linux | Official static Docker CLI (client only) |
| `dotnet` | Windows, macOS, Linux | .NET SDK 10.0 LTS |
| `flutter` | Windows, macOS, Linux | Flutter SDK (`--channel` / `--version`) |
| `git` | Windows, macOS, Linux | MinGit (Windows); system wrappers (Unix) |
| `go` | Windows, macOS, Linux | Latest stable Go toolchain |
| `gradle` | Windows, macOS, Linux | Latest Gradle binary ZIP |
| `jdk` | Windows, macOS, Linux | Eclipse Temurin JDK (default 25; `--version`) |
| `junit` | Windows, macOS, Linux | JUnit Platform Console Standalone (`--version`) |
| `kubectl` | Windows, macOS, Linux | Latest stable kubectl |
| `make` | Windows, macOS, Linux | GNU Make 4.4.1 Chocolatey package (Windows, no admin); system wrappers (Unix) |
| `maven` | Windows, macOS, Linux | Latest Apache Maven |
| `mono` | Windows, macOS, Linux | Mono 6.12 (Win/macOS); system wrappers (Linux) |
| `msys2` | Windows | Portable MSYS2 base runtime |
| `mysql` | Windows, macOS, Linux | MySQL 8.4 LTS portable |
| `ninja` | Windows, macOS, Linux | Latest Ninja binary |
| `node` | Windows, macOS, Linux | Latest Node.js LTS (npm / npx) |
| `php` | Windows, macOS, Linux | PHP NTS (Windows); system wrappers (Unix) |
| `platform-tools` | Windows, macOS, Linux | Android platform-tools (`adb`) |
| `pmd` | Windows, macOS, Linux | PMD Source Code Analyzer binary (`--version`) |
| `pnpm` | Windows, macOS, Linux | Latest pnpm standalone |
| `postgresql` | Windows, macOS, Linux | PostgreSQL 18 EDB Windows binaries; system wrappers (Unix) |
| `python` | Windows, macOS, Linux | Portable CPython 3.14 (python-build-standalone) |
| `qemu` | Windows, macOS, Linux | QEMU 11.1 silent NSIS install (Windows); system wrappers (macOS/Linux) |
| `rust` | Windows, macOS, Linux | Rust stable via rustup |
| `sqlite` | Windows, macOS, Linux | Official sqlite-tools CLI |
| `terraform` | Windows, macOS, Linux | Latest Terraform |
| `uv` | Windows, macOS, Linux | Latest uv and uvx |

Default install root:

- Windows: `C:\dev\<plugin>`
- macOS / Linux: `/opt/dev/<plugin>` if writable, else `~/dev/<plugin>`

Override with `DEVKIT_HOME`. Open a **new terminal** after install so PATH updates apply.

## Documentation

| Doc | Purpose |
|-----|---------|
| [docs/getting-started.md](docs/getting-started.md) | Run the app and the Makefile targets |
| [docs/plugins.md](docs/plugins.md) | Write a plugin |
| [docs/architecture.md](docs/architecture.md) | How DevKit works |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [ROADMAP.md](ROADMAP.md) | Future plans |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |

## Makefile

From the repository root. If `make` is not installed yet, run `scripts\ensure-makefile.bat` (Windows) or `sh scripts/ensure-makefile.sh` (macOS / Linux) first. That installs GNU make, then runs `make`.

Before every target, `make` checks for GNU make, `cargo`, and a C compiler, and installs them when they are missing. `make sh` also checks for `bash`.

`make` with no target prints:

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

`make bat` and `make sh` with no extra words open the plugin menu. Extra words are passed to the launcher, so `make bat doctor` is `devkit.bat doctor` and `make sh install git` is `devkit.sh install git`.

Without make:

```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

Git Flow branches: `main`, `develop`, `feature/*`, `release/*`, `hotfix/*`.

## License

MIT — see [LICENSE.md](LICENSE.md). Author: Axcel Blade.
