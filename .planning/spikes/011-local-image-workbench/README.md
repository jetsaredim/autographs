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
cargo run --manifest-path .planning/spikes/011-local-image-workbench/workbench/Cargo.toml -- \
  --source /path/to/private/images \
  --workspace /tmp/autographs-image-workbench
```

Then open `http://127.0.0.1:8787`. An alternate loopback port can be supplied
with `--port 8788`.

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
- Passed three workbench contract tests, strict Clippy, Rust formatting, the
  shared crate test, JavaScript syntax validation, and repository diff checks.
- The browser experience is running on loopback for the required operator
  checkpoint; no supported browser surface was available to the automation
  session, so visual usability remains intentionally unclaimed.

## Results

PENDING. Build, automated contract verification, and an operator experience
checkpoint remain required before the spike can be validated.
