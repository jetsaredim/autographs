---
quick_id: 260820-tvc
status: complete
completed: 2026-08-21T01:50:42Z
implementation_commit: 56bf5a5
---

# Quick Task 260820-tvc Summary

Migrated the production Rust controller from `oracle 0.6.3` to the Oracle-maintained pure-Rust `oracledb 26.0.0-beta.2` crate after the VM spike proved ADB wallet connectivity, catalog writes, OCI Object Storage upload/read/delete, and cleanup in 2.07 seconds.

## Implementation

- Added one shared Oracle connection boundary that configures wallet directory, wallet location, and wallet password for ADB mTLS without exposing credentials.
- Ported the production catalog repository, schema preflight/bootstrap, heartbeat, live persistence smoke, and live static publish smoke to `oracledb` query, bind, row, and affected-row APIs.
- Preserved the existing `production-persistence` and `live-persistence` feature names while removing the old `oracle`/ODPI-C dependency graph.
- Removed Oracle Instant Client from the controller and both established smoke images, then deleted the temporary spike-only feature, test, and Dockerfile.
- Made the wallet password requirement explicit in examples/runbooks and added an Ansible deploy preflight that rejects an empty production wallet password before container startup.
- Kept `oraclelinux:10-slim` as the runtime base for this migration. It remains a maintained, compact match for the OL10 VM and avoids combining a distribution change with the driver rollout; future Debian/distroless/non-root evaluation can be measured independently.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo test --manifest-path controller/Cargo.toml` (137 passed; 2 credential-gated live tests ignored)
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke --test live_static_publish_smoke --no-run`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --features production-persistence -- -D warnings`
- Ansible deploy playbook syntax check with writable temporary directories
- Built `controller/Dockerfile`, `controller/Dockerfile.smoke`, and `controller/Dockerfile.static-smoke`
- Inspected all three runtime RPM inventories; none contains an Oracle Instant Client package
- Started both smoke images without activation flags; both executables loaded and completed their intentional skip paths successfully

## Remaining Live Gate

After the new semver controller image is deployed, rerun the established live persistence smoke and live static publish smoke from `docs/static-runtime-runbook.md`. This confirms the migrated production repository and deployed controller behavior against the live ADB/OCI environment; it is an operator rollout gate, not a local implementation gap.
