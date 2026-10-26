---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-26T20:42:59Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-07-PR-294-REVIEW.md
iteration: 2
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

**Applied fix in round 1:** Caddy serves `/media/*` from a shared static namespace and falls back to `current/media` for an already active release before migration. That implementation migrated active media and staged candidate media before switching `current`; failed promotion tried to remove newly exposed candidate media. Cleanup retained media referenced by retained releases or touched within 24 hours and refreshed media before pruning a demoted release, including after operator rollback. Candidate manifests remained limited to the candidate's published files. Round 2 supersedes candidate staging and emergency takedown behavior below.

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

## Round 2: commit boundary and durable emergency denial

**Commit:** `a9d1da4` (`fix(08): enforce committed media visibility and durable takedown`)

**Finding lineage:** `CR-02` is a `CR-01` fix regression: the stable shared namespace solved stale-document origin misses but exposed candidate-only URLs before the atomic `current` pointer change. `WR-02` is a sibling consumer miss: manual removal could leave a retained-release copy, block pruning, or permit copy refresh. Both findings share the invariant that only committed fingerprints can be served and an exact emergency denial must override every release, copy, cleanup, and rollback path. The round-2 implementation followed the reviewed assumptions and contract revision recorded in `08-PR-294-CONVERGENCE.md` as `ready_to_resume`.

### CR-02: Candidate media was exposed before publication committed

The publisher now migrates only the already active release before promotion. It validates candidate media and tombstones before the pointer change, then copies the new active release into shared storage after that change. Caddy's `current/media` fallback serves new URLs during the copy. A postcommit copy failure leaves the release active, preserves retained releases, and reports a safe cleanup warning for repair/retry. A controlled publisher callback proves candidate-only media absent immediately before a forced promotion error and immediately before a successful pointer switch. Integration tests cover failed promotion, correction, rollback, postcommit copy failure, and retry.

### WR-02: Emergency removal was not durable across retained copies and pruning

An exact empty regular marker under `.media-tombstones/media/...webp` now denies that URL before either Caddy shared/current lookup on both listeners. Candidate validation rejects marked URLs; release-to-shared refresh skips them before reading missing source bytes; cleanup removes marked shared copies and prunes releases despite intentionally removed bytes, while unrelated missing bytes remain errors. The runbook requires volume and mount identity checks, Caddy marker readback, dual-origin `404` proof, exact CDN purge, and indefinite marker retention. The Ansible publish-freeze switch and disposable render fixture keep the two publish POST routes gated during the revised Caddy/controller rollout without blocking status.

**Files:** `controller/src/publisher.rs`, `controller/tests/publisher.rs`, `controller/tests/caddy_static_routes.rs`, `deploy/ansible/roles/autographs_deploy/defaults/main.yml`, `deploy/ansible/roles/autographs_deploy/files/Caddyfile`, `deploy/ansible/roles/autographs_deploy/tasks/main.yml`, `deploy/ansible/roles/autographs_deploy/tasks/publish_freeze.yml`, `deploy/ansible/playbooks/publish-freeze-render-test.yml`, `scripts/verify-caddy-media-origin.sh`, `docs/cdn-cache-contract.md`, `docs/static-artifact-contract.md`, `docs/static-runtime-runbook.md`.

**New files:** The publish-freeze role task, disposable render playbook, and executable Caddy origin probe implement the reviewed rollout and actual HTTP proof; they are not unrelated scope additions.

### Round-2 verification

All local gates ran in the main checkout because `workflow.use_worktrees=false`:

- Full `cargo test --manifest-path controller/Cargo.toml`: passed.
- Focused `cargo test --manifest-path controller/Cargo.toml --test publisher --test caddy_static_routes --test static_contract --quiet`: passed (49 publisher, 9 Caddy, 4 static-contract tests).
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_static_publish_smoke --no-run`: passed; credentialed live smoke was not run locally.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`: passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check`, Ansible deploy syntax check, `ansible-lint deploy/ansible/` (production profile, zero warnings), `bash -n scripts/verify-caddy-media-origin.sh`, and `git diff --check`: passed.
- `bash scripts/verify-caddy-media-origin.sh`: passed against containerized Caddy, including actual public and preview HTTP responses before/after pointer switch, marked URLs with both serving copies and current fallback alone, marker privacy, and Ansible-rendered freeze/unfreeze behavior. Caddy adapted and validated both configurations.

**Remaining Wave 8 live proof:** Perform the guarded production rollout, deployed dual-route origin and Cloudflare cache-miss checks, credentialed Oracle/OCI smoke, rollback/retention proof, and separate explicitly approved disposable emergency marker plus exact CDN purge. No live deployment or takedown was performed in this coder round.

_Fixer: GSD code fixer; review round 2. This report is left uncommitted for the orchestrator._
