# Roadmap

High-level plans for DevKit after **0.8.5**.

## 0.9.0 — Helpers & polish

- Shared download checksum verification
- Optional MySQL/PostgreSQL data-dir init helper after binary install
- Android `sdkmanager` license helper
- `devkit doctor flutter` deep checks (wraps `flutter doctor` when installed)
- Mirror / offline cache documentation
- Optional Redis / Yarn plugins
- Docker Compose helper that wires `DOCKER_HOST` for remote engines

## 1.0.0 — Stable application

- Frozen plugin API
- Packaged installers for Windows / macOS / Linux (optional)
- Documented upgrade path between DevKit versions
- Security review of env mutation paths

## Completed

- **0.9.1** — Launchers always open the menu with no arguments: fixed the
  rustup-init file name, rebuild on every run, internet only needed to install Rust
- **0.8.5** — Single shared PATH; per-tool `*_HOME` variables no longer set
- **0.8.4** — `msys2` plugin (portable base runtime, Windows); `qemu` plugin
  (silent NSIS install on Windows, system wrappers on macOS/Linux)
- **0.8.3** — `pmd` plugin; Unix writability probe so `list` does not fail on `/opt`
- **0.8.2** — `junit` plugin (console standalone + wrappers); doctor reports Rustc/Cargo
- **0.8.1** — Launchers check internet first, then install rustup into the
  machine `dev` folder (`CARGO_HOME` / `RUSTUP_HOME`) before building DevKit
- **0.8.0** — Interactive `devkit menu`; `devkit.bat`/`devkit.sh` auto-bootstrap
  Rust via rustup (with an internet check first); internet check before plugin
  downloads; Docker packaging removed
- **0.7.0** — Full rewrite from Python to Rust (same CLI, same plugin behavior)
- **0.6.1** — Mono MSI extract reliability on Windows CI
- **0.6.0** — Docker image/Compose + `docker` CLI plugin
- **0.5.0** — python/go/rust/dotnet/cmake/ninja/maven/postgresql/sqlite/kubectl/terraform/pnpm/deno/bun/platform-tools
- **0.4.0** — `gradle`, `php`, `mysql`, `node`, `android`; install flags; progress UI; Mono MSI smoke
- **0.3.0** — Git plugin (MinGit / system wrappers)
- **0.2.0** — CLI app, plugin framework, flutter/composer/jdk/mono/hello

## Principles

- Remain a **CLI application**, not a library-first product
- Prefer official vendor download channels
- Keep installs under the machine `dev` folder when portable archives exist
- Stay cross-platform: Windows, macOS, Linux

Dates are goals, not commitments. See [TODO.md](TODO.md) for granular tasks.
