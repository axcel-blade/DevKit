# Docker

Run DevKit inside a container, or install the Docker **CLI** plugin on the host.

## Run DevKit with Docker

Build and check health:

```bash
docker build -t devkit .
docker run --rm -e DEVKIT_HOME=/devkit -v devkit-data:/devkit devkit doctor
docker run --rm -e DEVKIT_HOME=/devkit -v devkit-data:/devkit devkit list
```

Install a plugin into the persistent volume:

```bash
docker run --rm -e DEVKIT_HOME=/devkit -v devkit-data:/devkit devkit install hello
```

### Compose

```bash
docker compose run --rm devkit doctor
docker compose run --rm devkit list
docker compose run --rm devkit install hello
```

`DEVKIT_HOME` is `/devkit` inside the container. The named volume `devkit-data`
keeps downloaded SDKs across runs.

### Notes

- The image is a multi-stage build: `rust:1-slim` compiles the `devkit`
  binary, then it's copied into a minimal `debian:bookworm-slim` runtime.
- Host env mutation (Windows registry / shell profiles) from plugins applies
  **inside** the container, not on your host OS. Prefer mounting `/devkit` and
  using tools from that volume when working in Docker.
- Large SDK downloads need network access from the container.

## Install Docker CLI via DevKit (host)

```bash
cargo run --release -- install docker
```

This installs the official **static Docker client** only. You still need Docker
Engine or Docker Desktop (or `DOCKER_HOST`) to run containers.
