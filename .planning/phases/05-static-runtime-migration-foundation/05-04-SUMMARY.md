---
phase: 05-static-runtime-migration-foundation
plan: 04
subsystem: static-publisher
tags: [rust, axum, static-site, webp, privacy, atomic-promotion]
requires:
  - phase: 05-03
    provides: private catalog repository and original-media boundaries
provides:
  - Local-mode static release publisher with full and incremental entry points
  - Sanitized thumbnail/detail WebP derivatives with public-safe paths
  - Candidate validation, atomic current-pointer promotion, and failed-candidate retention
  - Authenticated publish trigger and redacted status APIs
affects: [05-05-static-admin, 05-06-runtime-cutover, 05-07-live-proof]
tech-stack:
  added: [image]
  patterns: [candidate-before-promotion, source-aware privacy scan, deterministic public slugs, conservative incremental rebuild]
key-files:
  created:
    - controller/src/derivatives.rs
    - controller/tests/publisher.rs
  modified:
    - controller/src/publisher.rs
    - controller/src/routes.rs
    - docs/static-runtime-runbook.md
key-decisions:
  - "Generate public media as re-encoded WebP thumbnail/detail derivatives under media/{item_slug}/{image_slug}-{variant}.webp."
  - "Promote only validated candidates by atomically replacing the current symlink."
  - "Seed incremental candidates from current, then conservatively rebuild the union of impacted surfaces until durable change events exist."
patterns-established:
  - "Publisher validation checks required pages, JSON parsing, referenced derivatives, manifest inventory, WebP type, byte accounting, generic denied terms, and private source identifiers."
  - "Publish status returns release ID, artifact count, byte total, timestamps, and redacted failure text."
requirements-completed: [STATIC-02, STATIC-03, STATIC-04, STATIC-07]
duration: 35m
completed: 2026-06-02
---

# Phase 05 Plan 04: Static Publisher Summary

**Validated local-mode static releases with sanitized WebP derivatives and atomic promotion**

## Performance

- **Duration:** 35m
- **Tasks:** 3
- **Files modified:** 14

## Accomplishments

- Added complete static candidate generation for landing, collection, detail, JSON, facet, manifest, browse-filter, and derivative artifacts.
- Added thumbnail/detail WebP re-encoding, deterministic slugs, source-aware privacy scans, complete manifest inventory checks, and atomic `current` pointer promotion.
- Added authenticated full/incremental publish routes and redacted status reporting with timestamps, artifact counts, byte totals, and latest-failed-candidate retention.

## Deviations from Plan

### Conservative Incremental Rebuild

- **Issue:** The repository boundary does not yet persist a durable change journal, so the publisher cannot reliably distinguish every metadata, tag, publication, replacement, or deletion event between releases.
- **Implementation:** Incremental publish copies `current`, applies the explicit impact map as a correctness-first union, removes stale generated output, rebuilds the public surface, validates it, and then promotes it.
- **Follow-up:** Durable event tracking can narrow regeneration later without changing the candidate-validation or promotion contract.

## Verification

- `cargo test --manifest-path controller/Cargo.toml publisher -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml -- --nocapture`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`
- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `git diff --check`

## Next Phase Readiness

- `05-05` can add the minimal static admin shell against the authenticated seed and publish endpoints.
- `05-06` still owns production repository wiring, controller deployment, and Caddy candidate validation.

---
*Phase: 05-static-runtime-migration-foundation*
*Completed: 2026-06-02*
