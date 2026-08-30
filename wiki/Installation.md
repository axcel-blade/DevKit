# Installation

**DevKit 0.7.0**

Clone the repository, then from the root:

```bash
cargo run --release -- doctor
cargo run --release -- list
```

No separate install step is required — `cargo run` builds the binary on first
use. A Rust toolchain (`cargo`) must be on PATH for `cargo run`; or just use
`devkit.bat` / `devkit.sh`, which install one via rustup automatically (after
checking for internet access) if `cargo` isn't found.

Running either launcher with no arguments opens an interactive menu to pick
plugins to install/uninstall instead of needing to know their ids up front.

Optional flags for selected plugins:

```bash
cargo run --release -- install flutter --channel beta
cargo run --release -- install jdk --version 17
```

See [docs/getting-started.md](../docs/getting-started.md) for full details.
