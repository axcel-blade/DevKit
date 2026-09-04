## Summary

<!-- What does this PR change and why? -->

## Type

- [ ] Feature (`feature/*` → `develop`)
- [ ] Release (`release/*` → `main` + `develop`)
- [ ] Hotfix (`hotfix/*` → `main` + `develop`)
- [ ] Docs / chore

## Checklist

- [ ] Version updated if this is a release (`VERSION`, `Cargo.toml`, CHANGELOG)
- [ ] Docs updated when behavior changes
- [ ] `cargo test` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo fmt --all -- --check` passes
- [ ] No AI/bot co-author trailers on commits

## Test plan

- [ ] `cargo run --release -- doctor`
- [ ] `cargo run --release -- list`
- [ ] Relevant `install` / `status` smoke for touched plugins
