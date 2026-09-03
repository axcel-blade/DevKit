# Installation

**DevKit 0.8.1**

Clone the repository, then from the root:

```bash
cargo run --release -- doctor
cargo run --release -- list
```

No separate install step is required — `cargo run` builds the binary on first
use. Prefer `devkit.bat` / `devkit.sh`: they require an internet connection,
then install rustup stable into the machine `dev` folder (`C:\dev\rust`,
`/opt/dev/rust`, or `~/dev/rust`) if `cargo` is not already there.

Running either launcher with no arguments opens an interactive menu to pick
plugins to install/uninstall instead of needing to know their ids up front.

Optional flags for selected plugins:

```bash
cargo run --release -- install flutter --channel beta
cargo run --release -- install jdk --version 17
```

See [docs/getting-started.md](../docs/getting-started.md) for full details.
