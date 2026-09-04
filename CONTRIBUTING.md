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

## Development setup

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
2. Implement `status`, `install`, `uninstall`, and `env_spec`.
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
