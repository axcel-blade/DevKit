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

`main` and `develop` are protected: changes go through pull requests that
need one approving review and passing CI (the three `test` jobs and
`Mono MSI layout (Windows)`). Pull requests into `main` can only be
**squash-merged**.

1. **Prepare the release on a branch from `develop`.**

   ```bash
   git checkout develop
   git pull
   git checkout -b release/vX.Y.Z
   ```

   Bump the version in `Cargo.toml`, `Cargo.lock` (the `devkit` package
   entry), `README.md`, `docs/*.md`, and `wiki/*.md`. Move the
   `## [Unreleased]` notes in `CHANGELOG.md` under a new
   `## [X.Y.Z] - YYYY-MM-DD` heading and add its link at the bottom.
   Check that `cargo build --release` prints the new version with
   `devkit --version`.

2. **Merge the release branch into `develop`.** Open a PR from
   `release/vX.Y.Z` into `develop` and merge it with a normal merge commit.

3. **Squash-merge `develop` into `main`.** Open a PR from `develop` into
   `main` titled `Release vX.Y.Z` and use **Squash and merge**.

4. **Publish the GitHub release.** Tag the new `main` commit and publish
   the release with its notes from `CHANGELOG.md`:

   ```bash
   git checkout main
   git pull
   gh release create vX.Y.Z --target main --title "vX.Y.Z" --notes-file <notes.md>
   ```

   Publishing triggers the **Release binaries** workflow
   (`.github/workflows/release.yml`). It builds `devkit` for Windows x64,
   Linux x64, and macOS Intel and Apple Silicon, then attaches each archive
   and its `.sha256` file to the release. To rebuild binaries for an
   existing tag, run the workflow manually from the Actions tab with that
   tag.

5. **Reset `develop` to `main`.** The squash creates a new commit on
   `main` that `develop` does not have. Point `develop` at `main` so the
   branches match and the next release PR starts clean:

   ```bash
   git fetch origin
   git push --force-with-lease origin origin/main:refs/heads/develop
   ```

   This needs admin rights, because `develop` is protected. Merge every open
   PR into `develop` before the reset; otherwise rebase them onto the new
   `develop` afterwards.

6. **Clean up.** Delete the `release/vX.Y.Z` branch locally and on GitHub.

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
5. Document the plugin in `README.md` and `docs/PLUGINS.md`.

See [docs/PLUGINS.md](docs/PLUGINS.md).

## Commit messages

- Use clear, imperative subjects (e.g. `Add Node.js plugin`).
- Do **not** add AI or bot co-author trailers (`Co-authored-by: Cursor`, etc.).
- Keep commits focused; update docs when behavior changes.
- For releases, bump the version in `Cargo.toml` and all user-facing
  markdown (see [Release workflow](#release-workflow)).

## Pull requests

- Target `develop` for features; `main` only for hotfixes/releases.
  PRs into `main` are squash-merged.
- Fill out the PR template.
- Ensure CI (`cargo test` + `cargo run --release -- doctor`) passes.

## Code of conduct

By participating, you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
