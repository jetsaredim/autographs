---
status: resolved
trigger: "An admin item with two signers renders a taller signer cell, but the taxonomy-cell bottom border does not align with the other cells in the row."
created: 2026-09-19
updated: 2026-09-19
---

# Debug Session: Admin Taxonomy Cell Row Height

## Symptoms

- Expected behavior: every item-list cell should stretch to the full table-row height, including rows made taller by multiple signer links.
- Actual behavior: the taxonomy-cell bottom border ends above the signer, state, actions, and title cell borders on the one multi-signer item.
- Error messages: none; this is a visual layout defect.
- Timeline: observed on the current live admin interface.
- Reproduction: open the admin item list and inspect the item with two signers.

## Current Focus

- hypothesis: confirmed — applying `display: grid` directly to the taxonomy `<td>` removed native table-cell sizing, so its border followed its intrinsic content height instead of the row height.
- test: kept the `<td>` as a native table cell and moved the grid layout to a child wrapper.
- expecting: confirmed structurally — the taxonomy cell now participates in table row sizing while its primary and secondary labels remain stacked.
- next_action: visually confirm against the live multi-signer item after deployment.

## Evidence

- 2026-09-19: `.taxonomy-cell` applies `display: grid` directly to a `<td>`; signer, state, title, and action cells retain native table-cell display.
- 2026-09-19: the taxonomy cell was introduced with this direct grid display in commit `2c0fb9a7`.
- 2026-09-19: the reported mismatch occurs only when another cell makes the row taller, matching intrinsic-height behavior on the non-table taxonomy cell.
- 2026-09-19: live verification confirmed the taxonomy border fix; the same multi-signer row exposed a separate wrapped hyphenated name caused by the signer pill's `max-width: 100%` and permissive wrapping.
- 2026-09-19: the item table allocated 20% to signer and 24% to taxonomy; live review favored reversing those allocations to give signer names more room.

## Eliminated

- hypothesis: the multi-signer cell itself fails to contribute row height.
  reason: the title, state, actions, and signer borders all align at the correct taller row boundary.

## Resolution

- root_cause: `.taxonomy-cell` changed the taxonomy `<td>` from its native `table-cell` display to `grid`, so it did not stretch its border to a row made taller by multiple signer links.
- fix: added a `.taxonomy-cell-content` child wrapper and moved the grid layout to that wrapper, leaving the `<td>` in the table formatting context. Kept signer-link pills on one line, widened the signer column from 20% to 24%, narrowed taxonomy from 24% to 20%, and allowed the table's existing overflow container to handle narrow viewports.
- verification: `node --check controller/static-admin/admin.js`; `cargo test --manifest-path controller/Cargo.toml --test static_admin` (14 passed). Browser visual verification was unavailable because no browser session was connected.
- files_changed: `controller/static-admin/admin.js`; `controller/static-admin/admin.css`; `controller/tests/static_admin.rs`.
