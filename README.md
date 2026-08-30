# DevKit

**Version:** 0.8.0

DevKit is a **CLI application** that sets up developer environments on Windows, macOS, and Linux: download an SDK, install it under a machine `dev` folder, and configure PATH / environment variables through plugins.

It is **not** a published crate. Run it from this repository.

## Requirements

- Windows, macOS, or Linux
- A Rust toolchain (`cargo`) — `devkit.bat` / `devkit.sh` install one for you
  automatically via [rustup](https://rustup.rs) if `cargo` isn't already on
  PATH (checking for internet access first); install manually beforehand if
  you'd rather control that yourself.

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

| Plugin | Installs | Environment |
|--------|----------|-------------|
| `python` | Portable CPython (python-build-standalone) | `PATH`, `PYTHON_HOME` |
| `go` | Latest stable Go toolchain | `PATH`, `GOROOT` |
| `rust` | Rust stable via rustup | `PATH`, `CARGO_HOME`, `RUSTUP_HOME` |
| `dotnet` | .NET SDK 8.0 LTS | `PATH`, `DOTNET_ROOT` |
| `node` | Latest Node.js LTS (npm / npx) | `PATH`, `NODE_HOME` |
| `pnpm` | Latest pnpm standalone | `PATH`, `PNPM_HOME` |
| `deno` | Latest Deno runtime | `PATH`, `DENO_INSTALL` |
| `bun` | Latest Bun runtime | `PATH`, `BUN_INSTALL` |
| `jdk` | Eclipse Temurin JDK (default 21; `--version`) | `PATH`, `JAVA_HOME` |
| `maven` | Latest Apache Maven | `PATH`, `MAVEN_HOME` |
| `gradle` | Latest Gradle binary ZIP | `PATH`, `GRADLE_HOME` |
| `cmake` | Latest CMake binary | `PATH`, `CMAKE_HOME` |
| `ninja` | Latest Ninja binary | `PATH`, `NINJA_HOME` |
| `git` | MinGit (Windows); system wrappers (Unix) | `PATH`, `GIT_HOME` |
| `php` | PHP NTS (Windows); system wrappers (Unix) | `PATH`, `PHP_HOME` |
| `composer` | Composer PHAR + wrapper | `PATH`, `COMPOSER_HOME` |
| `mysql` | MySQL 8.4 LTS portable | `PATH`, `MYSQL_HOME` |
| `postgresql` | EDB Windows binaries; system wrappers (Unix) | `PATH`, `POSTGRESQL_HOME` |
| `sqlite` | Official sqlite-tools CLI | `PATH`, `SQLITE_HOME` |
| `android` | Android SDK cmdline-tools | `PATH`, `ANDROID_HOME`, `ANDROID_SDK_ROOT` |
| `platform-tools` | Android platform-tools (`adb`) | `PATH`, `ANDROID_PLATFORM_TOOLS` |
| `flutter` | Flutter SDK (`--channel` / `--version`) | `PATH`, `FLUTTER_ROOT` |
| `kubectl` | Latest stable kubectl | `PATH`, `KUBECTL_HOME` |
| `docker` | Official static Docker CLI (client only) | `PATH`, `DOCKER_HOME` |
| `terraform` | Latest Terraform | `PATH`, `TERRAFORM_HOME` |
| `mono` | Mono 6.12 (Win/macOS); system wrappers (Linux) | `PATH`, `MONO_HOME` |
| `hello` | Demo ZIP install | `PATH`, `DEVKIT_HELLO_ROOT` |

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
