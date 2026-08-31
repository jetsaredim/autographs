---
quick_id: 260830-vp0
status: complete
implementation_commits:
  - 5372520
completed: 2026-08-31
---

# Quick Task 260830-vp0 Summary: Introduce a Static Database Credential Provider

Moved the Oracle database password behind a crate-private shared snapshot provider without changing startup-only Vault resolution, Oracle connection attempts, error handling, retry behavior, or rotation behavior.

## Changes

- Added a concrete `DatabaseCredentialProvider` that owns one shared credential allocation and returns cheap `Arc` snapshots.
- Kept the provider immutable so this slice introduces no background work, synchronization, refresh, or mutation path.
- Changed `OracleConnectionSettings` to retain the injected provider instead of directly owning a password `String`.
- Made the production composition root construct and inject the provider while preserving required-value validation order and the existing shared settings allocation.
- Made every Oracle connection attempt retain a credential snapshot for the duration of connection construction.
- Kept the provider API crate-private and added focused tests for provider allocation reuse and settings/provider identity.

## Validation

- `cargo fmt --all -- --check` — passed.
- `cargo test` — passed.
- `cargo test --features production-persistence` — passed, including 49 production-feature unit tests and both new provider tests.
- `cargo check --features production-persistence` — passed.
- `cargo clippy --all-targets --features production-persistence -- -D warnings` — passed.
- `bash scripts/validate-runtime.sh` — passed.
- `git diff --check` — passed.

## Scope Boundary

Vault rereads, `ORA-01017` classification, synchronized refresh, connection retries, credential replacement, and automatic rotation remain separate follow-up work.
