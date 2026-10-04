# Installation

**DevKit 0.13.0**

Clone the repository, then from the root:

```bash
make bat
make sh
make bat doctor
make sh list
```

If `make` is not installed, run `scripts\ensure-makefile.bat` (Windows) or `sh scripts/ensure-makefile.sh` (macOS / Linux) first. Every `make` target checks for GNU make, `cargo`, and a C compiler, and installs them when they are missing. `make sh` also checks for `bash`. `make bat` and `make sh` with no extra words open the plugin menu. Extra words are passed to the launcher. `make` alone prints:

```text
DevKit
  make build          Debug build
  make release        Release build
  make test           Run tests
  make test-release   Run tests in release mode
  make fmt            Format Rust sources
  make fmt-check      Check formatting
  make clippy         Lint with warnings denied
  make doctor         Build and run devkit doctor
  make version        Print the DevKit version
  make run            Open the plugin menu via cargo
  make bat            Run devkit.bat and show the menu
  make sh             Run devkit.sh and show the menu
  make bat doctor     Run devkit.bat with arguments
  make sh list        Run devkit.sh with arguments
  make ci             fmt-check, clippy, release build, release tests, doctor
  make clean          Remove build artifacts
```

The same commands without make:

```bash
cargo run --release -- doctor
cargo run --release -- list
```

No separate install step is required — `cargo run` builds the binary on first
use. Prefer `devkit.bat` / `devkit.sh`: if `cargo` is not already in the
machine `dev` folder (`C:\dev\rust`, `/opt/dev/rust`, or `~/dev/rust`) they
check the internet connection and install rustup stable there, then rebuild
DevKit (instant when up to date) and run it. On Windows, if a previous DevKit
window still has `devkit.exe` open, the launcher renames that copy aside and
builds a new one. On Linux / WSL, `devkit.sh` also installs the system C
linker (`cc`, e.g. `build-essential`) when it is missing, since Rust needs it
to link.

`make bat` and `make sh` do the same thing. Running either launcher with no arguments (or double-clicking `devkit.bat`) opens an interactive menu. On Windows the terminal prints:

```text
Checking for available versions ...

DevKit
OS: Windows
Version: 0.13.0
#    ID               NAME                     STATUS         INSTALLED        AVAILABLE
--------------------------------------------------------------------------------------------------------
1    anaconda         Anaconda                 not_installed  -                2026.07-1
5    chocolatey       Chocolatey               installed      2.7.4            2.7.4 (up to date)
32   rust             Rust                     installed      1.98.1           1.99.0 (update available)

Enter a number to install/uninstall, 'r' to refresh versions, or 'q' to quit:
```

On macOS and Linux the OS line is `OS: macOS` or `OS: Linux`. Plugins that cannot be installed on this OS are left out (`chocolatey` and `msys2` appear only on Windows). The AVAILABLE column shows the newest version, `up to date`, or `update available`.

Optional flags for selected plugins:

```bash
cargo run --release -- install flutter --channel beta
cargo run --release -- install jdk --version 17
```

See [docs/getting-started.md](../docs/getting-started.md) for full details.
