FROM docker.io/library/rust:1.85-slim-bookworm

RUN apt-get update && apt-get install -y --no-install-recommends \
    ed python3 ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Official slim image ships rustc/cargo only; clippy is required by ./check.sh.
RUN rustup component add clippy

WORKDIR /work
