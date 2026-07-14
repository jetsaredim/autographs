---
status: complete
completed: 2026-07-14
branch: gsd/quick-refine-admin-item-list-vertical-alignmen
---

# Summary

Refined the admin item list density and alignment after the status/change icon update.

## Changes

- Vertically centered all item-list cells so title, signer, format, franchise/product, updated, icons, and action buttons align on the row centerline.
- Replaced long absolute updated timestamps in the item list with compact relative labels such as `5m ago`, while preserving the exact timestamp as hover text.
- Combined the publish-status action into the publication status icon button and removed the separate publish-status action button.
- Reduced the action column from three icon buttons to two, narrowed the updated/action columns, and lowered the table minimum width.
- Updated static admin source contract checks for the relative date and clickable status icon behavior.

## Verification

- `node --check controller/static-admin/admin.js`
- `cargo test --test static_admin`

## Follow-Up Suggestions

- If the list still feels crowded, the next best space win is collapsing `Franchise / Product Line` into a stacked two-line cell or showing product line only on hover.
- Consider adding a small primary-image thumbnail column later if visual recognition becomes more important than maximum metadata density.
