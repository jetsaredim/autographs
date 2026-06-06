---
phase: 05-static-runtime-migration-foundation
plan: 01
subsystem: static-publishing-contract
tags: [rust, serde, static-artifacts, privacy, fixtures]
requires:
  - phase: 04-public-showcase-and-hardening
    provides: public gallery privacy baseline and hardened public edge
provides:
  - Versioned Rust DTOs for public collection, facets, item details, media variants, and publish manifests
  - Fixture-backed split static artifact generation with privacy regression scans
  - Documented schemaVersion 1 artifact and media path contract
affects: [05-02-controller-auth, 05-04-static-publisher, 05-06-runtime-cutover]
tech-stack:
  added: [Rust, serde, serde_json, time, uuid]
  patterns: [versioned public DTOs, split collection plus per-item JSON, deterministic public media paths]
key-files:
  created:
    - controller/Cargo.toml
    - controller/src/contracts.rs
    - controller/src/publisher.rs
    - controller/tests/static_contract.rs
    - docs/static-artifact-contract.md
  modified:
    - .gitignore
key-decisions:
  - "Use split collection.json plus per-item detail JSON for Phase 5 incremental publishing even though the monolithic fixture JSON is smaller."
  - "Expose only deterministic /media/{item-slug}/{image-slug}-{variant}.{ext} derivative paths in public artifacts."
patterns-established:
  - "Public static DTOs are serialized from explicit Rust structs rather than source-of-truth records."
  - "Generated public artifact tests scan rendered JSON, HTML, manifests, and paths for private identifiers."
requirements-completed: [STATIC-01, STATIC-02, STATIC-03, STATIC-07]
duration: 7 min
completed: 2026-06-01
---

# Phase 05 Plan 01: Static Artifact Contract Summary

**Versioned Rust public DTOs, 500-item fixture profiling, split static artifact generation, and privacy regression scans**

## Performance

- **Duration:** 7 min
- **Completed:** 2026-06-01T16:27:40Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- Added the Rust controller crate and schemaVersion 1 public artifact DTOs.
- Profiled single, split, and hybrid JSON layouts with a 500-item multi-image fixture.
- Documented and tested the public artifact privacy boundary and deterministic media paths.

## Task Commits

1. **Task 1: Add Rust controller workspace skeleton for contract tests** - `e5c7d0d`
2. **Task 2: Generate and profile static public artifact shapes** - `c520e99`
3. **Task 3: Document static artifact contract and privacy rules** - `f786e6a`

## Files Created/Modified

- `controller/src/contracts.rs` - Versioned public DTO and publish manifest structs.
- `controller/src/publisher.rs` - Fixture generation, shape profiling, and split artifact output.
- `controller/tests/static_contract.rs` - Artifact shape, path, and privacy regression coverage.
- `docs/static-artifact-contract.md` - Human-readable public artifact contract.

## Decisions Made

- Retained split per-item JSON despite the smaller single-file profile because Phase 5 needs explicit incremental item publishing.
- Used public slugs and derivative variant names instead of private image UUIDs or original filenames.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Ignored Rust target build output**
- **Found during:** Task 1
- **Issue:** The new Rust crate produces `controller/target/`, which would remain noisy untracked state.
- **Fix:** Added `controller/target/` to `.gitignore`.
- **Files modified:** `.gitignore`
- **Verification:** `git status --short` excludes Rust build output.
- **Committed in:** `e5c7d0d`

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Standard Rust repository hygiene only; no scope expansion.

## Issues Encountered

- Cargo dependency download required network access once; the locked dependencies then built locally.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- The Rust controller contract crate is ready for `05-02` health, configuration, and single-admin access controls.
- Later publisher work can extend the tested artifact generator into candidate release validation and promotion.

## Self-Check: PASSED

- `cargo test --manifest-path controller/Cargo.toml`
- `cargo test --manifest-path controller/Cargo.toml static_contract -- --nocapture`
- `rg -n "schemaVersion|collection.json|facets.json|manifest|/media/" docs/static-artifact-contract.md`

---
*Phase: 05-static-runtime-migration-foundation*
*Completed: 2026-06-01*
