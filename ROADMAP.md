# Roadmap

High-level plans for DevKit after **0.3.0**.

## 0.4.0 — More runtimes

- PHP installer plugin (pairs with Composer)
- Node.js LTS plugin
- Shared download progress and checksum verification

## 0.5.0 — Flutter ecosystem

- Android cmdline-tools optional companion plugin
- `devkit doctor flutter` deep checks (wraps `flutter doctor` when installed)
- Mirror / offline cache documentation

## 1.0.0 — Stable application

- Frozen plugin API
- Packaged installers for Windows / macOS / Linux (optional)
- Documented upgrade path between DevKit versions
- Security review of env mutation paths

## Completed

- **0.3.0** — Git plugin (MinGit / system wrappers)
- **0.2.0** — CLI app, plugin framework, flutter/composer/jdk/mono/hello

## Principles

- Remain a **CLI application**, not a library-first product
- Prefer official vendor download channels
- Keep installs under the machine `dev` folder when portable archives exist
- Stay cross-platform: Windows, macOS, Linux

Dates are goals, not commitments. See [TODO.md](TODO.md) for granular tasks.
