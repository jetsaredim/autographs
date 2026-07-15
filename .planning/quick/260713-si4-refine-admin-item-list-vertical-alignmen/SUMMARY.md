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
- Removed the low-signal format column from the item list and changed franchise/product line into a compact two-line taxonomy cell.
- Regrouped publication status, image count, pending-change state, and updated time into a single `State` column with left-aligned text and right-aligned icons near row actions.
- Matched the publish-status icon button size to the pending-change badge and ordered the state icons as changes first, publication status second.
- Unified state and row action icon containers at 36px circular controls.
- Added visible `Signing in...` and `Loading items...` states so slow admin API calls no longer make the UI feel frozen.
- Unified positive/success icon coloring with the admin teal accent while preserving amber for pending/warning states.
- Replaced generic item loading copy with request/render phases: requesting item summaries, then preparing the returned item count.
- Addressed PR review feedback by moving item-list loading announcements to a separate `role=status` node and using status semantics, not alert semantics, for login progress.
- Kept visual loading placeholders out of live-region semantics while the item table wrapper is marked busy.
- Updated static admin source contract checks for the relative date and clickable status icon behavior.

## Verification

- `node --check controller/static-admin/admin.js`
- `cargo test --test static_admin`

## Follow-Up Suggestions

- If the list still feels crowded, the next best space win is showing product line only on hover or moving taxonomy under the title.
- Consider adding a small primary-image thumbnail column later if visual recognition becomes more important than maximum metadata density.
