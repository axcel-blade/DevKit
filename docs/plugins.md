# Plugins

DevKit **0.13.0** ships these built-in plugins. The OS column is where the menu lists the plugin and where `devkit install` accepts it.

| ID | OS | Notes |
|----|----|-------|
| `python` | Windows, macOS, Linux | Portable CPython 3.14.7 via python-build-standalone |
| `go` | Windows, macOS, Linux | Latest stable from go.dev/dl JSON |
| `rust` | Windows, macOS, Linux | rustup stable into `CARGO_HOME` / `RUSTUP_HOME` |
| `dotnet` | Windows, macOS, Linux | .NET SDK 10.0 LTS ZIP from Microsoft release metadata |
| `node` | Windows, macOS, Linux | Latest Node.js LTS (npm / npx) |
| `pnpm` | Windows, macOS, Linux | Latest pnpm standalone ZIP/tar from GitHub |
| `deno` | Windows, macOS, Linux | Latest Deno ZIP from GitHub |
| `bun` | Windows, macOS, Linux | Latest Bun ZIP from GitHub |
| `uv` | Windows, macOS, Linux | Latest uv and uvx archive from GitHub |
| `anaconda` | Windows, macOS, Linux | Latest Anaconda3 installer from repo.anaconda.com, run silently |
| `jdk` | Windows, macOS, Linux | Temurin via Adoptium; `--version` selects feature (default 25) |
| `maven` | Windows, macOS, Linux | Latest Apache Maven 3.x binary ZIP |
| `gradle` | Windows, macOS, Linux | Latest Gradle `-bin.zip` |
| `junit` | Windows, macOS, Linux | JUnit Platform Console Standalone JAR + wrapper; `--version` optional |
| `pmd` | Windows, macOS, Linux | PMD Source Code Analyzer binary ZIP from GitHub; `--version` optional |
| `cmake` | Windows, macOS, Linux | Latest Kitware CMake binary |
| `ninja` | Windows, macOS, Linux | Latest ninja-build binary ZIP |
| `make` | Windows, macOS, Linux | Chocolatey `make` 4.4.1 package unpacked into the DevKit folder on Windows (no elevated `choco`); system wrappers on Unix |
| `chocolatey` | Windows | Latest Chocolatey CLI unpacked into the DevKit folder; sets user `ChocolateyInstall` |
| `git` | Windows, macOS, Linux | MinGit on Windows; system wrappers on Unix |
| `php` | Windows, macOS, Linux | Windows NTS ZIP; system wrappers on Unix |
| `composer` | Windows, macOS, Linux | `composer.phar` + wrapper; needs PHP |
| `mysql` | Windows, macOS, Linux | MySQL 8.4 LTS portable (8.4.11, binaries only) |
| `postgresql` | Windows, macOS, Linux | PostgreSQL 18.4 EDB Windows binaries; system client wrappers on Unix |
| `sqlite` | Windows, macOS, Linux | Official sqlite-tools from sqlite.org |
| `android` | Windows, macOS, Linux | Android SDK cmdline-tools (build 16111833) |
| `platform-tools` | Windows, macOS, Linux | Android `adb` / fastboot ZIP |
| `android-ndk` | Windows, macOS, Linux | Android NDK r29 native toolchain ZIP |
| `flutter` | Windows, macOS, Linux | `--channel` (stable/beta/dev) and `--version` |
| `kubectl` | Windows, macOS, Linux | Latest stable from dl.k8s.io |
| `docker` | Windows, macOS, Linux | Official static Docker CLI (client only; no engine) |
| `terraform` | Windows, macOS, Linux | Latest from HashiCorp releases |
| `mono` | Windows, macOS, Linux | Windows MSI / macOS PKG; Linux system wrappers |
| `msys2` | Windows | Portable MSYS2 base runtime |
| `qemu` | Windows, macOS, Linux | QEMU 11.1.0 silent NSIS install (Windows); system QEMU wrappers on macOS/Linux |

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

### Versions in the menu

The interactive menu (`make bat`, `make sh`, or a launcher with no arguments) prints like this on Windows. Only plugins whose `supported_os` includes the host are listed. The **INSTALLED** and **AVAILABLE** columns are fed by two optional trait methods:

```text
Checking for available versions ...

DevKit
OS: Windows
Version: 0.13.0
#    ID               NAME                     STATUS         INSTALLED        AVAILABLE
--------------------------------------------------------------------------------------------------------
1    anaconda         Anaconda                 not_installed  -                2026.07-1
5    chocolatey       Chocolatey               installed      2.7.4            2.7.4 (up to date)
12   git              Git                      not_installed  -                2.56.0.windows.1
32   rust             Rust                     installed      1.98.1           1.99.0 (update available)

Enter a number to install/uninstall, 'r' to refresh versions, or 'q' to quit:
```

On macOS and Linux the OS line is `OS: macOS` or `OS: Linux`. Chocolatey and MSYS2 are omitted there. A matching install prints `up to date`. A newer release prints `update available`.

- `installed_version(&self, ctx)` — defaults to the first line of the
  `.devkit-<id>` marker in `install_dir` (placeholders such as `stable` or
  `system-wrapper` show as `-`). Override when the SDK records its own version
  (e.g. JDK `release`, Flutter `flutter.version.json`, `rustc --version`).
- `latest_version(&self, ctx)` — defaults to `Ok(None)`. Return the version
  your `install` would fetch right now, ideally by reusing the same resolver so
  the string matches the marker you write. Pinned plugins return their pinned
  constant. Lookups run in parallel when the menu opens (and on `r`).

```rust
fn latest_version(&self, _ctx: &InstallContext) -> Result<Option<String>> {
    Ok(Some(resolve_example_download()?.1))
}
```

### OS support

`Plugin::supported_os` defaults to Windows, macOS, and Linux. Override it when
a plugin cannot be installed on every host — Chocolatey and MSYS2 return
Windows only. The menu lists only plugins that support the host OS, so those
two are hidden on macOS and Linux. `devkit install` refuses the rest and
prints why:

```text
Cannot install chocolatey on macOS.
Reason: Chocolatey is only supported on Windows.
```

```rust
fn supported_os(&self) -> &'static [HostOS] {
    &[HostOS::Windows]
}
```

## Helpers

- `crate::download::install_archive_from_url` / `install_archive_from_urls`
- `crate::download::ensure_internet_access` / `has_internet_access` (call before
  any custom network request that doesn't already go through `download`/
  `plugin_utils` — see below)
- `crate::progress::download_progress` (shared progress bar for large downloads)
- `crate::plugin_utils::github_latest_release` / `pick_release_asset` / `binary_status` / `find_files_named`
- `crate::installers::extract_msi_admin` / `extract_pkg` (Windows/macOS installers)
- `crate::platform::pick_for_os`, `cpu_arch`, `adoptium_os`, `DESKTOP_HOSTS`
