---
quick_id: 260918-fnp
status: in_progress
description: Update Oracle FromDbValue lifetime bounds for oracledb beta.3
---

# Quick Task 260918-fnp Plan

Update the Oracle catalog value helpers for the lifetime-parameterized
`FromDbValue` trait introduced by `oracledb 26.0.0-beta.3`.

## Task 1: Update and verify the compatibility bounds

- **Files:** `controller/src/oracle_catalog.rs`
- **Action:** Apply a higher-ranked lifetime bound to both generic row-value
  helpers so they continue returning owned values after the source row drops.
- **Verify:** Run formatting, production-persistence checks and tests, and the
  release build used by the controller image.
- **Done:** Both controller CI paths compile with `oracledb 26.0.0-beta.3` and
  the production-feature test suite passes.
