# TODO

Track near-term tasks for DevKit **0.5.0 / 1.0.0**.

## In progress / next

_(none — see Backlog)_

## Backlog

- [ ] Dry-run mode (`install --dry-run`)
- [ ] Export/import of installed plugin manifest
- [ ] Signed release binaries (PyInstaller / platform packages)
- [ ] Optional MySQL data-dir init / license accept helpers for Android `sdkmanager`
- [ ] Shared download checksum verification

## Docs

- [ ] Expand wiki pages as the plugin set grows
- [ ] Add short demo GIF/screenshot to README

## Done (0.4.0)

- [x] `gradle` plugin (official binary ZIP, all platforms)
- [x] `php` plugin (Windows ZIP; Unix system wrappers)
- [x] `mysql` plugin (Community Server 8.4 LTS portable)
- [x] `node` plugin (Node.js LTS from nodejs.org)
- [x] `android` plugin (SDK cmdline-tools for Flutter)
- [x] Optional `--version` / `--channel` for Flutter and JDK
- [x] Shared download progress UI for large downloads
- [x] Mono MSI layout unit tests + Windows CI smoke extract

## Done (0.3.0)

- [x] `git` plugin (MinGit on Windows; system wrappers on Unix)

## Done (0.2.0)

- [x] CLI app entrypoints
- [x] Plugin framework + env manager
- [x] flutter, composer, jdk, mono, hello plugins
- [x] Cross-platform CI
- [x] GitHub community markdown set
