# Plugins

DevKit **0.3.0** ships these built-in plugins:

| ID | Notes |
|----|-------|
| `git` | MinGit ZIP on Windows; system Git wrappers on macOS/Linux |
| `flutter` | Latest stable from Flutter release JSON |
| `composer` | `composer.phar` + wrapper; needs PHP on PATH |
| `jdk` | Eclipse Temurin JDK 21 via Adoptium API |
| `mono` | Windows MSI / macOS PKG; Linux wraps system Mono |
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
- `devkit.installers.extract_msi_admin` / `extract_pkg` (Windows/macOS installers)
- `devkit.platform.pick_for_os`, `cpu_arch`, `adoptium_os`
