# Installation

**DevKit 0.7.0**

Clone the repository, then from the root:

```bash
cargo run --release -- doctor
cargo run --release -- list
```

No separate install step is required — `cargo run` builds the binary on first
use. A Rust toolchain (`cargo`) must be on PATH; get one from [rustup.rs](https://rustup.rs).

Optional flags for selected plugins:

```bash
cargo run --release -- install flutter --channel beta
cargo run --release -- install jdk --version 17
```

See [docs/getting-started.md](../docs/getting-started.md) for full details.

Docker image:

```bash
docker compose run --rm devkit doctor
```

See [docs/docker.md](../docs/docker.md).
