## Summary

<!-- What does this PR change and why? -->

## Type

- [ ] Feature (`feature/*` → `develop`)
- [ ] Release (`release/*` → `main` + `develop`)
- [ ] Hotfix (`hotfix/*` → `main` + `develop`)
- [ ] Docs / chore

## Checklist

- [ ] Version updated if this is a release (`VERSION`, `pyproject.toml`, `__version__`, tests, CHANGELOG)
- [ ] Docs updated when behavior changes
- [ ] `pytest` passes
- [ ] `ruff check src tests` passes
- [ ] No AI/bot co-author trailers on commits

## Test plan

- [ ] `python main.py doctor`
- [ ] `python main.py list`
- [ ] Relevant `install` / `status` smoke for touched plugins
