---
status: resolved
trigger: "Workbench image 1 applies coordinate adjustments but gets stuck on the rotation pass"
created: 2026-10-17
updated: 2026-10-18
---

# Debug Session: Workbench Rotation Backoff

## Symptoms

- Expected behavior: auto-alignment should retain improving coordinate and rotation changes until the rendered item is within tolerance.
- Actual behavior: image 1 reaches a rotation suggestion, rejects the full rotation step, rolls back, and presents the same suggestion again.
- Reproduction: load `?image=1` and run auto-alignment.

## Evidence

- timestamp: 2026-10-17
  observation: At identity, the reliable fitted-edge offsets are `+0.749`, `+0.131`, `+0.146`, and `+0.118` degrees. Their mean proposes a `-0.30` degree rotation and their RMS score is `0.392` degrees.
- timestamp: 2026-10-17
  observation: Rendering the full `-0.30` degree proposal changes the detected rectangle and worsens the RMS score to `0.565` degrees, so the existing loop correctly rolls it back but has no smaller-step recovery.
- timestamp: 2026-10-17
  observation: Smaller candidates improve the same score: `-0.05` degrees scores `0.262` and `-0.10` degrees scores `0.243`.
- timestamp: 2026-10-17
  observation: When a later proposal worsens alignment, rollback re-renders the retained adjustment. That render recomputes the rejected proposal and the advice panel presents its apply button again even though the status correctly says it was rejected.

## Current Focus

- hypothesis: Rollback and advice recomputation are individually correct, but the UI lacks a durable record that the proposal at the restored adjustment was already rejected.
- test: Bind a stopped-loop suppression message to the retained adjustment signature, remove the action button for that state, and clear suppression whenever the operator changes a source control.
- expecting: The retained preview remains visible with an explicit non-actionable explanation, and a manual control change enables a fresh recommendation.
- next_action: Resolved. Preserve bounded backoff and rejected-proposal suppression in the production integration slice.

## Resolution

- root_cause: Rotation suggestions were all-or-nothing. A full mean-tilt correction could change detector boundary selection even when a smaller rotation improved the preview.
- fix: Added rotation backoff candidates and raised the fine rigid-rotation threshold from 0.10 to 0.15 degrees. A stopped automatic loop now suppresses its rejected recommendation at the retained adjustment, explains why it stopped without an apply button, and permits reassessment after a source-control change.
- verification: JavaScript syntax, formatting, diff checks, release clippy, and the final 8 release workbench tests pass. The release workbench served the adjustment-bound suppression path and the explicit `rejected proposal is no longer actionable` state on the sole listener at `127.0.0.1:8788`; the operator subsequently accepted the completed experiment on 2026-10-18.
- files_changed: `.planning/spikes/011-local-image-workbench/workbench/static/app.js`, `.planning/spikes/011-local-image-workbench/workbench/src/main.rs`
