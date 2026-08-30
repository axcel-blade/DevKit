# Plugins

DevKit **0.7.0** ships these built-in plugins:

| ID | Notes |
|----|-------|
| `python` | Portable CPython via python-build-standalone |
| `go` | Latest stable from go.dev/dl JSON |
| `rust` | rustup stable into `CARGO_HOME` / `RUSTUP_HOME` |
| `dotnet` | .NET SDK 8.0 LTS ZIP from Microsoft release metadata |
| `node` | Latest Node.js LTS (npm / npx) |
| `pnpm` | Latest pnpm standalone ZIP/tar from GitHub |
| `deno` | Latest Deno ZIP from GitHub |
| `bun` | Latest Bun ZIP from GitHub |
| `jdk` | Temurin via Adoptium; `--version` selects feature (default 21) |
| `maven` | Latest Apache Maven 3.x binary ZIP |
| `gradle` | Latest Gradle `-bin.zip` |
| `cmake` | Latest Kitware CMake binary |
| `ninja` | Latest ninja-build binary ZIP |
| `git` | MinGit on Windows; system wrappers on Unix |
| `php` | Windows NTS ZIP; system wrappers on Unix |
| `composer` | `composer.phar` + wrapper; needs PHP |
| `mysql` | MySQL 8.4 LTS portable (binaries only) |
| `postgresql` | EDB Windows binaries; system client wrappers on Unix |
| `sqlite` | Official sqlite-tools from sqlite.org |
| `android` | Android SDK cmdline-tools |
| `platform-tools` | Android `adb` / fastboot ZIP |
| `flutter` | `--channel` (stable/beta/dev) and `--version` |
| `kubectl` | Latest stable from dl.k8s.io |
| `docker` | Official static Docker CLI (client only; no engine) |
| `terraform` | Latest from HashiCorp releases |
| `mono` | Windows MSI / macOS PKG; Linux system wrappers |
| `hello` | Offline demo of ZIP → `dev` folder → env |

## Authoring a plugin

Create `src/plugins/<name>.rs`:

```rust
use crate::download::install_archive_from_urls;
use crate::plugin::{EnvSpec, InstallContext, InstallResult, InstallState, Plugin, PluginStatus};
use anyhow::Result;

pub struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn id(&self) -> &'static str { "example" }
    fn name(&self) -> &'static str { "Example" }
    fn description(&self) -> &'static str { "Sample SDK installer" }

    fn status(&self, ctx: &InstallContext) -> PluginStatus {
        let marker = ctx.install_dir.join("bin").join("tool");
        if marker.is_file() {
            PluginStatus::new(InstallState::Installed, Some(ctx.install_dir.clone()))
        } else {
            PluginStatus::new(InstallState::NotInstalled, Some(ctx.install_dir.clone()))
        }
    }

    fn install(&self, ctx: &InstallContext) -> Result<InstallResult> {
        install_archive_from_urls(
            &[
                ("windows", "https://example.com/tool-win.zip"),
                ("macos", "https://example.com/tool-mac.zip"),
                ("linux", "https://example.com/tool-linux.tar.xz"),
            ],
            &ctx.install_dir,
            true,
            None,
        )?;
        Ok(InstallResult::new(ctx.install_dir.clone(), "ok"))
    }

    fn uninstall(&self, ctx: &InstallContext) -> Result<()> {
        if ctx.install_dir.exists() {
            std::fs::remove_dir_all(&ctx.install_dir)?;
        }
        Ok(())
    }

    fn env_spec(&self, ctx: &InstallContext) -> EnvSpec {
        EnvSpec {
            paths: vec![ctx.install_dir.join("bin")],
            vars: vec![("EXAMPLE_HOME".to_string(), ctx.install_dir.display().to_string())],
        }
    }
}
```

Register it in `crate::plugins::all()` (`src/plugins/mod.rs`) — Rust has no
runtime module scan, so every plugin is listed explicitly there.

## Helpers

- `crate::download::install_archive_from_url` / `install_archive_from_urls`
- `crate::progress::download_progress` (shared progress bar for large downloads)
- `crate::plugin_utils::github_latest_release` / `pick_release_asset` / `binary_status` / `find_files_named`
- `crate::installers::extract_msi_admin` / `extract_pkg` (Windows/macOS installers)
- `crate::platform::pick_for_os`, `cpu_arch`, `adoptium_os`
