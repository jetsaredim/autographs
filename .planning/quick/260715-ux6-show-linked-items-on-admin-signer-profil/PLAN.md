---
status: complete
created: 2026-07-16
branch: fix/admin-item-list-signer-cell-layout
---
# Show Linked Items On Admin Signer Profile Editor

Goal: Show the items linked to a reusable signer profile directly from the Signers admin editor.

Tasks:
- Load linked item summaries for each visible signer profile using the existing admin item list endpoint.
- Render a compact linked-items section with edit/history actions.
- Update static admin tests and focused validation.
