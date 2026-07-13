---
status: complete
completed: 2026-07-13
branch: gsd/quick-fix-admin-item-list-visual-alignment-and
---

# Summary

Fixed the admin item-list visual alignment and reduced list clutter by rendering publication status and pending-change state as compact, accessible icon badges.

## Changes

- Added reusable static-admin icon badge helpers for publication status and pending changes.
- Replaced visible status/change words in the item table with icon-only badges that retain `aria-label` and `title` text.
- Kept row action buttons inside an inner `.row-actions` wrapper so the action `<td>` keeps the same table-cell border alignment as the rest of the row.
- Added item-table column sizing and centered compact status/image/change/action columns.
- Updated static admin source contract coverage for the new icon badge and action-cell structure.
- Addressed review feedback by widening the action column and adding an order-sensitive item-list icon/accessibility contract test.

## Verification

- `node --check controller/static-admin/admin.js`
- `cargo test --test static_admin`

## Product Note

Broader thumbnail support for semantic facets like franchise, product line, format, or set makes sense, but it should be modeled as curated taxonomy media rather than inferred from arbitrary item thumbnails. A good follow-up would add optional taxonomy icon/thumbnail assets plus metadata references, then render them in public facet browsing and admin suggestions while preserving text fallbacks and avoiding copyrighted/logo ambiguity unless the assets are explicitly approved.
