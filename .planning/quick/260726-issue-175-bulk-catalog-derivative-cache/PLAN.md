---
status: in-progress
issue: 175
---

# Quick Task: Address issue 175 bulk catalog loading and derivative cache validation

## Goal

Optimize metadata-only static publishes by reducing Oracle catalog list query count and avoiding private original reads when image checksum metadata is present.

## Plan

1. Carry stored image checksum/etag metadata through `AutographImage` and Oracle image insert/update/load paths.
2. Populate checksum metadata during admin upload and replacement from the validated original bytes.
3. Add a bulk Oracle catalog list path that loads item rows and child rows in grouped queries, with a published-only variant for static publish.
4. Update static publishing to use the published-only repository path and use checksum/etag-backed derivative cache source keys, preserving byte-hash fallback for legacy images.
5. Add timing instrumentation for catalog loading and derivative validation/reuse.
6. Add tests for checksum-backed no-read cache reuse and legacy no-checksum fallback behavior.
7. Run focused controller tests and formatting/checks as practical.
