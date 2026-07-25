# Roadmap

High-level plans for DevKit after **0.4.0**.

## 0.5.0 — Flutter ecosystem & helpers

- Shared download checksum verification
- Optional MySQL data-dir init helper after binary install
- Android `sdkmanager` license / platform-tools helper
- `devkit doctor flutter` deep checks (wraps `flutter doctor` when installed)
- Mirror / offline cache documentation

## 1.0.0 — Stable application

- Frozen plugin API
- Packaged installers for Windows / macOS / Linux (optional)
- Documented upgrade path between DevKit versions
- Security review of env mutation paths

## Completed

- **0.4.0** — `gradle`, `php`, `mysql`, `node`, `android`; install flags; progress UI; Mono MSI smoke
- **0.3.0** — Git plugin (MinGit / system wrappers)
- **0.2.0** — CLI app, plugin framework, flutter/composer/jdk/mono/hello

## Principles

- Remain a **CLI application**, not a library-first product
- Prefer official vendor download channels
- Keep installs under the machine `dev` folder when portable archives exist
- Stay cross-platform: Windows, macOS, Linux

Dates are goals, not commitments. See [TODO.md](TODO.md) for granular tasks.
