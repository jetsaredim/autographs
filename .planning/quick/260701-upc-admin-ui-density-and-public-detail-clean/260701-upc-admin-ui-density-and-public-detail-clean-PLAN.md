---
status: complete
created: 2026-07-02
---

# Admin UI Density and Public Detail Cleanup

## Goal

Tighten the admin workflow UI by removing redundant hub actions, making dense status information collapsible, improving item list scanning and filtering, and restoring missing public item detail metadata.

## Scope

- Remove redundant large hub buttons in favor of the top workflow tabs.
- Convert crowded admin hub status tiles into collapsible dashboard sections.
- Replace item row text actions with accessible icon buttons.
- Make admin item filters visually separate and collapsible; add a changes filter and sortable title/signer headers.
- Render applicable Story, Provenance, Certification, and object/year detail fields on public item pages.
- Reduce excess public collection filter spacing.

## Verification

- `node --check controller/static-admin/admin.js`
- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo check --manifest-path controller/Cargo.toml`
- Focused controller tests for publisher/admin workflow changes.
