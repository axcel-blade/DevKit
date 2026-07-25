# DevKit

**Version:** 0.3.0

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
devkit.bat install git
devkit.bat install jdk
devkit.bat install flutter
```

**macOS / Linux**

```bash
chmod +x devkit.sh
./devkit.sh doctor
./devkit.sh list
./devkit.sh install jdk
```

**Any OS**

```bash
python main.py doctor
python main.py install composer
python main.py install mono
```

## Built-in plugins

| Plugin | Installs | Environment |
|--------|----------|-------------|
| `git` | MinGit (Windows); system Git wrappers (macOS/Linux) | `PATH`, `GIT_HOME` |
| `flutter` | Latest stable Flutter SDK | `PATH`, `FLUTTER_ROOT` |
| `composer` | Composer (`composer.phar` + wrapper) | `PATH`, `COMPOSER_HOME` (needs PHP) |
| `jdk` | Eclipse Temurin JDK 21 (LTS) | `PATH`, `JAVA_HOME` |
| `mono` | Mono 6.12 (Win/macOS); Linux wraps system Mono | `PATH`, `MONO_HOME` |
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
python -m pip install -e ".[dev]"
pytest
ruff check src tests
```

Git Flow branches: `main`, `develop`, `feature/*`, `release/*`, `hotfix/*`.

## License

MIT — see [LICENSE.md](LICENSE.md). Author: Axcel Blade.
