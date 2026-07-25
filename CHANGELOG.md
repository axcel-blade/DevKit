# Changelog

All notable changes to DevKit are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

[0.2.0]: https://github.com/axcel-blade/DevKit/releases/tag/v0.2.0
[0.1.0]: https://github.com/axcel-blade/DevKit/releases/tag/v0.1.0
