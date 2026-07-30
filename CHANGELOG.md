# Changelog

All notable changes to DevKit are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0] - 2026-07-30

### Added

- Plugins: `python`, `go`, `rust`, `dotnet`, `cmake`, `ninja`, `maven`
- Plugins: `postgresql`, `sqlite`, `kubectl`, `terraform`, `pnpm`, `deno`, `bun`
- Plugin: `platform-tools` (Android `adb` / fastboot companion)
- Shared `devkit.plugin_utils` helpers for GitHub releases and binary status

### Changed

- Documentation, wiki, SECURITY, SUPPORT, ROADMAP, and TODO updated for 0.5.0
- Restored plain `VERSION` file as the canonical version source (not `VERSION.md`)

## [0.4.0] - 2026-07-25

### Added

- `gradle` plugin: latest Gradle binary ZIP from services.gradle.org (`GRADLE_HOME` + PATH)
- `php` plugin: latest Windows NTS x64 ZIP from downloads.php.net; system wrappers on Unix
- `mysql` plugin: MySQL Community Server 8.4.10 LTS portable archives (`MYSQL_HOME` + PATH)
- `node` plugin: latest Node.js LTS from nodejs.org (`NODE_HOME` + PATH; includes npm/npx)
- `android` plugin: Android SDK cmdline-tools (`ANDROID_HOME` / `ANDROID_SDK_ROOT`; Flutter helper)
- `install --version` / `--channel` for Flutter; `install --version` for JDK feature lines
- Shared download progress bar (`devkit.progress`) for large SDK downloads
- Windows CI smoke job that admin-extracts the Mono MSI and verifies `mono.exe` layout
- Nested Mono MSI layout unit tests (Program Files / Mono / bin variants)

### Changed

- Documentation, wiki, SECURITY, SUPPORT, ROADMAP, and TODO updated for 0.4.0
- CI runs unit tests with `-m "not smoke"`; dedicated Windows job for Mono MSI smoke

### Removed

- Unused `assets/`, `scripts/dev.py`, and duplicate root `requirements*.txt` leftovers

## [0.3.0] - 2026-07-25

### Added

- `git` plugin: MinGit (Git for Windows) portable ZIP on Windows
- System Git wrapper registration on macOS/Linux for the `git` plugin
- `GIT_HOME` environment variable when Git is installed via DevKit

### Changed

- Documentation and wiki updated for version 0.3.0 and the Git plugin
- Roadmap / TODO adjusted (Git plugin completed)

## [0.2.0] - 2026-07-25

### Added

- CLI application entrypoints: `main.py`, `devkit.bat`, `devkit.sh`
- Plugin framework (`Plugin`, registry, `EnvManager`)
- Cross-platform install root (`C:\dev`, `/opt/dev`, or `~/dev`)
- Archive helpers: ZIP, tar.gz, tar.xz; MSI/PKG extractors
- Plugins: `flutter`, `composer`, `jdk` (Temurin 21), `mono`, `hello`
- Windows user env (registry) and macOS/Linux `~/.devkit/env.sh` + shell profile hook
- CI on Ubuntu, macOS, and Windows
- Project docs: CONTRIBUTING, SECURITY, SUPPORT, ROADMAP, TODO, Code of Conduct
- GitHub issue, PR, and discussion templates

### Changed

- Project repositioned as an application (not a PyPI library)
- Version bumped from 0.1.0 template to 0.2.0

### Removed

- Template `myapp` package leftovers

## [0.1.0] - 2026-07-25

### Added

- Initial project scaffold from auto-gen-py-project

[0.5.0]: https://github.com/axcel-blade/DevKit/releases/tag/v0.5.0
[0.4.0]: https://github.com/axcel-blade/DevKit/releases/tag/v0.4.0
[0.3.0]: https://github.com/axcel-blade/DevKit/releases/tag/v0.3.0
[0.2.0]: https://github.com/axcel-blade/DevKit/releases/tag/v0.2.0
[0.1.0]: https://github.com/axcel-blade/DevKit/releases/tag/v0.1.0
