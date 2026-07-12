# Deferred Items

## 2026-07-10 - Plan 07-05 verification

- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`
  failed on pre-existing Phase 7 code outside the Plan 07-05 file set:
  `controller/src/catalog.rs` has a derivable `Default` implementation for
  `ItemOrigin`, and `controller/src/taxonomy_migration.rs` has consecutive
  `str::replace` calls. These should be handled in a focused follow-up because
  Plan 07-05 is limited to rollout docs, security/live-smoke assertions, and
  codebase maps.
