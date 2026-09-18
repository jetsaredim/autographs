# syntax=docker/dockerfile:1

FROM rust:1.98-bookworm AS chef

WORKDIR /workspace

RUN cargo install cargo-chef --version 0.1.78 --locked

FROM chef AS planner

COPY controller .

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /workspace/recipe.json ./recipe.json

RUN cargo chef cook --release --features production-persistence --recipe-path recipe.json \
    && rm -rf src

COPY controller/Cargo.toml controller/Cargo.lock ./
COPY controller/src ./src
COPY controller/db ./db
COPY controller/static-public/404.html ./static-public/404.html
COPY controller/static-public/favicon.ico ./static-public/favicon.ico
COPY controller/static-public/icon.png ./static-public/icon.png
COPY controller/static-public/index.html ./static-public/index.html
COPY controller/static-public/architecture ./static-public/architecture
COPY controller/static-public/assets ./static-public/assets
COPY controller/static-public/collection ./static-public/collection
COPY controller/static-public/data/not-found-quotes.json ./static-public/data/not-found-quotes.json
COPY controller/static-public/templates ./static-public/templates
COPY controller/static-admin ./static-admin

RUN cargo build --release --features production-persistence \
    && install -Dm755 /workspace/target/release/autographs-controller /tmp/autographs-controller

FROM oraclelinux:10-slim

RUN microdnf install -y ca-certificates-2025.2.80_v9.0.305-102.el10_1 \
    && microdnf clean all

COPY --from=builder /tmp/autographs-controller /usr/local/bin/autographs-controller
COPY controller/static-admin /opt/autographs/static-admin

ENV TNS_ADMIN=/opt/autographs/wallet

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/autographs-controller"]
