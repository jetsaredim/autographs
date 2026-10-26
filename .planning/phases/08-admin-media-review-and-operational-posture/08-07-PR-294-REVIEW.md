---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-26T14:51:07Z
depth: deep
review_round: 1
files_reviewed: 14
files_reviewed_list:
  - .planning/phases/08-admin-media-review-and-operational-posture/08-07-PLAN.md
  - .planning/phases/08-admin-media-review-and-operational-posture/08-07-SUMMARY.md
  - controller/src/config.rs
  - controller/src/publisher.rs
  - controller/tests/live_static_publish_smoke.rs
  - controller/tests/publisher.rs
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
  status: clear
  recurring_critical_warning: 0
  scope_expansion: false
  requires_reassessment: false
  assumptions_evidence: null
  plan_evidence: null
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
