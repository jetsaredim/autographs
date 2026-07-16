---
status: complete
completed: 2026-07-16
branch: fix/admin-item-list-signer-cell-layout
---
# Show Linked Items On Admin Signer Profile Editor

Added a compact linked-items section to the Signers admin editor.

## Completed

- Each rendered signer profile now loads admin item summaries matching the signer name.
- Linked items are filtered exactly by `signerIds.includes(profile.id)`.
- The Signers editor shows linked item count, title, publication/taxonomy summary, and edit/history actions.
- Signer links in the admin item list keep the requested no-underline style while gaining a subtle pill border for non-color affordance.
- Oracle signer profile updates pre-load linked item signer text before updating `autograph_signers`, avoiding same-transaction reads that can trigger `ORA-12838`.
- Static admin tests cover the linked-items contract.

## Verification

- `node --check controller/static-admin/admin.js`
- `cargo test --test static_admin`
- `cargo test --test admin_workflow signer_profile_edits_record_history_for_linked_items_only`
- `cargo check --features production-persistence`
