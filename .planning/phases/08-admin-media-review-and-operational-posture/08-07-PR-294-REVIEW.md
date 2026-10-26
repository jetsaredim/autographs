---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-26T15:07:51Z
depth: deep
review_round: 2
files_reviewed: 16
files_reviewed_list:
  - .planning/phases/08-admin-media-review-and-operational-posture/08-07-PLAN.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-07-PR-294-REVIEW-FIX.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-07-SUMMARY.md
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
  - deploy/ansible/roles/autographs_deploy/files/Caddyfile
findings:
  critical: 1
  warning: 1
  info: 0
  total: 2
status: issues_found
convergence:
  status: ready_to_resume
  recurring_critical_warning: 2
  scope_expansion: false
  requires_reassessment: false
  assumptions_evidence: 08-PR-294-ASSUMPTIONS.md
  plan_evidence: 08-PR-294-CONTRACT-REVISION.md
  plan_review_evidence: 08-PR-294-CONTRACT-PLAN-REVIEW.md#revision-2--verification-passed
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
