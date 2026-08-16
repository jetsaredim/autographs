---
quick_id: 260816-aoi
status: complete
completed: 2026-08-16T11:45:55Z
implementation_commit: 627be09
---

# Quick Task 260816-aoi Summary

Added a production Oracle catalog heartbeat that starts after schema bootstrap succeeds and periodically runs `select 1 from dual` using the configured Oracle credentials. The heartbeat defaults to once per day, logs success or failure without credentials, and can be disabled by setting `AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS=0`.

Updated the deployment environment template, `.env.example`, and configuration contract so operators can see and tune the heartbeat interval.

## Verification

- `cargo fmt --check --manifest-path controller/Cargo.toml`
- `cargo test --manifest-path controller/Cargo.toml oracle_heartbeat --features production-persistence`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo check --manifest-path controller/Cargo.toml`
- `cargo clippy --manifest-path controller/Cargo.toml --features production-persistence -- -D warnings`
