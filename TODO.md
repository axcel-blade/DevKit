# TODO

Track near-term tasks for DevKit **0.9.0 / 1.0.0**.

## In progress / next

_(none — see Backlog)_

## Backlog

- [ ] Dry-run mode (`install --dry-run`)
- [ ] Export/import of installed plugin manifest
- [ ] Signed release binaries (`cargo build --release` artifacts per platform)
- [ ] Optional MySQL/PostgreSQL data-dir init / Android `sdkmanager` license helpers
- [ ] Shared download checksum verification
- [ ] Redis / Yarn plugins (optional)
- [ ] Docker Compose helper that wires `DOCKER_HOST` for remote engines

## Docs

- [ ] Expand wiki pages as the plugin set grows
- [ ] Add short demo GIF/screenshot to README

## Done (0.8.3)

- [x] `pmd` plugin (PMD Source Code Analyzer from GitHub releases)
- [x] Fix Unix `/opt/dev` writability probe so `devkit list` works on CI

## Done (0.8.2)

- [x] `junit` plugin (JUnit Platform Console Standalone from Maven Central)
- [x] `devkit doctor` reports Rustc/Cargo instead of Python

## Done (0.8.1)

- [x] Launchers require internet first; error if offline
- [x] Install / reuse Rust in the machine `dev` folder (not `~/.cargo`)
      before building and running the Rust binary

## Done (0.8.0)

- [x] `devkit menu` — interactive install/uninstall picker
- [x] `devkit.bat` / `devkit.sh` auto-bootstrap a Rust toolchain via rustup
      (with an internet check first) when `cargo` isn't already on PATH
- [x] Internet connectivity check before any plugin download
- [x] Docker packaging removed (`Dockerfile`, `docker-compose.yml`,
      `docs/docker.md`); `.pre-commit-config.yaml` removed

## Done (0.7.0)

- [x] Full rewrite from Python to Rust — same CLI, same plugin behavior
- [x] All 27 plugins ported to the `Plugin` trait
- [x] `cargo test` unit + integration suite replacing `pytest`
- [x] CI matrix on `cargo fmt` / `clippy` / `test` (Windows/macOS/Linux)

## Done (0.6.1)

- [x] Harden Mono MSI `msiexec` extract for Windows CI
- [x] Short smoke work dir under `RUNNER_TEMP` / `TEMP`
- [x] Unit tests for MSI path helpers

## Done (0.6.0)

- [x] Docker image + Compose for running DevKit
- [x] `docker` plugin (static CLI client)
- [x] docs/docker.md

## Done (0.5.0)

- [x] `python`, `go`, `rust`, `dotnet`, `cmake`, `ninja`, `maven`
- [x] `postgresql`, `sqlite`, `kubectl`, `terraform`
- [x] `pnpm`, `deno`, `bun`, `platform-tools`
- [x] Shared `devkit.plugin_utils` helpers

## Done (0.4.0)

- [x] `gradle` plugin (official binary ZIP, all platforms)
- [x] `php` plugin (Windows ZIP; Unix system wrappers)
- [x] `mysql` plugin (Community Server 8.4 LTS portable)
- [x] `node` plugin (Node.js LTS from nodejs.org)
- [x] `android` plugin (SDK cmdline-tools for Flutter)
- [x] Optional `--version` / `--channel` for Flutter and JDK
- [x] Shared download progress UI for large downloads
- [x] Mono MSI layout unit tests + Windows CI smoke extract

## Done (0.3.0)

- [x] `git` plugin (MinGit on Windows; system wrappers on Unix)

## Done (0.2.0)

- [x] CLI app entrypoints
- [x] Plugin framework + env manager
- [x] flutter, composer, jdk, mono, hello plugins
- [x] Cross-platform CI
- [x] GitHub community markdown set
