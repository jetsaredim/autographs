---
quick_id: 260816-n0o
status: complete
completed: 2026-08-16T20:36:54Z
implementation_commit: a0764ab
---

# Quick Task 260816-n0o Summary

Updated the production Oracle catalog heartbeat so a controller start, deploy, or reboot runs the first `select 1 from dual` immediately, then continues on the configured heartbeat interval. This gives operators a prompt `Oracle catalog heartbeat succeeded` or safe failure log instead of waiting one full day for confirmation.

Documented the startup heartbeat behavior in the runtime configuration contract and `.env.example`.

Review/coder follow-up on PR #206 replaced the trivial first-tick helper assertion with a paused Tokio timer test that exercises the actual heartbeat ticker, verifies the first tick is immediate, and verifies the next tick waits for the configured interval.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml`
- `cargo test --manifest-path controller/Cargo.toml oracle_heartbeat --features production-persistence`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo fmt --check --manifest-path controller/Cargo.toml`
- `cargo check --manifest-path controller/Cargo.toml`
- `cargo clippy --manifest-path controller/Cargo.toml --features production-persistence -- -D warnings`
