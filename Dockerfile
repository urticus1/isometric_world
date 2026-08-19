# syntax=docker/dockerfile:1

FROM rust:1.85-slim AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config \
        libx11-dev \
        libxkbcommon-dev \
        libwayland-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY resources ./resources

RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
        libx11-6 \
        libxkbcommon0 \
        libwayland-client0 \
        libgl1 \
        libxext6 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/fortress /app/fortress
COPY --from=builder /app/resources /app/resources

CMD ["/app/fortress"]
