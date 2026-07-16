---
status: complete
created: 2026-07-16
branch: gsd/quick-add-admin-signer-profile-management-tab-
---
# Normalize Signer IMDb And Wikipedia Profile Links

Goal: Store compact signer profile identifiers for IMDb and Wikipedia short links without a schema migration.

Tasks:
- Normalize admin/API signer profile values to `nm...` IMDb IDs and `w.wiki` short IDs.
- Keep canonical full URL inputs accepted and render full public URLs from stored IDs.
- Update static admin labels/tests and repository validation tests.
