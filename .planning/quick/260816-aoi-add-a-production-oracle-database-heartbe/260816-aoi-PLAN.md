---
quick_id: 260816-aoi
status: planned
created: 2026-08-16
---

# Quick Task 260816-aoi: Oracle ADB Heartbeat

Add a production Oracle heartbeat so the controller periodically performs a lightweight SQL command and prevents an Always Free Autonomous Database from being stopped for inactivity.

## Tasks

1. Add a production-persistence-only heartbeat module that parses `AUTOGRAPHS_ORACLE_HEARTBEAT_INTERVAL_SECONDS`, defaults to a daily interval, supports `0` to disable, and runs `select 1 from dual` on a blocking thread.
2. Start the heartbeat when the Oracle catalog repository is configured, after schema bootstrap succeeds and before the router is returned.
3. Document the new runtime variable in `.env.example`, the Ansible app environment template, and the configuration contract.

## Verification

- `cargo fmt --check --manifest-path controller/Cargo.toml`
- `cargo test --manifest-path controller/Cargo.toml oracle_heartbeat --features production-persistence`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
