---
spike: 011
idea: shared-image-processing
name: local-image-workbench
type: standard
validates: "Given a local image file or directory and an external workspace, when the workbench runs on loopback, then an operator can detect, numerically adjust, preview, and persist the exact production adjustment JSON through the shared crate without an admin, catalog, or cloud environment."
verdict: VALIDATED
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
- The supported workflow is one local operator in one browser session.
  Concurrent external API writers, multiple browser sessions, and multi-user use
  are unsupported.
- No private image, fixture, or rendered output belongs in Git. The committed
  code is a disposable spike harness, not a production runtime.

## How to Run

```bash
mkdir -p /tmp/autographs-image-workbench
cargo run --release --manifest-path .planning/spikes/011-local-image-workbench/workbench/Cargo.toml -- \
  --source /path/to/private/images \
  --workspace /tmp/autographs-image-workbench
```

The process prints a URL such as
`http://127.0.0.1:8788/?cap=<emitted-token>`. Copy and open that exact URL; the
random capability is required for every private API request. The bare
`http://127.0.0.1:8788` shell can load, but its API requests correctly receive
`403 Forbidden` because the capability is missing. Treat the emitted URL as a
sensitive local credential: do not put a live token in screenshots, issues,
logs, or other shared artifacts.

Port 8788 is the single standard workbench port. Stop the running instance
before rebuilding or restarting it; if the bind fails because 8788 is occupied,
stop that instance rather than selecting another port. The UI displays its
active address, and shell assets use `Cache-Control: no-store` so a restarted
workbench cannot silently reuse an earlier revision.

## What to Expect

The browser lists allowlisted images, renders the selected original and adjusted
preview, runs the production detector, exposes numeric adjustment and corner
coordinates, supports fine keyboard stepping, draws configurable alignment
overlays on the preview, and saves or reloads the exact production adjustment
JSON. Saving a fixture does not alter the original.

The corrected preview also provides a measurement quadrilateral initialized from
boundary detection on every rendered preview. Its edges are compared with the
fixed horizontal and vertical grid axes. A dashed quadrilateral identifies a
tentative boundary that is useful for measurement but did not satisfy the safer
automatic-correction contract. Dragging any orange handle switches that image to
a browser-local manual override; **Use automatic detection** discards the override
and resumes measurement from each render. Measurements remain separate from
adjustment fixtures and production data. To reopen a specific image, use the UI
after launching from the emitted URL or retain the capability explicitly as
`?cap=<emitted-token>&image=N`, where `N` is its zero-based list index. A bare
`?image=N` link cannot authorize private API access.

For reliable automatic boundaries, the framing panel measures photographed
background in rendered pixels and normalizes each side against the item's
shorter edge. It reports opposite-side allocation, the maximum symmetric band
that remains inside real pixel support, and an advisory status using an initial
4% target. Weak refined edges remain provisional. When automatic detection is
unsuitable, an operator can position all four orange handles and explicitly
confirm that manual boundary for real-pixel framing assessment. Changing the
boundary or adjustment invalidates the confirmation. Framing assessment never
changes crop, zoom, pan, corners, or the saved fixture.

When reliable support can improve the framing, **Preview crop** draws a cyan
rectangle and dims the pixels that would be removed without changing the draft.
**Apply to draft** records one explicit crop after perspective and rotation,
while **Undo framing** restores the complete prior draft. Adjustable images use
the configured target. A constrained thin-border image cannot be described as
meeting that target, but materially uneven opposite margins can still offer
explicit **Preview recentering** and **Apply recentering** actions. Recentering
preserves the limiting real-pixel border, remaps a confirmed manual boundary
through the crop, and automatically refreshes the assessment. Saving the
fixture remains a separate operator action.

## Supported Save Scope

Save is single-flight in the browser: an accidental repeated activation while a
save is pending sends no second request. Editing or navigating during that save
does not cancel the accepted fixture write, but its stale completion cannot
overwrite or claim to represent the current browser draft. After settlement,
the operator can explicitly save the then-current snapshot.

Fixture publication uses a uniquely created same-workspace temporary file and a
same-directory replacement. This removes the shared temporary-path collision,
but it does not define a winner for concurrent direct API requests. Concurrent
external writers and multi-user use remain unsupported.

Before productizing this workbench for multiple users, browser sessions,
background writers, or production persistence, design and verify all of the
following as one conflict contract:

- durable record versions or ETags with explicit conflict behavior;
- authorization plus actor and audit identity;
- serialized or compare-and-swap commits with a defined winner;
- conflict and retry UX that cannot silently overwrite another editor; and
- deterministic concurrent-write, recovery, and durability tests.

Until that contract exists, multi-user productization is blocked.

## Capability-Aware Smoke Check

Run the command above with a disposable external workspace, then use only the
exact capability-bearing URL printed by that process:

1. Confirm the image list loads and a selected private source renders.
2. Change a numeric adjustment, save it, reload it, and confirm the saved value
   returns without modifying the source image.
3. Reopen the selected image through the UI deep link and confirm the URL still
   contains both `cap` and `image`.
4. In a separate request or tab, omit `cap` from a private `/api/*` request and
   confirm it receives `403 Forbidden`.

Do not record the live token or private image content while capturing smoke
evidence.

## Investigation Trail

- Spike 010's extracted crate shipped in v0.4.2 and passed authenticated editor
  and incremental-publish production validation before this spike began.
- The workbench deliberately uses a path dependency on that crate instead of
  copying detector or transformation logic.
- Loaded the full 116-image private corpus without copying any original into
  the repository or workspace.
- Verified list, source, identity-render, detector, fixture-save, and
  fixture-load endpoints through the exact emitted capability URL against a real
  corpus image. The rendered response was a valid WebP, and the source SHA-256
  remained identical before and after the fixture round trip. The same private
  API correctly rejects a missing capability.
- Passed eight workbench contract tests, strict Clippy, Rust formatting, the
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
- The saved `wb-1.png` reference resolved to corpus index 7 and fixture
  `0007-adjustment.json`. Because its rendered boundary remained below
  the conservative detector's confidence gate, the preview now supports a
  persistent manual measurement quadrilateral. Four independently draggable or
  keyboard-nudgeable output handles color each edge against the fixed grid axes
  and report aspect-correct edge, corner, and opposite-side alignment without
  changing the saved production adjustment.
- Preview measurement now uses a separate shared-crate boundary-location
  contract. Unlike the production correction proposal, it returns near-identity
  boundaries and exposes lower-confidence candidates as tentative. The orange
  quadrilateral therefore follows each new preview automatically while the
  stricter production auto-correction abstention behavior remains unchanged.
- Source-supported framing assessment now distinguishes photographed pixels
  from transparent wedges created by geometric transforms. The initial UAT
  model uses a configurable 4% target, a 3–6% trial range, and a 45/55 balance
  threshold, while keeping crop application out of scope until the assessment
  proves useful across representative images.
- After representative assessment UAT, the workbench gained an explicit crop
  preview/apply/undo loop. The overlay and dismissal are non-mutating, applied
  crops remain unsaved drafts, and constrained images use one pixel less than
  the measured maximum symmetric band to avoid transform-edge rounding.
- Operator testing found that SVG overlays could remain painted after hiding or
  applying a crop. The workbench now reflects hidden state explicitly, clears
  stale proposals synchronously, and covers the behavior with a contract test.
- Slab testing established the accepted manual fallback: explicitly confirmed
  output handles can drive real-pixel framing without being mistaken for
  automatic geometry, and any unrelated geometry change invalidates them.
- Thin-border testing established that insufficient background prevents a claim
  of target framing but does not prevent conservative recentering. The final
  workflow preserves the limiting border, trims only excess real pixels, carries
  the confirmed boundary through the crop, and reassesses automatically.
- The final release workbench was relaunched with all eight tests passing on
  2026-10-18. The operator accepted the experiment after exercising alignment,
  automatic and manual framing, crop overlays, constrained cases, slab fallback,
  and the final thin-border recentering behavior.

## Results

VALIDATED. The shared image-processing crate supports a private, loopback-only
workbench for automatic and manual alignment, honest boundary diagnostics,
real-pixel framing assessment, explicit crop/recentering previews, draft-local
application, undo, and exact adjustment fixtures without an admin, catalog, or
cloud environment. The detector intentionally abstains on uncertain subjects;
the validated operator fallback handles those cases without weakening privacy
or claiming that cropping can create missing background.
