# Deferred Items

## 2026-07-10 - Plan 07-05 verification

- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`
  failed on pre-existing Phase 7 code outside the Plan 07-05 file set:
  `controller/src/catalog.rs` has a derivable `Default` implementation for
  `ItemOrigin`, and `controller/src/taxonomy_migration.rs` has consecutive
  `str::replace` calls. These should be handled in a focused follow-up because
  Plan 07-05 is limited to rollout docs, security/live-smoke assertions, and
  codebase maps.

## 2026-07-17 - Phase 8 taxonomy thumbnail exploration

- Phase 7 established first-class `productLine` and `setName` metadata, but
  public detail pages still render those values as text only.
- Defer image support for product-line and set identity to Phase 8
  AI-assisted ingest/taxonomy-media work. Explore whether AI can help identify,
  crop, and validate small public-safe derived thumbnails, such as recognizable
  card-back cues for product lines or sets.
- Keep text metadata as the source of truth. Any thumbnail support should be
  optional, graceful when absent, and explicitly reviewed for privacy/copyright
  risk before publication.
