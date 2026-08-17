---
phase: 08-admin-media-review-and-operational-posture
plan: "03"
subsystem: operations
tags: [cdn, cache, cloudflare, caddy, runbooks, testing]

requires:
  - phase: 08-admin-media-review-and-operational-posture
    provides: Plan 08-02 landed the pre-media posture PR evidence required before CDN/cache contract work
provides:
  - Phase 8 CDN/cache contract with Cloudflare rule names, origin headers, rollback, purge, and production verification probes
  - Source-level Caddy/cache contract assertions for origin headers and documented CDN rule vocabulary
  - Operator runbook links for CDN/cache contract and post-media verification timing
affects: [phase-08, ops-02, caddy, cloudflare, static-runtime, deployment-runbook]

tech-stack:
  added: []
  patterns:
    - CDN behavior is documented before media adjustment implementation
    - Caddy origin cache posture is source-tested against the operator contract
    - Routine media corrections rely on new fingerprinted public media URLs rather than manual purge

key-files:
  created:
    - docs/cdn-cache-contract.md
    - .planning/phases/08-admin-media-review-and-operational-posture/08-03-SUMMARY.md
  modified:
    - docs/dns-runbook.md
    - docs/static-runtime-runbook.md
    - docs/deployment-runbook.md
    - controller/tests/caddy_static_routes.rs
    - .planning/STATE.md
    - .planning/ROADMAP.md

key-decisions:
  - "Production CDN enablement remains deferred until adjusted-media cache behavior is proven by later Phase 8 media and publisher work."
  - "Cloudflare rules are named exactly `Bypass admin and API`, `Respect rollback-sensitive public documents`, and `Cache fingerprinted media and assets`."
  - "OPS-02 remains pending overall because production CDN enablement and post-media verification are owned by later Phase 8 plans."

patterns-established:
  - "Origin cache headers, Cloudflare rule names, purge behavior, rollback behavior, and production probes live together in docs/cdn-cache-contract.md."
  - "Caddy route tests read docs/cdn-cache-contract.md so operator-facing CDN rules cannot drift silently from source headers."

requirements-completed: []

duration: 8min
completed: 2026-08-17
---

# Phase 08 Plan 03: CDN Cache Contract Summary

**Cloudflare/CDN cache behavior is now documented, linked from operator runbooks, and enforced by Caddy route contract tests before Phase 8 media adjustment work begins.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-08-17T03:00:24Z
- **Completed:** 2026-08-17T03:07:48Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments

- Created `docs/cdn-cache-contract.md` with required sections for origin headers, Cloudflare cache rules, routine adjustment cache behavior, purge/rollback, and verification.
- Documented exact Cloudflare rule names: `Bypass admin and API`, `Respect rollback-sensitive public documents`, and `Cache fingerprinted media and assets`.
- Linked the contract from `docs/dns-runbook.md`, `docs/static-runtime-runbook.md`, and `docs/deployment-runbook.md`.
- Added Caddy route tests that read the contract and assert admin/API bypass paths, fingerprinted media/assets paths, rollback-sensitive public paths, and origin cache headers.
- Added production verification probes for admin, admin API, public JSON, repeated media requests, `Cache-Control`, and `CF-Cache-Status`.

## Task Commits

1. **Task 1: Commit the Phase 8 CDN/cache contract** - `5a8e481` (docs)
2. **Task 2: Enforce the origin header contract in source tests** - `1010c1e` (test)
3. **Task 3: Add rollback and production-verification hooks to operator docs** - `5bb9972` (docs)

**Plan metadata:** Pending closeout commit.

## Files Created/Modified

- `docs/cdn-cache-contract.md` - Canonical Phase 8 CDN/cache contract covering Caddy origin headers, Cloudflare rules, routine image adjustment fingerprints, rollback, purge, and probes.
- `docs/dns-runbook.md` - Links to the contract and records that CDN enablement waits for adjusted-media cache proof.
- `docs/static-runtime-runbook.md` - Links Caddy cache verification guidance to the Phase 8 contract.
- `docs/deployment-runbook.md` - Links deployment verification to the Phase 8 CDN/cache probes.
- `controller/tests/caddy_static_routes.rs` - Adds source assertions that the Caddyfile and documented CDN/cache contract stay aligned.

## Decisions Made

- Production CDN enablement is not part of this plan. It remains deferred until adjusted-media cache behavior is proven after the admin media and publisher cache-key work exists.
- `OPS-02` remains pending overall because later Phase 8 plans still own production CDN enablement and verification after adjusted media is implemented.
- Caddy route shape stayed unchanged because existing origin headers already satisfied the documented contract.

## Deviations from Plan

### Auto-fixed Issues

None.

### TDD Gate Notes

**1. Task 2 RED did not fail after test addition**
- **Found during:** Task 2 (Enforce the origin header contract in source tests)
- **Issue:** The new guardrail test passed immediately because Task 1 had already created the required contract text and the existing Phase 6 Caddy origin headers already satisfied the no-store, media, and public-document cache behavior.
- **Fix:** No implementation change was required; the planned test guardrail was committed as a test-only task change.
- **Files modified:** `controller/tests/caddy_static_routes.rs`
- **Verification:** `cargo test --manifest-path controller/Cargo.toml --test caddy_static_routes -- --nocapture`
- **Committed in:** `1010c1e`

---

**Total deviations:** 0 auto-fixed; 1 TDD gate note.
**Impact on plan:** No scope change. The behavior existed before Task 2, and the new source test now makes it verifiable.

## Issues Encountered

None.

## Verification

- `cargo test --manifest-path controller/Cargo.toml --test caddy_static_routes -- --nocapture` passed.
- `rg -n "Bypass admin and API|Respect rollback-sensitive public documents|Cache fingerprinted media and assets|/admin/api/\\*|/media/\\*|no-store|rollback|purge" docs/cdn-cache-contract.md` passed.
- `rg -n "cdn-cache-contract.md|Phase 8 CDN/cache contract" docs/dns-runbook.md docs/static-runtime-runbook.md` passed.
- `rg -n "CF-Cache-Status|AUTOGRAPHS_DOMAIN|admin/api/status|data/collection.json|media/\\.\\.\\.webp|adjusted-media cache behavior" docs/cdn-cache-contract.md docs/deployment-runbook.md docs/dns-runbook.md` passed.
- `rg -n "docs/cdn-cache-contract.md|Bypass admin and API|Cache fingerprinted media and assets" controller/tests/caddy_static_routes.rs` passed.
- `rg -n "handle /admin/api/\\*|Cache-Control \\\"no-store\\\"|@staticMedia path /media/\\*" deploy/ansible/roles/autographs_deploy/files/Caddyfile` passed.
- `git diff --check` passed.

## Known Stubs

None.

## Threat Flags

None. The plan's threat model already covered browser -> Cloudflare -> Caddy cache behavior and publisher -> public static release cache behavior.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Plan 08-04 is ready to start image adjustment transform/model work. The CDN/cache contract is now in place before media implementation, while production CDN enablement remains deferred until later Phase 8 adjusted-media verification.

## Self-Check: PASSED

- Created summary file exists on disk.
- Task commits `5a8e481`, `1010c1e`, and `5bb9972` exist in git history.
- Stub scan found no new plan stubs; the existing `not available` phrase in `docs/dns-runbook.md` is a factual OCI DNS note, not placeholder UI/content.
- No threat surface beyond the plan's CDN/cache and public static release boundaries was introduced.

---
*Phase: 08-admin-media-review-and-operational-posture*
*Completed: 2026-08-17*
