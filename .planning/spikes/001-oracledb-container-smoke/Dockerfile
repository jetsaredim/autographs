FROM rust:1.98-bookworm AS builder

WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/workspace/target,sharing=locked \
    cargo build --release \
    && install -Dm755 /workspace/target/release/autographs-oracledb-spike /tmp/autographs-oracledb-spike

FROM oraclelinux:10-slim

RUN microdnf install -y ca-certificates \
    && microdnf clean all

COPY --from=builder /tmp/autographs-oracledb-spike /usr/local/bin/autographs-oracledb-spike

ENV TNS_ADMIN=/opt/autographs/wallet

ENTRYPOINT ["/usr/local/bin/autographs-oracledb-spike"]
