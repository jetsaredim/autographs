---
quick_id: 260816-aoi
status: complete
completed: 2026-08-16T11:45:55Z
implementation_commit: 627be09
---

# Quick Task 260816-aoi Summary

Added a production Oracle catalog heartbeat that starts after schema bootstrap succeeds and periodically runs `select 1 from dual` using the configured Oracle credentials. The heartbeat defaults to once per day, logs success or failure without credentials, and can be disabled by setting `AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS=0`.

Updated the deployment environment template, `.env.example`, and configuration contract so operators can see and tune the heartbeat interval.

Review/coder follow-up on PR #205 wired the documented heartbeat interval GitHub variable through the deploy workflow, removed Oracle connection-string logging from the heartbeat startup message, switched heartbeat failure logs to safe error categories, and extended logging/deploy contract coverage.

## Verification

- `cargo fmt --check --manifest-path controller/Cargo.toml`
- `cargo test --manifest-path controller/Cargo.toml oracle_heartbeat --features production-persistence`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo check --manifest-path controller/Cargo.toml`
- `cargo clippy --manifest-path controller/Cargo.toml --features production-persistence -- -D warnings`
- `cargo test --manifest-path controller/Cargo.toml deploy_wires_oracle_heartbeat_interval_override`
- `cargo test --manifest-path controller/Cargo.toml controller_route_tracing_does_not_log_private_or_secret_terms`
- `git diff --check origin/main...HEAD -- .github/workflows/deploy.yml controller/src/oracle_heartbeat.rs controller/tests/logging_contract.rs controller/tests/caddy_static_routes.rs`
