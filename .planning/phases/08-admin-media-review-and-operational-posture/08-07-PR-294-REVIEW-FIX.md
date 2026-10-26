---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-26T15:02:59Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-07-PR-294-REVIEW.md
iteration: 1
findings_in_scope: 2
finding_families: 2
fixed: 2
skipped: 0
reassessment_required: false
status: all_fixed
---

# Phase 8 Wave 7: PR #294 Review Fix

## Invariant inventory

- `CR-01` (`fingerprinted_media_release_lifetime`, independent): the publisher writes each candidate's media, validates its manifest, changes `current`, then prunes releases. Caddy previously resolved `/media/*` only under `current`. Public JSON/HTML may stay fresh for 60 seconds; media may stay fresh for 24 hours. The live smoke and unpublish test previously expected immediate media 404. These producers, serving routes, promotion and rollback paths, cleanup, smoke, tests, and cache/runbook contracts were inspected before the fix.
- `WR-01` (`derivative_cache_metadata_dimensions`, independent): `DerivativeCache::write` records generated dimensions and byte digest; `read` verified the digest but returned unverified metadata dimensions to `build_public_items`, which wrote public JSON. Candidate validation checks WebP type and filename fingerprint. Both variants and corrupt-cache recovery tests were inspected before the fix.

## Fixed issues

### CR-01: Old fingerprinted media stops resolving while cached documents reference it

**Commit:** `e740082` (`fix(08): CR-01 retain public media across promotion and rollback`)

**Applied fix:** Caddy now serves `/media/*` from a shared static namespace and falls back to `current/media` for an already active release before migration. After candidate validation, the publisher migrates active media and stages candidate media before switching `current`; matching URLs must have identical verified WebP bytes and fingerprints. Failed promotion removes newly exposed candidate media. Cleanup retains media referenced by retained releases or touched within 24 hours and refreshes media before pruning a demoted release, including after operator rollback. Candidate manifests remain limited to the candidate's published files. Normal unpublish removes documents but keeps known media URLs for the bounded window; emergency takedown requires removal of shared and retained origin copies plus exact CDN purge.

**Files:** `controller/src/publisher.rs`, `controller/tests/publisher.rs`, `controller/tests/caddy_static_routes.rs`, `controller/tests/live_static_publish_smoke.rs`, `deploy/ansible/roles/autographs_deploy/files/Caddyfile`, `docs/cdn-cache-contract.md`, `docs/static-artifact-contract.md`, `docs/static-runtime-runbook.md`.

**Ownership expansion:** The credentialed live smoke was directly necessary because it asserted the old immediate-404 behavior after unpublish; its page/JSON checks remain 404 while recent media now checks 200.

**Behavioral proof:** Local publisher tests cover first-publish migration, old/new URL resolution after correction and manual rollback, aggressive release pruning after rollback, ordinary unpublish retention and expiry, and failed promotion cleanup. The Caddy contract test covers both public and private preview media routes. Actual deployed Caddy and Cloudflare cache-miss behavior remains for the credentialed Wave 8 live verification.

### WR-01: Cached dimensions can diverge from verified WebP bytes

**Commit:** `2eb17d4` (`fix(08): WR-01 verify cached WebP dimensions`)

**Applied fix:** Cache reuse decodes the verified WebP bytes and compares decoded width and height with metadata. A mismatch regenerates from the private original. Both thumbnail and detail corrupt-dimension regression cases pass.

**Files:** `controller/src/publisher.rs`, `controller/tests/publisher.rs`.

## Verification

All local gates ran in the main checkout because `.planning/config.json` sets `workflow.use_worktrees=false`.

- `cargo test --manifest-path controller/Cargo.toml`: passed after the final release-pruning and private-staging refinements; orchestrator reran the full suite before pushing the review fixes.
- `cargo test --manifest-path controller/Cargo.toml --test publisher --test caddy_static_routes`: passed after the refinements (46 publisher, 8 Caddy contract tests).
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_static_publish_smoke --no-run`: passed; live smoke was not run without Oracle, OCI, deployed Caddy, and credentials.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`: passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` and `git diff --check`: passed.

The installed Caddy binary and a live Cloudflare edge were unavailable locally. The origin route uses Caddy's documented `handle` and `try_files` behavior; confirm it against the deployed preview and an uncached edge request before CDN enablement.

_Fixer: GSD code fixer; review round 1._
