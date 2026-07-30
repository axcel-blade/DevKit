# Plugins

DevKit **0.5.0** ships these built-in plugins:

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
| `terraform` | Latest from HashiCorp releases |
| `mono` | Windows MSI / macOS PKG; Linux system wrappers |
| `hello` | Offline demo of ZIP → `dev` folder → env |

## Authoring a plugin

Create `src/devkit/plugins/<name>.py`:

```python
from devkit.download import install_archive_from_urls
from devkit.plugin import (
    EnvSpec,
    InstallContext,
    InstallResult,
    InstallState,
    Plugin,
    PluginStatus,
)


class ExamplePlugin(Plugin):
    id = "example"
    name = "Example"
    description = "Sample SDK installer"

    def status(self, ctx: InstallContext) -> PluginStatus:
        marker = ctx.install_dir / "bin" / "tool"
        if marker.exists():
            return PluginStatus(InstallState.INSTALLED, ctx.install_dir)
        return PluginStatus(InstallState.NOT_INSTALLED, ctx.install_dir)

    def install(self, ctx: InstallContext) -> InstallResult:
        install_archive_from_urls(
            {
                "windows": "https://example.com/tool-win.zip",
                "macos": "https://example.com/tool-mac.zip",
                "linux": "https://example.com/tool-linux.tar.xz",
            },
            ctx.install_dir,
        )
        return InstallResult(ctx.install_dir, message="ok")

    def uninstall(self, ctx: InstallContext) -> None:
        import shutil
        if ctx.install_dir.exists():
            shutil.rmtree(ctx.install_dir)

    def env_spec(self, ctx: InstallContext) -> EnvSpec:
        return EnvSpec(
            paths=[ctx.install_dir / "bin"],
            vars={"EXAMPLE_HOME": str(ctx.install_dir.resolve())},
        )
```

The registry auto-discovers `Plugin` subclasses under `devkit.plugins`.

## Helpers

- `devkit.download.install_archive_from_url` / `install_archive_from_urls`
- `devkit.progress.download_progress` (shared progress bar for large downloads)
- `devkit.plugin_utils.github_latest_release` / `pick_release_asset` / `binary_status`
- `devkit.installers.extract_msi_admin` / `extract_pkg` (Windows/macOS installers)
- `devkit.platform.pick_for_os`, `cpu_arch`, `adoptium_os`
