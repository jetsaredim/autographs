---
gsd_task_version: 1.0
task: reorder-showcase-before-admin
status: complete
completed_at: "2026-05-25T06:30:00Z"
---

# Quick Task Summary: Reorder Showcase Before Admin

## Completed

- Reordered `.planning/ROADMAP.md` so Public Showcase and Hardening is now Phase 4.
- Updated Admin Collection Workflow to Phase 5 and AI-Assisted Ingest to Phase 6.
- Updated `.planning/REQUIREMENTS.md`, `.planning/STATE.md`, and `.planning/PROJECT.md` to match the new order.
- Added a boundary note that Phase 4 hardens the current public-gallery/deployment surface, while later admin and AI phases remain responsible for their new surfaces.

## Verification

- Stale-reference scan across top-level planning files passed.
- `git diff --check` passed.
