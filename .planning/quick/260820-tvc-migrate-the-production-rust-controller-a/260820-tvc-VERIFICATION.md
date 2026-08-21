---
quick_id: 260820-tvc
verified: 2026-08-21T01:50:42Z
status: passed
score: 5/5 must-haves verified
---

# Quick Task 260820-tvc Verification

**Goal:** Migrate the production Rust controller and established smoke images from `oracle 0.6.3`/Instant Client to `oracledb 26.0.0-beta.2` using the validated ADB wallet configuration without changing persistence behavior.

## Must-Haves

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Production catalog, schema, heartbeat, and established live smokes use `oracledb 26.0.0-beta.2`. | VERIFIED | `controller/Cargo.toml`, `controller/src/oracle_connection.rs`, `controller/src/oracle_catalog.rs`, `controller/src/oracle_schema.rs`, `controller/src/oracle_heartbeat.rs`, and both established live smoke tests compile under `live-persistence`. `cargo tree` exposes `oracledb` and no `oracle` dependency. |
| 2 | Existing production feature contracts remain available. | VERIFIED | `production-persistence` now enables `dep:oracledb`; `live-persistence` still enables `production-persistence`. Production check and live-smoke no-run compilation passed. |
| 3 | ADB mTLS explicitly uses the mounted wallet directory and required password without logging secrets. | VERIFIED | The shared helper applies config directory, wallet location, and wallet password. Ansible supplies both values, rejects an empty production password with `no_log: true`, and operator docs explain `ewallet.pem` decryption. |
| 4 | Controller and smoke images contain no Oracle Instant Client packages. | VERIFIED | All three Dockerfiles install only their required CA/curl packages. All three images built, and RPM inspection returned no `oracle-instantclient` packages. |
| 5 | Existing catalog/privacy behavior remains covered. | VERIFIED | Full controller suite passed 137 tests with only two credential-gated live tests ignored; production-feature clippy/check, Dockerfile contract tests, Ansible syntax, and smoke executable startup all passed. The preceding live spike proved ADB plus OCI create/read/verify/cleanup in 2.07 seconds. |

## Human Rollout Check

The codebase goal is achieved. After deployment, the operator should run the established live persistence and static publish smokes to validate the new semver image in the production VM environment. A failure there should block rollout and trigger rollback, but no local implementation gap remains.

## Result

No gaps found. The production controller image source is ready for PR/deploy with the OL10 slim runtime retained as a deliberate operational choice.
