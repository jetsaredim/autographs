---
status: complete
completed: 2026-07-16
branch: gsd/quick-add-admin-signer-profile-management-tab-
---
# Add Admin Signer Profile Management Tab

Implemented a top-level static admin Signers tab for reusable signer profile edits while keeping new signer creation inside the item editor.

## Completed

- Added Signers tab/search/edit workflow for reusable signer display name, default role, Wikipedia URL, and IMDb URL.
- Added Manage profile links from item editor signer rows and signer deep links from the Items table.
- Removed reusable profile URL editing from item signer rows so item edits stay focused on signer name, item role, and context.
- Added signer-specific unsaved-edit guard copy, focused signer highlighting, and private-save/publish reminder copy.
- Added `signerIds` to admin item summaries to support item-list signer profile links.
- Updated static admin and admin workflow tests for the new contract.

## Verification

- `cargo fmt --check`
- `node --check controller/static-admin/admin.js`
- `cargo test --test static_admin`
- `cargo test --test admin_workflow`
