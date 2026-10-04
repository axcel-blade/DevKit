# Plugins

Built-in plugins in **0.13.0**. The menu (`make bat` / `make sh`) lists only the rows available on the current OS. `devkit install` refuses the others and prints the reason.

| OS | Plugins |
|----|---------|
| Windows, macOS, Linux | `anaconda`, `android`, `android-ndk`, `bun`, `cmake`, `composer`, `deno`, `docker`, `dotnet`, `flutter`, `git`, `go`, `gradle`, `jdk`, `junit`, `kubectl`, `make`, `maven`, `mono`, `mysql`, `ninja`, `node`, `php`, `platform-tools`, `pmd`, `pnpm`, `postgresql`, `python`, `qemu`, `rust`, `sqlite`, `terraform`, `uv` |
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

Authoring guide: [docs/plugins.md](../docs/plugins.md).
