---
spike: 011
idea: shared-image-processing
name: local-image-workbench
type: standard
validates: "Given representative private images, can the shared production image-processing crate support exact adjustment fixtures and an honest operator fallback without weakening detector abstention or copying production behavior?"
verdict: VALIDATED
related: [008, 009a, 009b, 010]
tags: [rust, image-processing, validation, fixtures, privacy]
---

# Spike 011: Local Image Workbench

## Validation Question

Could the shared production image-processing crate support an experiential,
fixture-driven editor for representative private images while keeping originals
immutable, generated output outside Git, and adjustment JSON identical to the
production contract?

## Boundaries

- Private originals and generated diagnostics remained outside the repository.
- Source images were read-only; only adjustment fixtures were persisted in a
  separately selected external workspace.
- Detector proposals and rendered previews exercised
  `autographs-image-processing` directly rather than copying detector or
  transformation logic.
- The validation covered a single local operator. It did not establish a
  multi-user editing, authorization, conflict-resolution, or durability
  contract; those concerns would block productizing this workflow for multiple
  users.

## Evidence

- Loaded the full 116-image private corpus without copying originals into the
  repository or workspace.
- Verified identity rendering, production detector proposals, exact adjustment
  fixture save/load, and unchanged source hashes against representative images.
- Exercised automatic and manual alignment, framing assessment, crop and
  recentering previews, draft application, undo, constrained-border cases, and
  the slab fallback.
- Confirmed that low-confidence boundary candidates remain diagnostic rather
  than being promoted into production auto-correction proposals.
- The operator accepted the experiment after the final representative-image
  pass on 2026-10-18.

## Results

**VALIDATED.** The shared production crate supports exact adjustment fixtures,
automatic proposals when evidence is reliable, and an explicit manual operator
fallback when the conservative detector abstains. The experiment also confirmed
that cropping cannot manufacture missing photographed background and that
uncertain geometry must not be presented as reliable correction.

The executable validation harness was intentionally removed after acceptance.
It was disposable experiment infrastructure, not an application runtime. The
surviving production behavior and regression coverage remain in
`controller/image-processing/` and `controller/tests/image_adjustments.rs`.
