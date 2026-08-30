# DevKit CLI image — run the installer app without a local Rust toolchain.
# Build:  docker build -t devkit .
# Run:    docker run --rm -v devkit-data:/devkit devkit doctor
#         docker run --rm -v devkit-data:/devkit -e DEVKIT_HOME=/devkit devkit list

FROM rust:1-slim-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim

LABEL org.opencontainers.image.title="DevKit" \
      org.opencontainers.image.description="CLI to install developer SDKs into a machine dev folder" \
      org.opencontainers.image.source="https://github.com/axcel-blade/DevKit"

# Installs and caches live under DEVKIT_HOME (mount a volume in production use).
ENV DEVKIT_HOME=/devkit

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/devkit /usr/local/bin/devkit
RUN mkdir -p /devkit

VOLUME ["/devkit"]

ENTRYPOINT ["devkit"]
CMD ["doctor"]
