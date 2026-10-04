# Contributing to DevKit

Thanks for contributing. DevKit uses **Git Flow**.

## Branches

| Branch | Purpose |
|--------|---------|
| `main` | Stable releases only |
| `develop` | Integration branch for the next release |
| `feature/*` | New features (branch from `develop`) |
| `release/*` | Release prep (branch from `develop`) |
| `hotfix/*` | Urgent fixes (branch from `main`) |

### Feature workflow

```bash
git checkout develop
git pull
git checkout -b feature/my-change
# ... commit ...
git push -u origin feature/my-change
# open PR into develop
```

### Release workflow

```bash
git checkout develop
git checkout -b release/0.x.0
# bump VERSION, CHANGELOG, docs
git checkout main
git merge --no-ff release/0.x.0
git checkout develop
git merge --no-ff release/0.x.0
```

## Makefile

If `make` is not installed, run `scripts\ensure-makefile.bat` (Windows) or `sh scripts/ensure-makefile.sh` (macOS / Linux) first. Before a target runs, `make` checks for GNU make, `cargo`, and a C compiler, and installs them when they are missing. `make sh` also checks for `bash`.

`make ci` runs the same checks as GitHub Actions. `make bat` and `make sh` with no extra words open the plugin menu. Extra words are passed to the launcher (`make bat doctor`, `make sh list`). `make` alone prints:

```text
DevKit
  make build          Debug build
  make release        Release build
  make test           Run tests
  make test-release   Run tests in release mode
  make fmt            Format Rust sources
  make fmt-check      Check formatting
  make clippy         Lint with warnings denied
  make doctor         Build and run devkit doctor
  make version        Print the DevKit version
  make run            Open the plugin menu via cargo
  make bat            Run devkit.bat and show the menu
  make sh             Run devkit.sh and show the menu
  make bat doctor     Run devkit.bat with arguments
  make sh list        Run devkit.sh with arguments
  make ci             fmt-check, clippy, release build, release tests, doctor
  make clean          Remove build artifacts
```

Or call Cargo directly:

```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

Run the app without a separate build step:

```bash
cargo run --release -- doctor
```

## Adding a plugin

1. Create `src/plugins/<name>.rs` with a struct implementing the `Plugin` trait and a unique `id()`.
2. Implement `status`, `install`, `uninstall`, and `env_spec`. Override `supported_os` when the plugin does not run on every desktop OS.
3. Register the plugin in `crate::plugins::all()` (`src/plugins/mod.rs`).
4. Add tests in a `#[cfg(test)] mod tests` block at the bottom of the file.
5. Document the plugin in `README.md` and `docs/plugins.md`.

See [docs/plugins.md](docs/plugins.md).

## Commit messages

- Use clear, imperative subjects (e.g. `Add Node.js plugin`).
- Do **not** add AI or bot co-author trailers (`Co-authored-by: Cursor`, etc.).
- Keep commits focused; update docs when behavior changes.
- Bump version in `VERSION` and `Cargo.toml`, and all user-facing markdown
  for releases. Prefer the plain `VERSION` file (not `VERSION.md`).

## Pull requests

- Target `develop` for features; `main` only for hotfixes/releases.
- Fill out the PR template.
- Ensure CI (`cargo test` + `cargo run --release -- doctor`) passes.

## Code of conduct

By participating, you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
