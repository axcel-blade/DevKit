# Roadmap

High-level plans for DevKit after **0.7.0**.

## 0.8.0 — Helpers & polish

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
