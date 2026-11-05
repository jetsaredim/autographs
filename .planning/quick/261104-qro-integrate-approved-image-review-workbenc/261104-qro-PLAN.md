---
quick: 261104-qro
status: planned
mode: quick-full
branch: gsd/quick-create-standalone-local-image-adjustment
---

# Integrate approved image review workbench layout

Apply the approved standalone sketch to the existing Rust controller's static admin while preserving image adjustment, preview, save/reset, and session behavior.

## Tasks

1. Update the review markup and CSS for a tall preview stage, larger image frames, compact control rail, inline slider values, one-row overlays, shared coordinate headers, and click-open help popups.
2. Replace the narrow inline JSON panel with a centered saved/draft dialog. Highlight changed values while showing the exact saved and canonical draft JSON objects already maintained by review state.
3. Extend focused tests for the layout and JSON behavior, run static admin and controller checks, commit implementation, push the branch with required escalation, and open a ready-for-review PR. Preserve a PR audit trail for review findings and fixes.

## Verification

- `node controller/tests/static_admin_behavior.mjs`
- `cargo test --test static_admin` from `controller/`
- `cargo fmt --check` and `cargo check` from `controller/` if relevant to changed files
- Browser render of the admin workbench with representative review state at desktop and narrow widths where practical.
