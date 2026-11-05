---
quick: 261104-ptm
status: planned
mode: quick
branch: gsd/quick-create-standalone-local-image-adjustment
---

# Local image review layout sketch

Create one standalone HTML mockup under `prototypes/` that reflects the live image review screen and addresses the annotated screenshots. Keep it independent of the production admin and image processing code so the layout can be reviewed before implementation.

## Tasks

1. Build a responsive, viewport-height preview stage with two larger dummy images and a compact control rail.
2. Put slider values beside labels, make overlays a single compact row, consolidate corner coordinates, move persistent explanations into click-accessible help, and give JSON readable contrast.
3. Verify the mockup loads locally at desktop and narrow widths, then record the design choices for the production follow-up.

## Done when

- The preview stage uses the full available left-column height at desktop size.
- Controls and JSON can be inspected without production credentials or image data.
- No production controller or admin files change.
