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

## Round 3: production deployment freeze (`CR-03`)

**Finding lineage:** `CR-03` is a sibling deployment-consumer miss in the `CR-02` candidate-visibility family. The local pointer-first publisher and Caddy tests passed, but the actual Release workflow omitted the freeze variable and would clear a manual gate. The third-review checkpoint and revised assumptions, implementation plan, independent plan review, and resume evidence are recorded in `08-07-PR-294-REVIEW.md` and `08-PR-294-CONVERGENCE.md` before this coder work.

**Applied contract:** The production workflow runs a frozen preflight before full deploy, passes `autographs_deploy_publish_frozen=true` explicitly, verifies origin readiness, and invokes a separate readiness-backed unfreeze before release completion. On an existing VM, preflight requires the old live gate, decouples Caddy from the controller while static content remains served, stops the old controller, verifies unit/container/PID exit, and audits shared media before new Caddy routing. Unproven bytes are quarantined outside the served volume; ambiguous fresh or retained URLs stop the cutover. The full role rechecks the process barrier, starts and verifies the reviewed controller before new Caddy restart, and preserves the gate and recovery record across failures. Controller-only rollback refuses an image digest lacking pointer-first approval. The origin probes use loopback with proxy bypass and a unique checked disposable marker path, so the probe cannot remove an existing emergency marker.

**Files:** `.github/workflows/deploy.yml`, `.github/workflows/ci.yml`, `controller/tests/caddy_static_routes.rs`, `deploy/ansible/playbooks/controller-rollback.yml`, `deploy/ansible/playbooks/publish-freeze-render-test.yml`, `deploy/ansible/playbooks/publish-freeze-deploy-test.yml`, `deploy/ansible/playbooks/publish-freeze-preflight.yml`, `deploy/ansible/playbooks/publish-unfreeze.yml`, `deploy/ansible/roles/autographs_deploy/defaults/main.yml`, `handlers/main.yml`, `tasks/main.yml`, `tasks/publish_freeze.yml`, `templates/autographs-caddy.container.j2`, `files/publish_freeze_audit.py`, `deploy/ansible/tests/test_publish_freeze_deploy.py`, `docs/deployment-runbook.md`, and `docs/static-runtime-runbook.md`.

**Local verification:** Full Cargo test suite passed; focused Caddy tests passed (9). The six deployment contract unit tests passed, including the actual workflow invocation, queued process-exit barrier, candidate-only quarantine, and retained-manifest rejection. The disposable Ansible deploy fixture, deploy/preflight/unfreeze/rollback syntax checks, production-profile `ansible-lint` (zero findings), containerized Caddy public/preview HTTP matrix, and `git diff --check` passed. The coder also ran strict Clippy, production feature check, and formatting checks successfully before its session ended. No deployed OCI/Oracle/Cloudflare proof was performed.

**Live checkpoints:** The current VM has no automatic Oracle or archived promotion evidence export. If a shared path has ambiguous committed history, preflight halts and Wave 8 must resolve its provenance before retry. A fresh VM still requires a valid seeded `current` release to satisfy the existing static cutover gate. The guarded production rollout, credentialed media smoke, and Cloudflare cache verification remain Wave 8 work.

## Round 4: durable publication history and marker-aware audits (`CR-04`, `WR-03`)

**Commit:** `3cdd0d5` (`fix(08): preserve committed media history across deployment audits`)

**Finding lineage:** `CR-04` is the later-deployment consumer missed by `CR-01`: the old audit quarantined A after B became current. `WR-03` is the preflight consumer missed by `WR-02`: it rejected an active media file intentionally removed under an exact durable marker. Both are recurring families. The renewed round-4 assumptions, §5 contract revision, independent plan review, and `ready_to_resume` evidence preceded this coherent publication-history change.

**Applied contract:** The publisher writes private versioned, fsynced promotion intent before atomic `current` replacement, then finalizes full-hash release receipts and last-active timestamps in history. Reconciliation uses pointer state to discard an uncommitted intent or complete a committed transition; postpointer finalization failure keeps the active release with a warning and blocks later publish/prune until recovery. Shared cleanup uses committed receipts and last-active time rather than mtime or unproven retained manifests. An unrecorded direct pointer repoint fails closed. A protected Python rollback helper journals a verified committed target, preserves exact markers, and repairs only verified unmarked shared bytes.

The deployment audit has a separate read-only first-cutover inventory before and after verified old-process exit. It imports only independently reviewed archived promotion/manifest evidence; it never treats a hand-entered success-ID list as proof. Unarchived pruned URLs remain untouched under the old frozen current-only route until the conservative `T_exit + 86,400 + 60 + 300` bound, provided boot, controller invocation, pointer, gate, and synchronized UTC remain stable. A retained path without proof remains blocked. Only then does the mutation pass quarantine unprotected bytes and write versioned history and migration marker. Subsequent preflight and role reaudit verify history, current/retained manifests and hashes, exact markers, and old shared retention before any Caddy route change. Marked active/retained bytes may be absent; unmarked committed corruption is fatal. Successful audits alone replace the freeze record, which binds pointer, manifest/history digests, and marker inventory. The full role checks refreshed identity and newly marked dual-origin 404; unfreeze revalidates committed bytes and record digests, probes unmarked active and retained old URLs on both origins, and keeps the gate until readiness passes.

**Files:** `controller/src/publisher.rs`, `controller/tests/publisher.rs`, `deploy/ansible/roles/autographs_deploy/files/publish_freeze_audit.py`, new `static_release_rollback.py`, `deploy/ansible/tests/test_publish_freeze_deploy.py`, `deploy/ansible/playbooks/publish-freeze-preflight.yml`, `deploy/ansible/playbooks/publish-unfreeze.yml`, `deploy/ansible/roles/autographs_deploy/tasks/main.yml`, `scripts/verify-caddy-media-origin.sh`, `docs/cdn-cache-contract.md`, `docs/static-runtime-runbook.md`, and `docs/deployment-runbook.md`. The existing production workflow already invokes preflight → frozen full deploy → readiness → explicit unfreeze; the Python regression parses that call order.

**Verification in main checkout (`workflow.use_worktrees=false`):** Full `cargo test --manifest-path controller/Cargo.toml` passed. Focused suites passed: publisher 49, Caddy 9, static contract 4, publisher journal fault unit test, Python deployment audit 14. The Rust live persistence smoke compiled with `--no-run`; production persistence `cargo check`, strict all-target/all-feature Clippy, `cargo fmt --check`, `git diff --check`, Ansible deploy/preflight/unfreeze/rollback syntax checks, production-profile `ansible-lint` (zero findings), and the production-shaped Ansible fixture passed. The containerized real Caddy HTTP matrix passed public and preview origin misses for precommit B 404, postcommit B 200, A after two history-backed audits 200, journaled rollback A/B 200, and marked active/retained deletion 404. These are local proofs; the credentialed deployed smoke was not run.

**Wave 8 prerequisites and limits:** The deployment path has no direct verified Oracle job-row adapter. Initial migration therefore requires a trustworthy preexisting archived promotion receipt and manifest for protected retained/pruned URLs, or the documented non-destructive wait and retained-release resolution. The helper refuses an unverifiable success-ID export. Production must prove the manual gate, old process exit, trusted UTC and archive source, static volume identity, Caddy marker readback, public/preview 404 for legacy marked paths, seeded current release on a fresh VM, later A→B deployment and marked active deletion, credentialed Oracle/OCI smoke, and Cloudflare behavior. No live deployment, emergency marker, purge, or release merge occurred in this coder round.

_Fixer: GSD code fixer; review round 4. This report is left uncommitted for the orchestrator._

## Round 5: state-aware origin readiness (`WR-04`) — checkpoint

**Implementation commit:** `c38276b` (`fix(08): validate state-aware origin probe before unfreeze`). This report remains uncommitted for the parent. No push or PR resolution comment was made by the coder.

**Lineage and shared invariant:** `WR-04` is the explicit-unfreeze sibling miss of `WR-02`/`WR-03` in the `emergency_media_takedown_retention` family. An exact-marked active deletion may leave no anonymously available image. A healthy frozen release must still prove a current document 200, every exact marker 404, and protected retained URL reachability before reopening publishing. The round-5 reviewed assumptions, §6 implementation revision, independent plan review, and `ready_to_resume` convergence evidence at `2feb3cb` authorized this round.

**Applied change:** The audit now selects a versioned `{kind, path}` probe from the observed current release after checking every unmarked current WebP's path, type, byte size, filename fingerprint, and committed full hash. It selects a verified media path when one exists and `manifest.json` for an empty or all-marked release; corrupt unmarked bytes cannot fall back to a document. The durable record and read-only report carry the probe, pointer/manifest/history identity, marker digest/paths, and retained probe. Preflight selects through the same helper before old-process stop, consumes the final audited record after exit, and requires public/preview 200 plus marker readback and dual-origin 404. The full role consumes the refreshed record and checks the selected 200 and all marker 404s after Caddy restart. Verify-only and mutating unfreeze compare a fresh read-only probe state to the record, check both origins and persistent markers, keep the disposable marker through mutating Caddy restart, and recheck selected/retained media and marker denial before deleting the freeze record. Failure cleanup removes only disposable fixture paths; post-restart failure restores the frozen Caddyfile and leaves the durable record. The production path retains systemd restart and root ownership; the local fixture adapter is guarded to a disposable local domain/path.

**Regression evidence in the main checkout:** `python3 -m unittest deploy/ansible/tests/test_publish_freeze_deploy.py` passed 19 tests. The new real-Caddy fixture executes the actual `publish-unfreeze.yml` in verify-only and mutating modes for a sole-image exact-marked/deleted release over local HTTPS and preview HTTP: document 200, marked URL 404, frozen POST 503/no-store, post-unfreeze POST 401, authenticated status, gate removal, and record removal after postchecks. It also injects a post-restart origin failure through the test Podman adapter and proves the actual rescue tasks restore frozen POST 503 and preserve the record. Selector regressions cover empty, mixed marked/unmarked, corruption/missing unmarked bytes, malformed marker, read-only pending-intent refusal, missing retained shared bytes, and journaled rollback to an all-marked release. `bash scripts/verify-caddy-media-origin.sh` passed against real containerized Caddy for media and document 200 choices plus exact marker 404 before/after frozen and unfrozen reloads.

Focused Cargo suites passed (49 publisher, 9 Caddy routes, 4 static contract); `cargo fmt --check`, production-persistence `cargo check`, strict all-target/all-feature Clippy, live-persistence smoke `--no-run`, Ansible deploy/preflight/unfreeze/test/rollback syntax, production-profile `ansible-lint` with zero findings, shell syntax, and `git diff --check` passed. The existing CI Ansible fixture now executes the Docker-backed unfreeze regression and checks Docker availability.

**Completed production-shaped matrix:** The follow-up disposable fixture executes the actual `publish-freeze-preflight.yml`, the tagged production role reaudit and frozen Caddy restart tasks, and both verify-only and mutating `publish-unfreeze.yml` against real local Caddy HTTP origins. The sole marked/deleted image, empty release, mixed marked/unmarked release, and journaled rollback to all-marked release each complete the preflight → role reaudit → unfreeze sequence. It asserts document or verified media 200, each exact marker 404 on public and preview origins, frozen POST 503/no-store, post-unfreeze controller authentication, and record removal only after postchecks. Injected unmarked missing bytes, retained-history failure, marker drift, and post-restart origin failure leave or restore the frozen gate and record. The actual preflight execution exposed and fixed an undefined `freeze_current_media` reference. The complete expanded real-Caddy test passed in 399 seconds locally; Ansible syntax and production lint passed, and CI syntax coverage now includes the role reaudit fixture. The fixture substitutes only host-specific controller, Podman, systemd, and health operations; live VM behavior remains a Wave 8 gate.

**Wave 8 live prerequisites:** The guarded real-VM preflight and full-role reaudit, sole-image marked/deleted release through both production unfreeze invocations, dual-origin document 200 and marker 404 after restart, authenticated status, later A→B retained URL, credentialed Oracle/OCI smoke, and Cloudflare exact purge/cache behavior remain live checkpoints. No live deployment, takedown, purge, or release merge occurred.
