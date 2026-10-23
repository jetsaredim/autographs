---
quick: 261017-fdg
status: complete
checkpoint: accepted
---

# Refine workbench preview alignment measurements

## Task 1: Refine detected sides

- Keep the current boundary detector as the coarse locator.
- Sample nearby Canny edge pixels and fit each side independently at sub-degree precision.
- Return refined corners, edge coverage, straightness residual, and bow deviation without changing the production auto-correction decision.

## Task 2: Improve the workbench readout

- Calculate edge-axis and parallelism measurements from the refined fits.
- Show straightness, bow, and coverage diagnostics with honest precision.
- Preserve automatic measurement and manual-override behavior.

## Task 3: Validate locally

- Add focused image-processing and workbench contract tests.
- Run formatting, tests, Clippy, JavaScript validation, and the saved reference fixture.
- Relaunch the sole workbench instance on `127.0.0.1:8788` for operator review.
- Do not push these changes to PR #281 until the operator approves the result.

## Task 4: Add advisory next-step values

- Convert reliable preview alignment measurements into one conservative next experiment.
- Recommend a rotation value first when the edges share a meaningful rigid tilt.
- Otherwise recommend trial source-corner values when opposing edges show perspective divergence.
- Never start correction automatically; one explicit operator action may run a bounded render-and-reassess loop to tolerance.
- Retain only improving passes and stop on tolerance, unreliable detection, a repeated adjustment, negligible improvement, or the iteration cap.
- Keep the displayed preview and diagnostics stable during internal passes, then publish the best result once.
- Bootstrap from reliable coarse corners when four refined edge fits are unavailable, then retry fine measurement after each broad correction.
- Validate rotation against rigid-tilt error and perspective against residual tilt/parallelism so proposals and acceptance use matching objectives.
- Back off rejected rotation proposals through smaller bounded steps so detector boundary changes do not leave the workflow repeating the same unusable full correction.
- Do not present straightness or bow as correctable by the four-corner transform.

## Task 5: Add assessment-only framing diagnostics

- Name the specific output corner with the largest deviation from square.
- Measure the top, right, bottom, and left background bands between the detected item boundary and the rendered preview frame.
- Report vertical and horizontal background allocation as intuitive side-to-side splits, with conservative balance indicators.
- Keep framing descriptive: do not apply crop, zoom, pan, or corner changes and do not claim a fixed aesthetic margin target.

## Task 6: Clarify manual corner controls and unstable fine-edge fallback

- Present corner values as positive inward distances from each corner's adjacent source edges while retaining canonical image-space coordinates in adjustment JSON.
- Name the weakest fitted edge in straightness and coverage diagnostics.
- Treat low-coverage or high-residual fine fits as unavailable for prescriptions and acceptance scoring, then fall back to the reliable coarse boundary rather than withholding all guidance.
- Correct perspective divergence before rigid rotation when opposing edges disagree materially.

## Task 7: Assess source-supported background framing

- Normalize top, right, bottom, and left margins against the detected item's shorter pixel edge.
- Distinguish real photographed pixels from transparent transform support and report the maximum feasible symmetric band.
- Start UAT with a configurable 4% target, 3–6% trial range, and 45/55 opposite-side balance threshold.
- Classify reliable results as Balanced, Adjustable, Constrained, or Retake recommended while keeping unstable or manual boundaries explicitly provisional.
- Do not mutate crop, zoom, pan, source corners, or fixture JSON from framing assessment.

## Task 8: Preview and explicitly apply framing

- Draw a cyan crop rectangle and dim only the pixels that a suggested crop would remove.
- Use the configured target for Adjustable images and the best safe symmetric real-pixel band for Constrained images.
- Keep preview and dismissal non-mutating; apply only through an explicit Apply to draft action.
- Preserve fixture save as the persistence boundary and provide a one-step Undo framing action for an applied draft crop.
- Never synthesize missing background or offer a crop from tentative, unstable, manual, clipped, or already-cropped support.

## Task 9: Make overlay visibility authoritative

- Hide SVG measurement and framing overlays with reflected attributes and an explicit CSS display rule rather than relying on HTML-only property behavior.
- Clear the framing overlay synchronously when preview is hidden, dismissed, applied, undone, or invalidated by another draft change.
- Verify an applied crop cannot leave its pre-apply cyan proposal painted over the rerendered output.

## Task 10: Permit explicitly confirmed manual framing boundaries

- Keep automatic edge detection advisory for slabs and other unreliable subjects.
- Let the operator explicitly confirm four manually placed output handles as the item boundary for background assessment.
- Recompute real-pixel framing support from the rendered image and confirmed manual boundary without changing adjustment values.
- Invalidate the confirmation whenever the adjustment or manual boundary changes, requiring deliberate reassessment.

## Task 11: Distinguish thin-border recentering from target framing

- Do not claim that a crop can achieve the framing target when the maximum symmetric real-pixel band is below the 3% lower trial bound.
- Still offer explicitly labeled recentering when opposite sides are materially imbalanced, while preserving the limiting thin border.
- Carry a confirmed manual boundary through the crop transform and automatically reassess the rendered result.
- Keep Undo framing available after applying the crop.

## Completion Evidence

- The final release workbench passed 8 contract tests, strict Clippy, Rust
  formatting, JavaScript syntax validation, and repository diff checks.
- Operator UAT covered automatic adjustment, output measurement, framing
  classifications, crop preview/apply/undo, overlay clearing, manual slab
  boundaries, constrained framing, and thin-border recentering.
- After the final recentering build was relaunched on `127.0.0.1:8788`, the
  operator accepted the experiment on 2026-10-18. The pause handoff mistakenly
  left this accepted checkpoint pending; resumption reconciled that stale state.
