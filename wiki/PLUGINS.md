# Plugins

Built-in plugins in **1.0.1**. The menu (`make bat` / `make sh`) lists only the rows available on the current OS. `devkit install` refuses the others and prints the reason.

| OS | Plugins |
|----|---------|
| Windows, macOS, Linux | `anaconda`, `android`, `android-ndk`, `bun`, `cmake`, `composer`, `deno`, `docker`, `dotnet`, `flutter`, `git`, `go`, `gradle`, `jdk`, `junit`, `kubectl`, `llvm`, `make`, `maven`, `mono`, `mysql`, `nasm`, `ninja`, `node`, `npm`, `php`, `platform-tools`, `pmd`, `pnpm`, `postgresql`, `python`, `qemu`, `rust`, `sqlite`, `terraform`, `uv`, `vscode`, `yarn` |
| Windows | `chocolatey`, `msys2` |

Highlights:

```bash
cargo run --release -- install python
cargo run --release -- install go
cargo run --release -- install rust
cargo run --release -- install cmake
cargo run --release -- install make
cargo run --release -- install chocolatey
cargo run --release -- install platform-tools
cargo run --release -- install jdk
cargo run --release -- install maven
cargo run --release -- install gradle
cargo run --release -- install junit
cargo run --release -- install pmd
cargo run --release -- install kubectl
```

Authoring guide: [docs/PLUGINS.md](../docs/PLUGINS.md).

## Prerequisites

Override `prerequisites()` to return `(plugin_id, binary)` pairs. Before
`devkit install` or the menu installs a plugin, DevKit checks each pair. It
skips the pair when that plugin is installed or `binary` is on PATH. Otherwise
it installs that plugin first, prerequisites of prerequisites included.

| Plugin | Needs |
|--------|-------|
| `android`, `gradle`, `junit`, `maven`, `pmd` | `jdk` (`java`) |
| `composer` | `php` (`php`) |
| `flutter` | `git` (`git`) |

Updates (`u` / `a`) do not install prerequisites again.

