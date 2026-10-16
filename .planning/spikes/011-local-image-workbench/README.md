---
spike: 011
idea: shared-image-processing
name: local-image-workbench
type: standard
validates: "Given a local image file or directory and an external workspace, when the workbench runs on loopback, then an operator can detect, numerically adjust, preview, and persist the exact production adjustment JSON through the shared crate without an admin, catalog, or cloud environment."
verdict: PENDING
related: [008, 009a, 009b, 010]
tags: [rust, image-processing, workbench, fixtures, localhost, privacy]
---

# Spike 011: Local Image Workbench

## What This Validates

The extracted image-processing crate can support an experiential, fixture-driven
editor without the controller's admin, authentication, catalog, Oracle, OCI, or
publishing layers. The workbench must read private originals in place, never
modify them, and persist only adjustment JSON in an explicitly selected external
workspace.

## Boundaries

- The server binds to `127.0.0.1` only.
- Input is one local image or a non-recursive directory of JPEG, PNG, and WebP
  images.
- Source images are addressed through an in-memory allowlist; request paths
  cannot select arbitrary files.
- The source path is read-only. Fixture JSON is written only beneath the
  separately supplied workspace.
- Detector proposals and adjusted previews call `autographs-image-processing`
  directly.
- No private image, fixture, or rendered output belongs in Git. The committed
  code is a disposable spike harness, not a production runtime.

## How to Run

```bash
mkdir -p /tmp/autographs-image-workbench
cargo run --release --manifest-path .planning/spikes/011-local-image-workbench/workbench/Cargo.toml -- \
  --source /path/to/private/images \
  --workspace /tmp/autographs-image-workbench
```

Then open `http://127.0.0.1:8788`. This is the single standard workbench port.
Stop the running instance before rebuilding or restarting it; if the bind fails
because 8788 is occupied, stop that instance rather than selecting another port.
The UI displays its active address, and shell assets use `Cache-Control: no-store`
so a restarted workbench cannot silently reuse an earlier revision.

## What to Expect

The browser lists allowlisted images, renders the selected original and adjusted
preview, runs the production detector, exposes numeric adjustment and corner
coordinates, supports fine keyboard stepping, draws configurable alignment
overlays on the preview, and saves or reloads the exact production adjustment
JSON. Saving a fixture does not alter the original.

## Investigation Trail

- Spike 010's extracted crate shipped in v0.4.2 and passed authenticated editor
  and incremental-publish production validation before this spike began.
- The workbench deliberately uses a path dependency on that crate instead of
  copying detector or transformation logic.
- Loaded the full 116-image private corpus without copying any original into
  the repository or workspace.
- Verified list, source, identity-render, detector, fixture-save, and
  fixture-load endpoints against a real corpus image. The rendered response was
  a valid WebP, and the source SHA-256 remained identical before and after the
  fixture round trip.
- Passed four workbench contract tests, strict Clippy, Rust formatting, the
  shared crate test, JavaScript syntax validation, and repository diff checks.
- The browser experience is running on loopback for the required operator
  checkpoint. Automated headless checks cover layout and loading, while overall
  usability remains intentionally unclaimed until the operator retest.
- The first operator check exposed progressive JPEG scanlines as an apparently
  clipped top portion. The UI now hides partial browser paints behind an
  explicit source-loading state, gives both image elements fixed contain boxes,
  and computes overlays from the actual letterboxed bounds. A headless browser
  check confirmed the complete portrait and preview are visible. The runbook
  uses an optimized build so full-resolution transforms complete promptly.
- A follow-up retest exposed four simultaneously running workbench revisions on
  ports 8787 through 8790. All were stopped, 8788 became the sole default and
  documented port, and the runtime now identifies its address in the UI. Port
  hopping is no longer part of the workbench test procedure.
- Operator feedback showed that the original geometry colors were easy to read
  as an output-quality score even though they described only the source
  quadrilateral and ignored the subsequent rotation. The readout now separates
  neutral source-selection geometry (including the applied rotation) from
  corrected-output alignment. Output scores appear only when the shared detector
  finds a reliable boundary in the rendered preview; otherwise the UI explicitly
  defers to visual grid alignment rather than manufacturing a score.

## Results

PENDING. Build, automated contract verification, and an operator experience
checkpoint remain required before the spike can be validated.
