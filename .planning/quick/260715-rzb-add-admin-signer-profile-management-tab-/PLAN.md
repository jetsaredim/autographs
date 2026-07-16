---
status: complete
created: 2026-07-16
branch: gsd/quick-add-admin-signer-profile-management-tab-
---

# Add Admin Signer Profile Management Tab

## Goal

Add a signer profile management workflow to the static admin UI while keeping new signer creation in the item editor.

## Tasks

- Add a top-level Signers tab with search/list results.
- Let operators edit existing signer profile fields: display name, default role, Wikipedia URL, IMDb URL.
- Preserve item-editor signer creation and item-specific credit editing.
- Add an item-editor bridge button for existing signers that opens the Signers tab focused on that profile.
- Keep shared-profile fields disabled in the item editor and make that handoff discoverable.
- Update static admin contract tests and run focused validation.
