# Installation

**DevKit 0.9.3**

Clone the repository, then from the root:

```bash
cargo run --release -- doctor
cargo run --release -- list
```

No separate install step is required — `cargo run` builds the binary on first
use. Prefer `devkit.bat` / `devkit.sh`: if `cargo` is not already in the
machine `dev` folder (`C:\dev\rust`, `/opt/dev/rust`, or `~/dev/rust`) they
check the internet connection and install rustup stable there, then rebuild
DevKit (instant when up to date) and run it.

Running either launcher with no arguments (or double-clicking `devkit.bat`) opens an interactive menu to pick
plugins to install/uninstall instead of needing to know their ids up front. The menu
shows each plugin's installed version next to the newest available one, so pending
updates are visible at a glance.

Optional flags for selected plugins:

```bash
cargo run --release -- install flutter --channel beta
cargo run --release -- install jdk --version 17
```

See [docs/getting-started.md](../docs/getting-started.md) for full details.
