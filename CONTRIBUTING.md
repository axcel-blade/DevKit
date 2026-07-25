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
python -m pip install -e ".[dev]"
pytest
ruff check src tests
```

Run the app without packaging:

```bash
python main.py doctor
```

## Adding a plugin

1. Create `src/devkit/plugins/<name>.py` with a `Plugin` subclass and unique `id`.
2. Implement `status`, `install`, `uninstall`, and `env_spec`.
3. Add tests under `tests/`.
4. Document the plugin in `README.md` and `docs/plugins.md`.

See [docs/plugins.md](docs/plugins.md).

## Commit messages

- Use clear, imperative subjects (e.g. `Add Node.js plugin`).
- Do **not** add AI or bot co-author trailers.
- Keep commits focused; update docs when behavior changes.

## Pull requests

- Target `develop` for features; `main` only for hotfixes/releases.
- Fill out the PR template.
- Ensure CI (pytest + `python main.py doctor`) passes.

## Code of conduct

By participating, you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
