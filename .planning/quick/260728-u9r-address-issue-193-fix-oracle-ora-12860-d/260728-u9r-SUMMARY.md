---
status: complete
quick_id: 260728-u9r
completed: 2026-07-29
commit: 341e425
---

# Quick Task 260728-u9r Summary

Addressed issue 193 by reducing Oracle signer-credit write churn during item updates.

## Changes

- Skipped Oracle signer-credit persistence when the admin save payload includes signer credits that resolve to the same persisted rows.
- Added update-time signer-credit synchronization that locks rows in signer-id order, deletes removed credits individually, updates existing rows, and inserts only new rows.
- Added feature-gated Oracle unit coverage for unchanged submitted signer credits and stable signer-credit row ordering.
- Cleaned up a clippy-blocking iterator loop in the same Oracle module.

## Verification

- `cargo fmt --check`
- `cargo test`
- `cargo test --features production-persistence oracle_`
- `cargo check --features production-persistence`
- `cargo clippy --features production-persistence -- -D warnings`
