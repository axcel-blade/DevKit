# DevKit

**Version:** 0.6.0

DevKit is a **CLI application** that sets up developer environments on Windows, macOS, and Linux: download an SDK, install it under a machine `dev` folder, and configure PATH / environment variables through plugins.

It is **not** a PyPI library. Run it from this repository.

## Requirements

- Python 3.12+
- Windows, macOS, or Linux

## Quick start

**Windows**

```bat
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
./devkit.sh doctor
./devkit.sh list
./devkit.sh install python
```

**Any OS**

```bash
python main.py doctor
python main.py install cmake
python main.py install kubectl
python main.py install docker
```

### Docker

```bash
docker build -t devkit .
docker run --rm -e DEVKIT_HOME=/devkit -v devkit-data:/devkit devkit doctor
docker compose run --rm devkit list
```

See [docs/docker.md](docs/docker.md).

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
| [docs/docker.md](docs/docker.md) | Run DevKit in Docker / install Docker CLI |
| [docs/plugins.md](docs/plugins.md) | Write a plugin |
| [docs/architecture.md](docs/architecture.md) | How DevKit works |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [ROADMAP.md](ROADMAP.md) | Future plans |
| [CONTRIBUTING.md](CONTRIBUTING.md) | How to contribute |

## Development

```bash
python -m pip install -e ".[dev]"
pytest
ruff check src tests
```

Git Flow branches: `main`, `develop`, `feature/*`, `release/*`, `hotfix/*`.

## License

MIT — see [LICENSE.md](LICENSE.md). Author: Axcel Blade.
