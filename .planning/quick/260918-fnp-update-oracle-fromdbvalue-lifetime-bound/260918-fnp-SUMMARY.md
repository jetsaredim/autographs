---
quick_id: 260918-fnp
status: complete
implementation_commit: 9301c6a
completed: 2026-09-18
---

# Quick Task 260918-fnp Summary

Updated the Oracle catalog value helpers for the lifetime-parameterized
`FromDbValue` trait introduced by `oracledb 26.0.0-beta.3`.

## Delivered

- Added a higher-ranked `for<'a> FromDbValue<'a>` bound to `row_value` and
  `query_row_value`.
- Preserved the helpers' owned-value contract so `query_row_value` cannot
  return data borrowing from its local Oracle row.

## Verification

- `cargo fmt --check` — passed.
- `cargo test --tests --features production-persistence` — 202 passed, 2
  ignored live-credential smoke tests.
- `cargo build --release --features production-persistence` — passed, matching
  the controller Dockerfile's failing compilation path.

Implementation commit: `9301c6a`
