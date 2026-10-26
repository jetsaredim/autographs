---
phase: 08-admin-media-review-and-operational-posture
plan: "07"
subsystem: static-publisher
tags: [rust, image-adjustments, derivative-cache, static-publish, cloudflare]
requires:
  - phase: 08-06
    provides: revision-aware private image adjustment API
provides:
  - Adjustment-aware public thumbnail and detail publishing
  - Schema v2 derivative cache key and payload verification
  - Gated live adjusted-media publish procedure for Wave 8
affects: [08-08, MEDIA-05, MEDIA-06, OPS-02]
actuals:
  tokens: 8401
  tasks: 3
  commits: 3
commits: 3
plan_head_before: 8d986ab0573ab9514c610217b7e34fc025bfec54
plan_head_after: d943e0bfb0b5758dbded4b717247e16597b5ffec
tech-stack:
  added: []
  patterns: [canonical adjustment cache keys, final-byte WebP fingerprints, source-backed corrupt-cache regeneration]
key-files:
  created: []
  modified:
    - controller/src/publisher.rs
    - controller/tests/publisher.rs
    - controller/tests/static_contract.rs
    - controller/tests/live_static_publish_smoke.rs
    - docs/static-artifact-contract.md
    - docs/static-runtime-runbook.md
key-decisions:
  - "Bind source SHA-256, transform version, canonical adjustment, and variant to internal cache entries; verify key and WebP digests before reuse."
  - "Keep public URLs fingerprinted from final adjusted WebP bytes and publish updated references through the normal release boundary."
requirements-completed: [MEDIA-05, MEDIA-06]
requirements-advanced: [OPS-02]
duration: 40min
completed: 2026-10-26
status: complete
---

# Phase 08 Plan 07: Adjusted Media Publishing Summary

Saved corrections now flow through normal static publishing to both public WebP variants. The final adjusted bytes determine new `/media/*` URLs and updated public references, so routine Cloudflare purges are unnecessary.

## Accomplishments

- Checksum-backed publishes reuse only schema v2 cache entries whose requested key and WebP payload digests verify; checksum-less originals still get read and byte-hashed on every publish.
- A saved crop, rotation, or perspective change invalidates both variant caches without changing the private original. V1, malformed, wrong-key, and payload-corrupt entries regenerate from the private source. An unreadable original leaves `current` unchanged.
- Actual adjusted public artifacts and schema contract tests exclude private adjustment data, image UUIDs, storage coordinates, original filenames, and unpublished items.
- The credential-gated live smoke now saves through the revision-aware API, checks private-before-public timing, validates both final-byte URLs and public references, and retains cleanup. The runbook provides the candidate and Wave 8 evidence procedure.

## Task Commits

1. Task 1: `be1b1ed` — adjusted publishing and verified cache with an intentionally red tracer regression before implementation.
2. Task 2: `d01b55d` — legacy/corrupt cache recovery, failure preservation, privacy tests, and artifact contract.
3. Task 3: `d943e0b` — revision-aware live smoke and operator proof procedure.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test publisher -- --nocapture` — 42 passed.
- `cargo test --manifest-path controller/Cargo.toml --test static_contract -- --nocapture` — 4 passed.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_static_publish_smoke --no-run` — passed.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_static_publish_smoke -- --ignored --nocapture` — passed the environment gate only; `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE` was unset, so this is not live Oracle, OCI, or Caddy evidence.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings` — blocked by the preexisting `oracle_wallet_dir` dead-code warning in `controller/src/config.rs:31`; this plan did not change that file. The same all-target check passed with only `-A dead-code` added. The orchestrator will resolve the strict gate before PR review.

## Deviations from Plan

- The TDD tracer's intentional RED assertion failure was observed before publisher edits. Repository instructions allow only `feat`, `fix`, `perf`, `revert`, `docs`, and `chore` commit types, so the RED regression and GREEN implementation share Task 1's `feat` commit rather than using the GSD template's invalid `test` type.
- Strict default-feature Clippy cannot pass until the preexisting unused configuration field is handled outside this plan's file scope. No unrelated source was changed.

## Known Stubs

None.

## Threat Flags

None beyond the plan's derivative-cache, private-to-public release, and live operator boundaries.

## Next Phase Readiness

Wave 8 can use the runbook's credentialed adjusted-media publish to collect real edge-cache evidence before CDN enablement. The local gate result must not be cited as a live pass. Resolve the existing strict Clippy warning before PR review.
OPS-02 remains pending until Wave 8 enables and verifies the production CDN.

## Self-Check: PASSED

All six declared implementation/documentation files and this summary exist. Task commits `be1b1ed`, `d01b55d`, and `d943e0b` are present on the Wave 7 branch.
