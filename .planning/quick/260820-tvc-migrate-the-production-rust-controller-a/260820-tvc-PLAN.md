---
quick_id: 260820-tvc
status: complete
mode: validate
description: Migrate the production Rust controller and smoke images from oracle 0.6.3/Instant Client to oracledb 26.0.0-beta.2 using the validated ADB wallet configuration, preserving catalog behavior and tests
must_haves:
  truths:
    - Production catalog, schema bootstrap, heartbeat, persistence smoke, and static publish smoke use oracledb 26.0.0-beta.2.
    - The existing production-persistence and live-persistence feature contracts remain available.
    - ADB mTLS connections explicitly use the mounted wallet directory and required wallet password without logging secrets.
    - Controller and smoke runtime images contain no Oracle Instant Client packages.
    - Existing catalog and privacy behavior remains covered by the full Rust test suite and production-feature checks.
  artifacts:
    - controller/src/oracle_connection.rs
    - controller/src/oracle_catalog.rs
    - controller/src/oracle_schema.rs
    - controller/src/oracle_heartbeat.rs
    - controller/Dockerfile
    - controller/Dockerfile.smoke
    - controller/Dockerfile.static-smoke
  key_links:
    - controller routes initialize schema and repositories through the shared oracledb connection helper.
    - Ansible runtime env supplies ORACLE_DB_WALLET_DIR and ORACLE_DB_WALLET_PASSWORD to the controller.
    - Existing smoke Dockerfiles compile the migrated live-persistence tests and run without native Oracle libraries.
---

# Quick Task 260820-tvc Plan

## Task 1: Port production persistence to oracledb

**Files:** `controller/Cargo.toml`, `controller/Cargo.lock`, `controller/src/lib.rs`, `controller/src/oracle_connection.rs`, `controller/src/oracle_catalog.rs`, `controller/src/oracle_schema.rs`, `controller/src/oracle_heartbeat.rs`, `controller/tests/live_persistence_smoke.rs`, `controller/tests/live_static_publish_smoke.rs`

**Action:** Replace `oracle 0.6.3` with `oracledb 26.0.0-beta.2`, add a shared ADB configuration/connection boundary, port query/bind/row traits and heartbeat behavior, and keep existing public feature names intact. Collapse the temporary spike-only feature/test once the established live smokes use the production driver.

**Verify:** `cargo check --features production-persistence`; compile both ignored live smoke tests; run the ordinary Rust test suite.

**Done:** No production controller or established live smoke source imports the old `oracle` crate, and all compile/test checks pass.

## Task 2: Remove Instant Client and update runtime contracts

**Files:** `controller/Dockerfile`, `controller/Dockerfile.smoke`, `controller/Dockerfile.static-smoke`, `controller/Dockerfile.oracledb-smoke`, `controller/tests/caddy_static_routes.rs`, `.env.example`, `docs/configuration-contract.md`, `docs/deployment-runbook.md`, `docs/static-runtime-runbook.md`

**Action:** Remove native Oracle packages from controller/smoke images, delete the now-redundant spike-only image/test, make the wallet password requirement explicit, and update image contract tests and operator documentation.

**Verify:** Build the production controller image and established smoke images; inspect their installed RPMs for absence of Instant Client; run targeted Dockerfile contract tests.

**Done:** Production artifacts use the validated pure-Rust driver configuration and no runtime image installs Instant Client.

## Task 3: Full regression verification and closeout

**Files:** `.planning/quick/260820-tvc-migrate-the-production-rust-controller-a/260820-tvc-SUMMARY.md`, `.planning/STATE.md`

**Action:** Run formatting, tests, production checks, clippy, and container verification; document results and the remaining operator-run post-deploy live smoke requirement.

**Verify:** All local checks pass and the diff contains no unrelated security-scan artifacts.

**Done:** Production migration is committed atomically with a concise summary and state entry.
