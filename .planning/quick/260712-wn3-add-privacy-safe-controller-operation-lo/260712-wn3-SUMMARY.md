---
status: complete
completed: 2026-07-13
---

# Quick Task 260712-wn3 Summary

Added broader privacy-safe controller operation logging beyond the publish path.

## Changes

- Added admin status, login/logout, item list/get/history, signer suggestion/update/merge, taxonomy suggestion, image upload, primary image, delete, replace, cleanup retry, and media validation/rejection logs.
- Removed private media object keys from route tracing fields and replaced raw media-provider errors with coarse `media_*` error categories.
- Removed unvalidated raw route parameters from auth-rejection logs, including the publication update route caught by review.
- Added `controller/tests/logging_contract.rs` to prevent route/publisher tracing blocks from logging private object-key, filename, bucket, namespace, secret, token, and password terms.
- Extended the contract to catch auth-rejection tracing blocks that include raw path parameters before parsing.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml`
- `cargo test --manifest-path controller/Cargo.toml --test logging_contract`
- `cargo test --manifest-path controller/Cargo.toml`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo clippy --manifest-path controller/Cargo.toml -- -D warnings`
