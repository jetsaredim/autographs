---
status: complete
created: 2026-07-26
completed: 2026-07-26
branch: issue-175-bulk-catalog-derivative-cache
issue: 175
---

# Summary

Addressed issue 175 by optimizing catalog publication and derivative cache source validation.

## Completed

- Added checksum and ETag metadata to `AutographImage`.
- Populated image checksums on admin upload and replacement.
- Added `CatalogRepository::list_published` and an Oracle override that loads item rows plus tags, signer credits, characters, franchises, and images in grouped bulk queries.
- Updated static publishing to load published items through the repository bulk path.
- Updated derivative cache validation to use stored checksum/ETag source keys and read private originals only on cache miss or legacy rows without source metadata.
- Added timing/count instrumentation for catalog load and derivative cache source-key behavior.
- Added regression coverage for checksum-backed no-read incremental publish reuse and legacy fallback cache validation.

## Verification

- `cargo fmt`
- `cargo test`
- `cargo check --features production-persistence`
- `cargo test --features production-persistence oracle_`
- `cargo clippy`
