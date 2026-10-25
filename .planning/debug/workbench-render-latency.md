---
status: resolved
trigger: "something seems to be taking a while to render the image"
created: 2026-10-17
updated: 2026-10-18
---

# Debug Session: Workbench Render Latency

## Symptoms

- Expected behavior: changing a rotation or vertex value should produce a newly assessed preview quickly enough for iterative adjustment.
- Actual behavior: the image appears to take a while to render after an adjustment.
- Error messages: none reported.
- Timeline: observed after adding per-render alignment assessment and advisory values to the local workbench.
- Reproduction: change an individual vertex or rotation value and wait for the adjusted preview and assessment.

## Current Focus

- hypothesis: confirmed — the workbench was relaunched as an unoptimized Rust debug executable, making the full-resolution transform, encoding, and boundary-analysis loops unsuitable for an interactive adjustment cycle.
- test: compare the same saved fixture against debug and release builds without changing resolution or analysis behavior.
- expecting: release mode materially reduces latency while returning the same full-resolution preview and assessment headers.
- next_action: Resolved. Keep release-mode execution and the sole-instance restart contract in future workbench use.

## Evidence

- timestamp: 2026-10-17
  checked: three live POST requests for saved image 7 against the debug-mode `/render` endpoint
  found: requests took 26.88s, 26.92s, and 27.34s and each returned a 5,492,168-byte preview
  implication: the reported delay is deterministic server-side work rather than advisory-panel JavaScript or a transient browser stall
- timestamp: 2026-10-17
  checked: the identical fixture against the optimized release executable
  found: the request completed in 1.22s and returned the same 5,492,168-byte full-resolution preview
  implication: compiler optimization removes about 95% of observed latency without weakening preview fidelity or alignment assessment
- timestamp: 2026-10-17
  checked: revised suggestion interaction and workbench verification
  found: one explicit action now runs up to eight render-and-reassess passes, retains only improving reliable results, and stops at tolerance, repetition, worsening score, measurement precision, or the iteration cap; internal passes do not replace the visible preview or diagnostics, and the best final result is published once; 6 release-mode workbench tests, strict Clippy, JavaScript syntax, formatting, and diff checks pass
  implication: the operator no longer has to chase changing coordinates, manually advance converging passes, or watch the page visibly refresh between internal passes
- timestamp: 2026-10-17
  checked: operator reproduction with a coarse-only boundary followed by manual corner and rotation correction
  found: the initial reliable quadrilateral had no four-edge refinement and therefore produced no suggestion; manual corners unlocked a rotation proposal, but automatic validation compared the mean-tilt proposal against a worst-edge composite score and could roll it back, while manual rotation allowed later perspective passes to finish
  implication: automatic alignment needed a coarse bootstrap and stage-specific objectives; the transform itself was sound
- timestamp: 2026-10-17
  checked: coarse-to-fine implementation against a live reliable response whose `x-workbench-boundary-edges` value is empty
  found: the workbench now derives conservative coarse rotation/perspective suggestions from the returned corners, scores rotation by RMS rigid tilt, scores perspective by residual tilt plus opposing-edge parallelism, and retries refined measurement after each accepted render; release tests, strict Clippy, JavaScript syntax, formatting, and diff checks pass
  implication: the automatic path can now reproduce the manual coarse-corners → rotation → fine-perspective progression without treating missing refinement as infinite misalignment

## Eliminated

## Resolution

- root_cause: "The local workbench was running an unoptimized debug executable; its full-resolution transform and analysis took about 27 seconds. The first automatic loop also required four refined edges before suggesting anything and used one worst-edge composite score for both rotation and perspective, so coarse-only images could not bootstrap and useful rotation could be rolled back."
- fix: "Run the optimized release executable; use a bounded loop that hides intermediate passes; bootstrap from reliable coarse corners; and validate rotation and perspective with matching stage-specific objectives while retrying fine measurement after each pass."
- verification: "Live renders reduced to approximately 1.2–1.34s; a live reliable/empty-edges response enters the coarse suggestion path; the final 8 release tests, strict Clippy, JavaScript syntax, formatting, and diff checks pass. The operator used and accepted the release-mode experiment on 2026-10-18."
- files_changed: [".planning/spikes/011-local-image-workbench/workbench/static/app.js", ".planning/spikes/011-local-image-workbench/workbench/static/app.css", ".planning/spikes/011-local-image-workbench/workbench/static/index.html", ".planning/spikes/011-local-image-workbench/workbench/src/main.rs"]
