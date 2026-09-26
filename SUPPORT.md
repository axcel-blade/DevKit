# Support

## How to get help

1. Read [README.md](README.md) and [docs/getting-started.md](docs/getting-started.md).
2. Run diagnostics:

   ```bash
   cargo run --release -- doctor
   cargo run --release -- status <plugin>
   ```

3. Search [existing issues](https://github.com/axcel-blade/DevKit/issues).
4. Open a new issue with the bug or question template.
5. For discussion (ideas, Q&A), use [GitHub Discussions](https://github.com/axcel-blade/DevKit/discussions) if enabled.

## What to include in a bug report

- DevKit version (`cargo run --release -- --version`) — current is **0.9.2**
- OS and architecture
- Command you ran
- Full error output
- Whether `DEVKIT_HOME` is set

## Maintainer

- Author: **Axcel Blade**
- Repository: https://github.com/axcel-blade/DevKit

## Out of scope

- Support for unofficial plugins not shipped in this repository
- Troubleshooting every third-party SDK beyond DevKit's install/env steps
  (e.g. Flutter doctor device issues after a successful install)
