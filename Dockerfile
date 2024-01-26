FROM rust:1-slim-bookworm as builder

WORKDIR /build

RUN apt-get update && apt-get --no-install-recommends -y install \
    pkg-config=1.8.1-1 \
    libssl-dev=3.0.11-1~deb12u2

COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo fetch --locked

COPY src src/
RUN --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --release

FROM debian:bookworm-slim AS app-base

WORKDIR /app

RUN apt-get update && apt-get --no-install-recommends -y install \
    libssl3=3.0.11-1~deb12u2 \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    abc

FROM app-base as frontend

COPY --from=builder /build/target/release/frontend ./

COPY assets assets/
COPY css css/
COPY templates templates/

USER abc:abc
CMD ["./frontend"]

FROM app-base as backend

COPY --from=builder /build/target/release/backend ./

USER abc:abc
CMD ["./backend"]
