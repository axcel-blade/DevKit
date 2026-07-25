# TODO

Track near-term tasks for DevKit **0.3.x / 0.4.0**.

## In progress / next

- [ ] Optional `--version` / channel flags for Flutter and JDK plugins
- [ ] Progress UI polish for large downloads
- [ ] PHP plugin (so Composer has a matching runtime installer)
- [ ] Verify Mono MSI layout across Windows editions in CI smoke tests

## Backlog

- [ ] Node.js / npm plugin
- [ ] Android SDK / cmdline-tools helper for Flutter
- [ ] Dry-run mode (`install --dry-run`)
- [ ] Export/import of installed plugin manifest
- [ ] Signed release binaries (PyInstaller / platform packages)

## Docs

- [ ] Expand wiki pages as the plugin set grows
- [ ] Add short demo GIF/screenshot to README

## Done (0.3.0)

- [x] `git` plugin (MinGit on Windows; system wrappers on Unix)

## Done (0.2.0)

- [x] CLI app entrypoints
- [x] Plugin framework + env manager
- [x] flutter, composer, jdk, mono, hello plugins
- [x] Cross-platform CI
- [x] GitHub community markdown set
