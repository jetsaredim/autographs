---
status: needs_review
completed: 2026-11-13
branch: sketch/compact-item-editor-workbench
commit: aa6e1a8
---

# Summary

Applied the approved stacked-section item editor layout to the production admin page while retaining its normal 1120px shell.

## Changes

- Condensed the add/edit form into compact, full-width sections with three-column field rows on desktop.
- Changed item image tiles into full-width records. Saved adjustments show independent unadjusted source and saved-result previews; unadjusted images keep one preview.
- Kept the same-origin private preview endpoints, revision checks, retry controls, and image actions.
- Updated the DOM behavior and static admin contract tests for the new presentation.

## Verification

- `node --check controller/static-admin/admin.js`
- `node controller/tests/static_admin_behavior.mjs`
- `cargo test --test static_admin`
- `cargo fmt --check`
- `git diff --check`
- Local Chrome renders at 1440px and 390px using the production admin markup and CSS with dummy media.

The local render uses dummy media and does not verify an authenticated production session.
