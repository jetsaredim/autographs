---
quick_id: 260831-bj8
status: complete
implementation_commits:
  - ce80006
  - 1d6653d
  - f8322af
  - f48f2e2
pull_request: 229
completed: 2026-08-31
---

# Quick Task 260831-bj8 Summary: Recover Oracle Connections After Vault Rotation

Added bounded, failure-driven database credential recovery for the long-running controller and incorporated PR #228's Rust crate updates without enabling automatic OCI rotation.

## Changes

- Retained the Oracle database password Vault OCID as a refresh coordinate after startup resolution while continuing to move the password itself into one shared provider.
- Added generation-tagged credential snapshots, a serialized refresh lock, and a generation re-check so concurrent failures using the same stale credential share one `CURRENT`-stage Vault read.
- Classified only the Oracle driver's `DbError` code `ORA-01017` as a refresh trigger.
- Retried a failed connection once after a successful or concurrent refresh; unrelated errors, session initialization, queries, and application operations remain non-retrying.
- Routed catalog and heartbeat connections through the common asynchronous recovery helper while leaving direct-password use and synchronous schema bootstrap unchanged.
- Documented that wallet-password and admin-hash rotation remain restart-only and scheduled OCI rotation remains disabled pending live proof.
- Updated Argon2 from 0.5.3 to 0.6.0 and UUID from 1.24.1 to 1.26.0 in the controller and retained Oracle smoke crate. Migrated password hashing to Argon2 0.6's built-in random-salt API and retained PHC verification behavior.
- Added safe OCI Vault bundle-version observability: every successful startup secret load records only its logical kind, numeric bundle version, and requested `CURRENT` stage. Database refresh/reuse logs correlate the replacement controller generation with the durable Vault version without logging secret values, OCIDs, user-defined version names, or customer metadata.

## Validation

- `cargo fmt --manifest-path controller/Cargo.toml --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml` — passed.
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence` — passed, including 64 unit tests and all non-live integration suites.
- Focused concurrency coverage proves concurrent successful refreshes share one source read and generation, while concurrent failed refreshes share one error/read and a later independent attempt can retry Vault.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --features production-persistence --all-targets -- -D warnings` — passed.
- `cargo check --manifest-path .planning/spikes/001-oracledb-container-smoke/Cargo.toml` — passed with UUID 1.26.0.
- `bash scripts/validate-runtime.sh` — passed.
- `git diff --check` — passed.
- PR #229 post-fix CI run `33398993944` — all seven jobs passed, including controller coverage/Clippy, runtime image build, Terraform plan, Ansible validation, and secret scan.
- Deep review found CR-01 in the failed-refresh concurrency path; commit `f8322af` fixed it and replied directly to the inline thread. Re-review at that head was clean with no actionable findings remaining.
- PR #229 Vault-version observability CI run `33411730625` — all seven jobs passed.
- Deep review of observability commit `f48f2e2` found no actionable findings and posted the clean confirmation directly on PR #229.

## Live-Proof Boundary

The implementation and CI prove the recovery algorithm and production build path. They do not prove a live rotation. After merge and deploy, first confirm ordinary database access and an incremental publish, then perform a controlled coordinated ADB/Vault rotation and verify the controller logs one successful credential refresh without a restart. Automatic rotation remains disabled until that evidence exists.
