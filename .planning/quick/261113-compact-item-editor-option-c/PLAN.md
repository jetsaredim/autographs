---
status: complete
created: 2026-11-13
branch: sketch/compact-item-editor-workbench
---

# Apply compact item editor option C

## Goal

Apply the approved local option C to the production add/edit item page at the normal 1120px admin width. Keep editable sections stacked across the page and show source and saved image previews side by side when adjustment JSON exists.

## Tasks

- Reuse the local workbench to compare the implementation with the approved layout.
- Tighten the existing admin form spacing and arrange fields in compact rows.
- Render full-width image records with independent source and saved previews for adjusted images.
- Preserve single-preview behavior for unadjusted images and existing image actions.
- Verify behavior tests, static admin contracts, and desktop/mobile rendering.
