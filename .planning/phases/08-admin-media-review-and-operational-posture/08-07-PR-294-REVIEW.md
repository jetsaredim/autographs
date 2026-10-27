---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-27T09:39:17Z
depth: deep
review_round: 5
files_reviewed: 38
files_reviewed_list:
  - .github/workflows/ci.yml
  - .github/workflows/deploy.yml
  - .planning/phases/08-admin-media-review-and-operational-posture/08-07-PLAN.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-07-PR-294-REVIEW-FIX.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-07-SUMMARY.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-PR-294-ASSUMPTIONS.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-PR-294-CONTRACT-PLAN-REVIEW.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-PR-294-CONTRACT-REVISION.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-PR-294-CONVERGENCE.md
  - controller/src/config.rs
  - controller/src/publisher.rs
  - controller/tests/live_static_publish_smoke.rs
  - controller/tests/publisher.rs
  - controller/tests/caddy_static_routes.rs
  - controller/tests/static_contract.rs
  - docs/static-artifact-contract.md
  - docs/static-runtime-runbook.md
  - docs/cdn-cache-contract.md
  - controller/image-processing/src/image_adjustments.rs
  - controller/image-processing/src/derivatives.rs
  - controller/src/routes.rs
  - deploy/ansible/playbooks/deploy.yml
  - deploy/ansible/playbooks/controller-rollback.yml
  - deploy/ansible/playbooks/publish-freeze-deploy-test.yml
  - deploy/ansible/playbooks/publish-freeze-preflight.yml
  - deploy/ansible/playbooks/publish-freeze-render-test.yml
  - deploy/ansible/playbooks/publish-unfreeze.yml
  - deploy/ansible/roles/autographs_deploy/defaults/main.yml
  - deploy/ansible/roles/autographs_deploy/files/Caddyfile
  - deploy/ansible/roles/autographs_deploy/files/publish_freeze_audit.py
  - deploy/ansible/roles/autographs_deploy/files/static_release_rollback.py
  - deploy/ansible/roles/autographs_deploy/handlers/main.yml
  - deploy/ansible/roles/autographs_deploy/tasks/main.yml
  - deploy/ansible/roles/autographs_deploy/tasks/publish_freeze.yml
  - deploy/ansible/roles/autographs_deploy/templates/autographs-caddy.container.j2
  - deploy/ansible/tests/test_publish_freeze_deploy.py
  - docs/deployment-runbook.md
  - scripts/verify-caddy-media-origin.sh
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
convergence:
  status: reassessment_required
  recurring_critical_warning: 1
  scope_expansion: false
  requires_reassessment: true
  assumptions_evidence: null
  plan_evidence: null
  plan_review_evidence: null
---

# Phase 8 Wave 7: PR #294 Code Review

**Reviewed:** 2026-10-26T14:51:07Z  
**Depth:** deep  
**Status:** issues_found

## Summary

The adjusted generation, cache-key change, source reread on damaged cache, candidate validation, private preview, and credential-gated smoke were traced across their producers and consumers. The source-only review found two independent defects. No credentialed live Oracle, OCI, Caddy, or Cloudflare run was performed.

## Critical Issues

### CR-01: Old fingerprinted media stops resolving while cached documents reference it

**File:** `docs/static-artifact-contract.md:111`  
**Finding family:** `fingerprinted_media_release_lifetime`  
**Lineage:** `independent`  
**Shared invariant:** Every public media URL referenced by a still-fresh public document remains fetchable during promotion and rollback.  
**Consumers inspected:** Candidate media writing and promotion in `controller/src/publisher.rs`; Caddy current-root routing and cache headers; CDN contract; local publisher tests and live smoke.  
**Evidence level:** `source_only`  
**PR comment:** https://github.com/jetsaredim/autographs/pull/294#discussion_r4378674255  
**Issue:** The publisher builds each candidate without prior media, and Caddy serves only `static/current`. After promotion, a document cached for up to 60 seconds can still reference an old fingerprinted image that is absent from the new current release. An edge or browser miss gets 404. Rollback creates the reverse gap. The new URLs alone do not guarantee the claimed Cloudflare no-purge behavior.  
**Fix:** Serve fingerprinted media from a stable shared namespace or resolve retained release media by URL through the document freshness and rollback window. Add a test for old and new URL reachability across promotion and rollback.

## Warnings

### WR-01: Cached dimensions can diverge from verified WebP bytes

**File:** `controller/src/publisher.rs:594`  
**Finding family:** `derivative_cache_metadata_dimensions`  
**Lineage:** `independent`  
**Shared invariant:** Public image dimensions describe the exact WebP payload whose bytes determine the public URL.  
**Consumers inspected:** `DerivativeCache::write/read`, `build_public_items`, candidate validation, public contract, damaged-cache tests.  
**Evidence level:** `source_only`  
**PR comment:** https://github.com/jetsaredim/autographs/pull/294#discussion_r4378675458  
**Issue:** A cache metadata width or height changed alone passes both key and payload digest checks. The reused derivative exports the corrupt dimension to public JSON; candidate validation never compares decoded WebP dimensions.  
**Fix:** Decode dimensions on cache reuse and compare them with metadata, or derive dimensions from the verified bytes. Regenerate on mismatch and add a dimension-corruption regression.

---

_Reviewer: GSD code reviewer; review round 1._

## Round 2 Review — 41809f1

**Reviewed:** 2026-10-26T15:07:51Z
**Status:** issues_found
**Evidence level:** source-only; the local Caddy test checks config text and the publisher tests model URL resolution through filesystem reads. The credentialed Caddy and Cloudflare smoke remains outstanding.

### Round 1 finding disposition

- `CR-01` / `fingerprinted_media_release_lifetime`: The shared media route and retention fix keep previously published URLs available after normal correction, unpublish, and rollback in the local filesystem model. That original symptom is resolved. The fix introduces `CR-02`, a pre-promotion exposure in the same release/media boundary.
- `WR-01` / `derivative_cache_metadata_dimensions`: Resolved. Cache reuse decodes WebP bytes and compares actual dimensions to metadata, and the new test changes the stored width for both variants and checks regeneration.

### CR-02: Candidate media becomes public before release promotion

**Severity:** BLOCKER
**File:** `controller/src/publisher.rs:2113`
**Finding family:** `candidate_media_precommit_exposure`
**Lineage:** `fix_regression` (introduced by the `CR-01` shared-media fix)
**Shared invariant:** Only media belonging to a committed public release is anonymously served. A failed or in-progress candidate remains private.
**Consumers inspected:** Candidate build and validation, `prepare_shared_media`, `copy_release_media_to_shared`, `promote_candidate`, failure cleanup, Caddy `/media/*` route, failed-promotion test, live smoke, public privacy and CDN contracts.
**Evidence level:** `source_only`
**PR comment:** https://github.com/jetsaredim/autographs/pull/294#discussion_r4378860862
**Issue:** `prepare_shared_media` renames candidate WebP files into the Caddy-public `static/media` path before `promote_candidate` changes `current`. The after-failure cleanup test does not test the interval in which those URLs are accessible. If promotion fails or stalls, candidate media has already crossed the public boundary.
**Fix:** Copy only active-release media to shared storage before promotion. Leave candidate media private until `current` changes; Caddy's `current/media` fallback can serve new URLs immediately after that switch. Copy new active media into shared storage after promotion. Verify candidate URLs are unavailable before promotion, including a controlled failure or pause.

### WR-02: Emergency takedown can wedge cleanup and resurrect a missed origin copy

**Severity:** WARNING
**File:** `controller/src/publisher.rs:2130`
**Finding family:** `emergency_media_takedown_retention`
**Lineage:** `sibling_path_miss` (retention and emergency-removal consumer of the `CR-01` shared-media invariant)
**Shared invariant:** A removed emergency media URL stays removed, and routine release pruning remains operable after the documented removal procedure.
**Consumers inspected:** CDN and runtime takedown instructions, shared-media route, `copy_release_media_to_shared`, release pruning, shared pruning, post-promotion warning, unpublish test, rollback test, failed-promotion test.
**Evidence level:** `source_only`
**PR comment:** https://github.com/jetsaredim/autographs/pull/294#discussion_r4378862927
**Issue:** The runbook directs operators to delete affected WebP files from shared and retained release copies. When a retained release is later pruned, its manifest still lists the deleted file, so `copy_release_media_to_shared` fails reading it before the release can be removed. Cleanup then skips shared pruning and repeats the warning on later publishes. If a retained copy is missed, the same callback can republish the purged URL.
**Fix:** Represent emergency removal durably, or retire/update affected releases, and make pruning honor that state without recreating removed media. Test the full takedown and later release-pruning sequence.

## Convergence Reassessment

**Finding lineage:** Round 1 `CR-01` and `WR-01` were addressed in the pushed fix. Round 2 `CR-02` is a regression from `CR-01`; `WR-02` is a missed retention/takedown consumer of the same shared-media change. These are recurring families for convergence purposes, so status is `reassessment_required` pending revised assumptions and implementation plan evidence.

**Shared invariants:** (1) Candidate media is private until the active release pointer changes. (2) A public URL referenced by a fresh document remains fetchable through normal promotion, rollback, and unpublish windows. (3) Emergency-removed media is never served or regenerated by retention work. (4) Public dimensions match verified bytes.

**Consumer inventory:** Private original and adjustment producers; derivative cache read/write; candidate generator and validator; candidate-to-shared copy; Caddy shared-root and `current/media` fallback; pointer promotion and failure cleanup; rollback and release pruning; shared-media age pruning; unpublish; emergency origin removal and CDN purge; status warnings; publisher, Caddy-text, and credentialed live-smoke tests. The dimension path is resolved; the public release/media transaction and emergency cleanup paths need one coherent contract.

**Assumption audit:** The fix assumed copying a validated candidate into an anonymously served directory was safe before promotion. Validation proves content correctness, not publication authorization. It also assumed every retained release manifest remains readable and complete, whereas the documented emergency path intentionally deletes media files. Local filesystem reads and Caddy string assertions do not prove live HTTP behavior.

| Failure point | Current effect | Required behavior |
| --- | --- | --- |
| Candidate shared copy succeeds; promotion is pending | New candidate URL is anonymously readable | Candidate URL remains unavailable until promotion |
| Promotion fails after shared copy | Cleanup removes new files only afterward; removal can itself fail | No precommit public exposure or persistent candidate URL |
| Emergency deletion removes shared and retained media | Later release pruning fails on missing manifest-listed bytes | Tombstoned media stays unavailable and cleanup completes |
| Operator misses one retained copy during takedown | Release pruning can copy it back to shared media | Retention respects durable takedown state |
| Normal promotion or rollback with cached documents | Old and new URLs must resolve across origin misses | Preserve the fixed `CR-01` lifetime guarantee |

**Revised plan requirements:** Specify one commit boundary for public media visibility; use the existing Caddy fallback to serve new active media after pointer promotion; preserve old media without exposing candidate files early; define durable emergency tombstones or an equivalent release-retirement process; keep pruning safe after intentional media deletion; add deterministic tests for precommit invisibility, promotion failure, rollback, takedown, and later pruning, plus credentialed Caddy/edge proof before CDN enablement.

**Resume criteria:** Independently review revised assumptions and revised implementation plan covering every invariant and failure row above. Record their evidence paths in `assumptions_evidence` and `plan_evidence` before changing convergence status to `ready_to_resume`. No status-only transition is valid.

## Round 3 Convergence Checkpoint — 40ff873

**Reviewed:** 2026-10-26T20:46:29Z
**Status:** issues_found; one in-scope Blocker remains. No further point-fix round should start.

### Finding lineage and resolved paths

- `CR-01` (`fingerprinted_media_release_lifetime`): The shared media root preserves prior fingerprints across normal promotion, unpublish, and rollback in local tests and the real Caddy fixture. Resolved in the reviewed source.
- `WR-01` (`derivative_cache_metadata_dimensions`): Cache reuse decodes WebP bytes and verifies dimensions. Resolved in the reviewed source and regression test.
- `CR-02` (`candidate_media_precommit_exposure`, `CR-01` fix regression): The publisher now migrates only active media before pointer promotion and copies candidate media after promotion. The controlled callback and real Caddy fixture support the local boundary. The production deployment path remains unprotected; `CR-03` is a sibling-path miss of this family.
- `WR-02` (`emergency_media_takedown_retention`): Caddy denies marked URLs before shared/current fallback; the publisher rejects marked candidates and skips marked copies during pruning. Local tests exercise missing retained bytes, and the real Caddy fixture exercises both origin routes. Resolved in the reviewed source. The credentialed production takedown remains a Wave 8 proof.

### CR-03: Production deployment does not preserve the publish freeze

**Severity:** BLOCKER
**File:** `deploy/ansible/roles/autographs_deploy/tasks/publish_freeze.yml:19`
**Finding family:** `candidate_media_precommit_exposure`
**Prior family ID:** `CR-02`
**Lineage:** `sibling_path_miss`
**Shared invariant:** During the non-atomic Caddy/controller rollout, no old publisher request can cross into a new public shared-media route until the revised publisher and both origin routes are verified.
**Consumers inspected:** Manual freeze runbook, Ansible Caddyfile copy and block insertion, handler flush and Caddy restart, quadlet bind mount, frozen/unfrozen render tests and real Caddy fixture, `deploy/ansible/playbooks/deploy.yml`, production `.github/workflows/deploy.yml` invocation, publisher route lock/status.
**Evidence level:** `source_only` for the production workflow mismatch; `deterministic_test` for the rendered freeze switch and Caddy status behavior.
**PR comment:** https://github.com/jetsaredim/autographs/pull/294#discussion_r4382273384
**Issue:** The production GitHub Actions deployment does not pass `autographs_deploy_publish_frozen`. The role defaults it to `false`, copies a clean Caddyfile, removes the freeze block, and restarts Caddy. Thus the normal Release-PR deployment cannot carry the documented manual gate through cutover. An old controller may accept a publish while the shared media route is live.
**Fix:** Make the production deployment path preserve or enforce the freeze through both service restarts and readiness checks, then explicitly unfreeze. A deployment test must exercise the same parameters as the GitHub Actions call, including a pre-existing manual freeze, and verify the gate remains active until deliberate release.

### Verification performed in round 3

- `cargo test --manifest-path controller/Cargo.toml --test publisher --test caddy_static_routes --test static_contract --quiet`: passed (49 publisher, 9 Caddy, 4 static-contract tests).
- `bash -n scripts/verify-caddy-media-origin.sh` and `git diff --check`: passed.
- `bash scripts/verify-caddy-media-origin.sh`: passed against containerized Caddy, including precommit 404, postcommit 200, marker denial on public and preview routes, and frozen/unfrozen publish route responses. This does not exercise the production GitHub Actions invocation or credentialed OCI/Oracle/Cloudflare paths.

## Round 3 Convergence Reassessment

**Finding lineage:** `CR-03` is the still-open deployment consumer of round-2 `CR-02`, which was a regression from round-1 `CR-01`. The local publisher boundary is fixed, but the production deploy can remove its only gate. This is the mandatory third-review checkpoint; no additional point fix is authorized until assumptions and plan are revised and reviewed.

**Shared invariants:** Candidate URLs remain private before pointer commit; old committed URLs stay available through bounded document freshness; emergency-marked URLs remain denied; and no old publisher can run under new shared-media serving during deployment. The first three are supported locally; the fourth fails in the actual release deployment path.

**Complete consumer inventory:** Candidate generator/validator; derivative cache; active-only shared copy; pointer promotion; postcommit copy and warning; Caddy shared/current/marker routes; rollback and pruning; emergency marker and CDN purge; admin publish routes, lock, status, queued jobs; manual host freeze; Ansible Caddyfile copy/blockinfile/handler restart; controller and Caddy quadlets; the production GitHub Actions deploy call; local publisher/Caddy/Ansible tests; credentialed live smoke and Wave 8 edge probes.

**Assumption audit:** The revision assumed the operator would invoke Ansible with `autographs_deploy_publish_frozen=true`. The repository's normal delivery contract uses `.github/workflows/deploy.yml`, which supplies no such variable; the role's `false` default removes the gate. The local render fixture explicitly tests `true` and `false` but never the production call's implicit default against an already frozen host.

| Failure point | Current effect | Required behavior |
| --- | --- | --- |
| Manual gate is installed, then GitHub Actions runs its normal deploy | Role copies a clean Caddyfile and renders freeze `absent` | Gate survives until verified explicit unfreeze |
| New Caddy route restarts while old controller can still publish | Old publisher can place candidate media in public shared storage precommit | Publish routes remain blocked and queued work drained |
| Deployment fails after Caddy restart but before controller/readiness checks | Implicit `false` leaves publish routes open | Failed rollout remains frozen and recoverable |
| Controller and both Caddy routes pass readiness | No automatic proof is connected to gate removal | Explicit readiness-backed unfreeze and evidence record |

**Revised plan requirements:** Reconcile the release-please/GitHub Actions deployment with the freeze protocol. Specify where the gate is set before mutation, how the exact production workflow passes or reads frozen state, how queued/in-flight jobs are drained, how every restart and failure preserves it, and how an explicit readiness gate removes it. Extend deployment tests to model the real invocation and a pre-frozen host. Preserve all current publisher, marker, retention, and Caddy behavior.

**Resume criteria:** Produce revised deployment assumptions and a reviewed implementation plan that cover each failure row and the production workflow. Record both evidence paths, plus the plan-review evidence, before changing `convergence.status` to `ready_to_resume`. A status-only change is invalid. The next coder action must follow that reviewed plan as one coherent contract change.

## Round 4 Review — b33a421

**Reviewed:** 2026-10-27T02:33:14Z
**Status:** issues_found; one Blocker and one Warning.

### Prior finding disposition

- `CR-01` and `WR-01`: The shared namespace and decoded cache dimensions remain correct in publisher, Caddy, and contract tests. `CR-04` below is a later deployment consumer that can remove a valid old shared URL.
- `CR-02`: Pointer-first candidate visibility remains correct in the local publisher and real Caddy origin fixture. The newly introduced audit has a separate retention failure.
- `WR-02`: Publisher pruning and Caddy marker denial remain correct. `WR-03` below is the newly added deployment preflight consumer that rejects a documented active-media takedown.
- `CR-03`: The actual production workflow now runs preflight, passes `autographs_deploy_publish_frozen=true`, starts the reviewed controller before new Caddy routing, checks frozen origin readiness, and explicitly unfreezes before release completion. The role retains the gate through its service restarts. Resolved in source and production-shaped local tests; no production rollout was executed.

### CR-04: Routine subsequent deployment quarantines a committed old media URL

**Severity:** BLOCKER
**File:** `deploy/ansible/roles/autographs_deploy/files/publish_freeze_audit.py:114`
**Finding family:** `fingerprinted_media_release_lifetime`
**Prior family ID:** `CR-01`
**Lineage:** `sibling_path_miss`
**Shared invariant:** A committed media URL referenced by a still-fresh document remains anonymously fetchable through a later production deployment, and the cutover audit distinguishes committed old URLs from candidate-only files.
**Consumers inspected:** Publisher active-only precommit copy, postcommit shared copy, release retention/pruning and rollback; shared/current Caddy routes; preflight and role reaudit; production workflow on every mutating release; local audit fixture, publisher regression, and CDN/runbook contract.
**Evidence level:** `deterministic_test`
**PR comment:** https://github.com/jetsaredim/autographs/pull/294#discussion_r4384547991
**Issue:** After normal correction or unpublish, an old committed fingerprint remains in shared `media/` and a retained release while `current` points to the new release. The audit regards only current media as proven, moves the old file out of the public root, and exits 2 because it is retained or younger than 24 hours. The production workflow invokes this preflight for every mutating deployment. A disposable active-plus-retained reproduction returned exit 2 and showed the old shared file absent. The existing audit test expects this outcome but models the retained release as ambiguous, missing the normal committed-history case.
**Fix:** Persist or verify promotion provenance for prior committed URLs and preserve those files through their retention period; quarantine candidate-only paths. Test successful publications A then B followed by the next deployment preflight, including an old-URL origin miss.

### WR-03: Preflight rejects intentional active-media emergency removal

**Severity:** WARNING
**File:** `deploy/ansible/roles/autographs_deploy/files/publish_freeze_audit.py:76`
**Finding family:** `emergency_media_takedown_retention`
**Prior family ID:** `WR-02`
**Lineage:** `sibling_path_miss`
**Shared invariant:** An exact durable marker permits an intentionally deleted active or retained media file while continuing to deny its URL and allowing later deployment.
**Consumers inspected:** Emergency-removal runbook, Caddy deny on both routes, publisher candidate rejection and tombstone-aware copy/prune, active-manifest verification in preflight, full-role reaudit, and audit tests.
**Evidence level:** `deterministic_test`
**PR comment:** https://github.com/jetsaredim/autographs/pull/294#discussion_r4384548637
**Issue:** The runbook explicitly permits deleting the active release copy under a durable exact marker when needed. The publisher skips marked entries before reading them. The preflight verifies every active manifest media file without consulting markers, so the next deploy fails closed before its audit or controller cutover. A disposable active manifest with a regular marker and removed WebP reproduced exit 1 (`invalid shared media path`).
**Fix:** Verify unmarked active media strictly, but accept an intentionally absent marked file and keep the marker denial authoritative. Test active takedown followed by deployment preflight.

### Verification performed in round 4

- `python3 -m unittest deploy/ansible/tests/test_publish_freeze_deploy.py`: six tests passed. The retained-manifest test currently encodes the `CR-04` failure.
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check` on preflight, unfreeze, and rollback: passed.
- Production-shaped `publish-freeze-deploy-test.yml`: passed.
- Focused Cargo tests: 49 publisher, 9 Caddy, and 4 static-contract tests passed.
- Containerized Caddy origin HTTP matrix: passed for precommit 404, postcommit 200, retained old URL 200, marked URL 404 on both routes, and frozen/unfrozen publish responses.
- Disposable direct audit reproductions: ordinary active-plus-retained media returned exit 2 and quarantined the old shared file; an active marked/deleted file returned exit 1.
- `git diff --check` for this local report edit: passed. No credentialed production OCI, Oracle, or Cloudflare run occurred.

## Round 4 Convergence Reassessment

**Finding lineage:** `CR-03`, the third-review deployment-gate finding in the `CR-02` candidate-visibility family, is resolved by the revised workflow and frozen role sequence. The third-review checkpoint was satisfied by the revised assumptions, contract plan, independent plan review, and separate convergence artifact before round-3 coder work. Round 4 exposes `CR-04`, a missed deployment consumer of `CR-01`'s committed URL lifetime, and `WR-03`, a missed preflight consumer of `WR-02`'s emergency marker contract. Both are recurring families, so further coder work requires renewed assumptions and plan review.

**Shared invariants:** New candidate media is private until pointer commit; only committed shared media may be served; every normally retained old fingerprint remains fetchable through origin misses and later deployments; exact emergency markers deny both origin routes and permit intentionally removed bytes; the old process exits before new shared routing; deployment keeps the gate until explicit readiness-backed unfreeze.

**Complete consumer inventory:** Private originals and derivative cache; candidate build/validation; pointer promotion; active/shared copy and retention; Caddy current/shared/tombstone routes; emergency marker and exact purge; rollback/unpublish/pruning; old controller request queue and process-exit barrier; preflight quarantine and durable record; role reaudit and service ordering; production GitHub Actions preflight/deploy/unfreeze/retry/rollback steps; public and preview origin probes; local publisher/Caddy/Ansible/Python tests and credentialed live checks.

**Assumption audit:** The round-3 revision correctly treated a retained manifest alone as insufficient proof for the *initial* cutover, because an old candidate could be retained without pointer promotion. It applied that one-time rule to every later deployment without recording a postcutover promotion ledger. After ordinary operation, a retained manifest and shared file may represent a known committed prior URL, but the audit cannot tell. The audit also assumed every current-manifest image file exists, whereas the documented one-way emergency path deliberately removes a marked active file.

| Failure point | Current effect | Required behavior |
| --- | --- | --- |
| Normal publish B follows committed A; next production deploy audits A in shared media | A is quarantined and preflight exits 2, breaking an old origin URL | Preserve proven committed A through retention while rejecting candidate-only bytes |
| A shared URL has aged out but is still referenced by a retained release | It is quarantined and preflight still exits 2 | Decide retention from committed provenance and release policy, not current-only membership |
| Emergency marker protects an active URL whose source copy was deleted | Active media verification exits 1 before cutover | Marker-aware verification accepts intentional absence while keeping unmarked corruption fatal |
| Preflight or role reaudit fails after controller stop | Gate remains, but routine deployment cannot complete and quarantined old URLs remain unavailable | Keep the origin contract while stopped, provide deterministic recovery and retry evidence |

**Revised plan requirements:** Add a durable, auditable committed-media history for postcutover publishes or an equivalent authoritative source that the preflight can verify before moving shared files. Preserve the strict first-cutover ambiguity stop. Make active verification marker-aware and never restore a marked URL. Recheck preflight and role reaudit on successive successful publishes, retained-release pruning, rollback, emergency active removal, failure, and retry. The revised plan must include public/preview origin-miss checks for old and marked URLs.

**Resume criteria:** Independently revise and review assumptions and implementation plan for both failures and all sibling paths above; record those evidence references in the convergence artifact before changing its status to `ready_to_resume`. Keep the review findings and fixes linked in PR #294. A status-only transition is invalid.

## Round 5 Review — 67af9cf

**Reviewed:** 2026-10-27T09:39:17Z
**Status:** issues_found; zero Blockers and one Warning. The reviewed round-4 reassessment authorized this implementation round. The new Warning requires renewed convergence reassessment before another point fix.

### Prior finding disposition

- `CR-01` and `CR-04` (`fingerprinted_media_release_lifetime`): Versioned committed-release receipts now prove old URLs across later deployment audits. The publisher retains verified history, and the audit distinguishes committed retained bytes from candidate-only files. The local publisher, audit, and real Caddy origin tests pass successive publish, old-URL origin miss, rollback, and pruning cases. Resolved in reviewed source; credentialed production proof remains outstanding.
- `WR-01` (`derivative_cache_metadata_dimensions`): Decoded WebP dimension verification remains in place and its regression passes. Resolved.
- `CR-02` and `CR-03` (`candidate_media_precommit_exposure`): Pointer-first visibility, old-controller process exit, frozen deployment ordering, and explicit unfreeze gating remain in source and production-shaped local tests. Resolved for the reviewed local paths.
- `WR-02` and `WR-03` (`emergency_media_takedown_retention`): The audit now permits exact-marked active files to be absent; Caddy denies marked public and preview paths, and the regression passes. `WR-04` below is an unfreeze consumer of the same valid state.

### WR-04: Explicit unfreeze rejects a release with no unmarked active image

**Severity:** WARNING
**File:** `deploy/ansible/playbooks/publish-unfreeze.yml:180`
**Finding family:** `emergency_media_takedown_retention`
**Prior family IDs:** `WR-02`, `WR-03`
**Lineage:** `sibling_path_miss`
**Shared invariant:** An exact-marked active deletion may leave no anonymously available active image, yet a healthy frozen deployment must be able to prove the current document, verify marked URLs remain denied, and explicitly unfreeze.
**Consumers inspected:** Emergency marker writer and Caddy deny routes; publisher marked-media validation and pruning; audit active receipt and `active_probe` record; preflight's `manifest.json` fallback; role reaudit; unfreeze's public and preview probes; rollback helper; Python, Cargo, Ansible, and origin tests.
**Evidence level:** `source_only` for the unfreeze control-flow failure; `deterministic_test` for valid marked deletion in the audit and real Caddy fixture.
**PR comment:** https://github.com/jetsaredim/autographs/pull/294#discussion_r4387116272
**Issue:** The audit writes `active_probe: ""` when every active media path is marked. Preflight correctly probes `manifest.json` in that state. Unfreeze's Jinja `default(...)` preserves the defined empty string, then asserts a WebP path and stops. A fallback to the first manifest media path alone would select a marked 404 URL. Thus a sole-image emergency removal, or an empty current release, can pass preflight and reaudit but leave the healthy deployment permanently frozen.
**Fix:** Apply the same public-document fallback at explicit unfreeze, retain 200 checks for both origin routes when an unmarked image exists, and prove marked paths stay 404. Add a production-shaped full preflight, reaudit, and unfreeze test with the sole active image marked and deleted.

### Verification performed in round 5

- `python3 -m unittest deploy/ansible/tests/test_publish_freeze_deploy.py`: 14 tests passed, including retained history, intent/pointer recovery, initial cutover ambiguity, marker-aware deletion, and rollback paths. The suite does not exercise full unfreeze with every active image marked.
- `cargo test --manifest-path controller/Cargo.toml --test publisher --test caddy_static_routes --test static_contract --quiet`: 49 publisher, 9 Caddy, and 4 static-contract tests passed.
- Ansible syntax checks for deploy, preflight, unfreeze, and rollback playbooks passed. The production-shaped `publish-freeze-deploy-test.yml` passed.
- `bash scripts/verify-caddy-media-origin.sh` passed against containerized Caddy: old committed A remains 200 after B and subsequent audit, rollback A/B origin misses return 200, tombstoned and marked active media return 404, history and marker files stay private, and frozen/unfrozen routes return the expected statuses. The fixture does not run the unfreeze playbook in the all-marked state.
- No credentialed live Oracle, OCI, production Caddy, or Cloudflare run was performed.

## Round 5 Convergence Reassessment

**Finding lineage:** The round-4 `CR-04` and `WR-03` defects are resolved by the reviewed history journal, marker-aware audit, and release rollback contract. `WR-04` is a sibling-path miss of the same `emergency_media_takedown_retention` family as `WR-02` and `WR-03`: the audit and preflight accept a marked/deleted active image, but explicit unfreeze still requires an image 200. Because a recurring Warning remains, `convergence.status` is `reassessment_required`; further point fixes need revised assumptions and reviewed plan evidence.

**Shared invariants:** Candidate-only paths remain private until pointer commit; committed URLs stay fetchable through publication, later audit, rollback, and bounded retention; exact-marked URLs stay denied and their missing bytes do not block release maintenance; deployment remains frozen until process provenance, static/current integrity, and origin readiness are proved; origin readiness must be provable even when no active media is publicly fetchable.

**Consumer inventory:** Source originals and derivative cache; candidate receipts; commit intent, pointer, history reconciliation, and publication status; retained shared URLs and last-active pruning; rollback helper and marker-aware emergency removal; Caddy shared/current and marker routes; initial cutover inventory, old process exit, quarantine, and history migration; later preflight and role reaudit; release workflow, frozen Caddy/controller ordering, and explicit unfreeze public/preview probes; Cargo, Python, Ansible, Caddy origin, and credentialed live tests.

**Assumption audit:** The round-4 revision correctly made active receipt verification marker-aware and selected a document probe in preflight when no unmarked image exists. It assumed explicit unfreeze would use the same state-aware probe. The stored `active_probe` is intentionally empty in that state; Jinja's `default` filter does not replace a defined empty string without its truthy argument. A first manifest image is not a safe substitute because it may be the marked URL that must return 404.

| Failure point | Current effect | Required behavior |
| --- | --- | --- |
| Exact marker protects the sole active image and its bytes are deleted | Audit, preflight, and role reaudit accept; unfreeze asserts nonempty WebP and stops | Probe a public document on both origins, verify marker denial, then permit explicit unfreeze |
| Active release has no images | Audit records an empty active probe; unfreeze stops | Use a document probe and complete readiness proof |
| Active release contains both unmarked and marked images | Unmarked image is selected; current local checks pass | Keep unmarked 200 and marked 404 proofs through unfreeze |
| Rollback selects a release whose images are all marked | Pointer and history recovery can complete; next unfreeze fails | Apply the same state-aware readiness rule after rollback |

**Revised plan requirements:** Treat active readiness as a document-or-unmarked-media choice shared by preflight, role reaudit, and unfreeze. Specify how public and preview origin probes prove 200 for a safe current artifact and 404 for every relevant exact-marked URL. Cover empty releases, sole-image takedown, mixed marked/unmarked media, and rollback to an all-marked release in the production-shaped playbook test; preserve fail-closed behavior for missing unmarked bytes and unproven retained history.

**Resume criteria:** Revise assumptions and implementation plan for this complete probe contract, obtain independent plan review, and record those evidence paths in the convergence artifact before changing status to `ready_to_resume` or starting another coder point fix. The posted `WR-04` PR comment and this report retain the finding lineage.
