---
status: resolved
trigger: "Image 31 remains labeled unstable and refuses automatic guidance after manual squaring"
created: 2026-10-17
updated: 2026-10-18
---

# Debug Session: Workbench Image 31 Stability

## Symptoms

- Expected behavior: reliable coarse geometry should remain actionable when one optional refined edge fit is weak, with the weak edge named explicitly.
- Actual behavior: image 31 withholds all automatic guidance after rotation because the fine-fit stability gate fails even though most displayed values are within tolerance.
- Related usability issue: raw top-left-origin X/Y coordinates make manual corner adjustment direction difficult to reason about.

## Evidence

- timestamp: 2026-10-17
  observation: Identity rendering detects a reliable four-corner coarse boundary but produces no refined edges.
- timestamp: 2026-10-17
  observation: The first coarse suggestion applies `-0.40°` rotation. The rerender produces four edge fits with top/right/bottom coverage of `100%`, `93.75%`, and `100%`, but left coverage is only `45.83%` against the `70%` prescription threshold.
- timestamp: 2026-10-17
  observation: The existing all-or-nothing fine-fit gate returns no suggestion instead of using the still-reliable coarse boundary. Opposing-edge divergence is also larger than the shared rigid tilt, so perspective should precede another rotation.
- timestamp: 2026-10-17
  observation: The operator's saved fixture uses rotation `-0.40°` and source corners `[(0,.02),(.995,0),(1,.99),(.004,1)]`. Its coarse output offsets are `-0.129°`, `-0.002°`, `-0.020°`, and `0°`, visually and numerically within tolerance.
- timestamp: 2026-10-17
  observation: The saved output's left fit remains at `41.67%` coverage because detection follows the preview frame edge near `x=.0016`, not the visible card edge. Alignment can be accepted from its orientation, but its framing position is unsafe for crop guidance.
- timestamp: 2026-10-18
  observation: The operator accepted image 31's saved output as well aligned after reviewing the updated controls and diagnostics. Live reassessment reports a `0.25°` worst edge deviation, `0.37°` worst corner deviation, and no further numerical adjustment worth suggesting.

## Current Focus

- hypothesis: Fine-edge quality is being treated as authority over coarse reliability instead of an optional precision enhancement, and the operator controls expose storage coordinates instead of directionally consistent adjustments.
- test: Fall back to coarse corners for suggestion and acceptance scoring when fine fits are unstable; correct large opposing-edge divergence before rotation; expose positive inward corner insets.
- expecting: Image 31 continues through conservative perspective guidance while clearly identifying the weak left edge, and every positive corner input moves inward from its labeled source edge.
- next_action: Resolved. Keep the weak left edge visible as a framing-confidence limitation while continuing framing UAT.

## Resolution

- root_cause: Fine-edge coverage was an all-or-nothing prescription gate. A single fit following the preview frame rather than the card edge suppressed otherwise valid coarse alignment, while raw top-left-origin storage coordinates obscured manual correction direction.
- fix: Unstable fine fits now fall back to coarse geometry for alignment suggestions and consistent scoring; perspective divergence precedes rigid rotation; rejected perspective passes back off; diagnostics name weak edges; manual controls expose positive inward edge distances; unstable positional boundaries explicitly make framing provisional.
- verification: JavaScript syntax, formatting, diff checks, release clippy, and all 7 release workbench tests pass. An API-backed image-31 simulation retains improving coarse-perspective passes before the detector discontinuity, the saved fixture is correctly classified as coarse alignment complete with no numerical suggestion, and the operator accepted its visual alignment on 2026-10-18.
- files_changed: `.planning/spikes/011-local-image-workbench/workbench/static/app.js`, `.planning/spikes/011-local-image-workbench/workbench/static/index.html`, `.planning/spikes/011-local-image-workbench/workbench/src/main.rs`
