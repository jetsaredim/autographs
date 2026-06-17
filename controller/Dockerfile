# syntax=docker/dockerfile:1

FROM rust:1.96-bookworm AS builder

WORKDIR /workspace

COPY controller/Cargo.toml controller/Cargo.lock ./
COPY controller/src ./src
COPY controller/db ./db
COPY controller/static-public ./static-public

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/workspace/target,sharing=locked \
    cargo build --release --features production-persistence \
    && install -Dm755 /workspace/target/release/autographs-controller /tmp/autographs-controller

FROM oraclelinux:10-slim

RUN microdnf install -y oracle-instantclient-release-26ai-el10-1.0-6.el10 \
    && microdnf install -y ca-certificates-2025.2.80_v9.0.305-102.el10_1 oracle-instantclient-basic-23.26.2.0.0-2.el10 \
    && microdnf clean all

COPY --from=builder /tmp/autographs-controller /usr/local/bin/autographs-controller
COPY controller/static-admin /opt/autographs/static-admin

ENV TNS_ADMIN=/opt/autographs/wallet

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/autographs-controller"]
