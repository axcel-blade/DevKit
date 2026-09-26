# DevKit

**Version:** 0.9.2

DevKit is a **CLI application** that sets up developer environments on Windows, macOS, and Linux: download an SDK, install it under a machine `dev` folder, and configure PATH / environment variables through plugins.

It is **not** a published crate. Run it from this repository.

## Requirements

- Windows, macOS, or Linux
- An internet connection for the first run (to install Rust and fetch crates)
  and for SDK downloads; once Rust is installed the launchers start offline
- A Rust toolchain in the machine `dev` folder (`C:\dev\rust`, `/opt/dev/rust`,
  or `~/dev/rust`; override the root with `DEVKIT_HOME`). `devkit.bat` /
  `devkit.sh` install rustup stable there if `cargo` is missing.

## Quick start

Running either launcher with **no arguments** opens an interactive menu
listing every plugin with its install status — pick a number to install or
uninstall it, `q` to quit.

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

## Built-in plugins

Every plugin below is added to the same shared user `PATH` — DevKit does not
set per-tool `*_HOME` variables (`PYTHON_HOME`, `GOROOT`, etc.).

| Plugin | Installs |
|--------|----------|
| `python` | Portable CPython 3.14 (python-build-standalone) |
| `go` | Latest stable Go toolchain |
| `rust` | Rust stable via rustup |
| `dotnet` | .NET SDK 10.0 LTS |
| `node` | Latest Node.js LTS (npm / npx) |
| `pnpm` | Latest pnpm standalone |
| `deno` | Latest Deno runtime |
| `bun` | Latest Bun runtime |
| `jdk` | Eclipse Temurin JDK (default 25; `--version`) |
| `maven` | Latest Apache Maven |
| `gradle` | Latest Gradle binary ZIP |
| `junit` | JUnit Platform Console Standalone (`--version`) |
| `pmd` | PMD Source Code Analyzer binary (`--version`) |
| `cmake` | Latest CMake binary |
| `ninja` | Latest Ninja binary |
| `git` | MinGit (Windows); system wrappers (Unix) |
| `php` | PHP NTS (Windows); system wrappers (Unix) |
| `composer` | Composer PHAR + wrapper |
| `mysql` | MySQL 8.4 LTS portable |
| `postgresql` | PostgreSQL 18 EDB Windows binaries; system wrappers (Unix) |
| `sqlite` | Official sqlite-tools CLI |
| `android` | Android SDK cmdline-tools |
| `platform-tools` | Android platform-tools (`adb`) |
| `flutter` | Flutter SDK (`--channel` / `--version`) |
| `kubectl` | Latest stable kubectl |
| `docker` | Official static Docker CLI (client only) |
| `terraform` | Latest Terraform |
| `mono` | Mono 6.12 (Win/macOS); system wrappers (Linux) |
| `msys2` | Portable MSYS2 base runtime (Windows only) |
| `qemu` | QEMU 11.1 silent NSIS install (Windows); system wrappers (macOS/Linux) |
| `hello` | Demo ZIP install |

Default install root:

- Windows: `C:\dev\<plugin>`
- macOS / Linux: `/opt/dev/<plugin>` if writable, else `~/dev/<plugin>`

Override with `DEVKIT_HOME`. Open a **new terminal** after install so PATH updates apply.

## Documentation

| Doc | Purpose |
|-----|---------|
| [docs/getting-started.md](docs/getting-started.md) | Run and install tools |
| [docs/plugins.md](docs/plugins.md) | Write a plugin |
| [docs/architecture.md](docs/architecture.md) | How DevKit works |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [ROADMAP.md](ROADMAP.md) | Future plans |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |

## Development

```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

Git Flow branches: `main`, `develop`, `feature/*`, `release/*`, `hotfix/*`.

## License

MIT — see [LICENSE.md](LICENSE.md). Author: Axcel Blade.
