---
status: complete
completed: 2026-07-02
---

# Summary

Implemented the requested admin UI cleanup and public detail metadata improvements:

- Removed redundant hub CTA buttons; top tabs remain the primary navigation.
- Reworked admin status into collapsible sections for publish state, pending changes, cleanup warnings, runtime/retention, and redacted diagnostics.
- Added redacted cleanup warning rows to `/admin/api/status`.
- Replaced item list text actions with accessible icon buttons and tooltips.
- Added collapsible item filters, a changes filter, and title/signer sorting.
- Added public detail groups for Story, Provenance, Certification, object reference, and estimated year when present.
- Removed the empty selected-filter spacer that made public collection filters look bottom-heavy.

Verification:

- `node --check controller/static-admin/admin.js`
- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo check --manifest-path controller/Cargo.toml`
