---
phase: 06-admin-collection-workflow
plan: 09
subsystem: optimization
tags: [static-delivery, derivatives, cache, cdn, cleanup]

requires:
  - phase: 06-admin-collection-workflow
    provides: completed admin workflow and refreshed codebase maps
provides:
  - public derivative size reduction
  - origin cache header posture
  - deferred Cloudflare/CDN decision record
  - post-Phase 6 runtime cleanup guidance
affects: [publisher, caddy, runbooks, codebase-maps]

tech-stack:
  added: []
  patterns: [measured optimization, moderate cache headers, deferred CDN checklist]

key-files:
  created: []
  modified:
    - controller/src/derivatives.rs
    - controller/tests/publisher.rs
    - controller/tests/caddy_static_routes.rs
    - deploy/ansible/roles/autographs_deploy/files/Caddyfile
    - docs/dns-runbook.md
    - docs/deployment-runbook.md
    - docs/static-runtime-runbook.md
    - docs/security-review.md
    - .planning/codebase/ARCHITECTURE.md
    - .planning/codebase/INTEGRATIONS.md
    - .planning/codebase/TESTING.md

key-decisions:
  - "Detail derivatives are capped at 960x1280 while thumbnails remain capped at 480x640."
  - "Cloudflare/CDN fronting is deferred for v1, with an enablement checklist and admin/API bypass requirements documented."
  - "Caddy origin cache headers are moderate because generated public paths are slug-based, not content-addressed."
  - "Post-Phase 6 VM cleanup remains operator-run and dry-run-first; local validation does not claim live cleanup."

patterns-established:
  - "Optimization claims require before/after byte evidence."
  - "Admin/API paths use no-store; public HTML/JSON are short-lived; public media/assets are cacheable but rollback-aware."

requirements-completed: [STATIC-03, STATIC-04, STATIC-05, ADMIN-05, SHIP-01, SHIP-05]

duration: live session
completed: 2026-07-02
---

# Phase 06-09: Static Delivery Optimization Summary

**Final Phase 6 optimization reduced generated detail derivative size, added rollback-aware cache headers, documented CDN posture, and recorded runtime cleanup guidance.**

## Performance

- **Duration:** live session
- **Started:** 2026-07-02
- **Completed:** 2026-07-02
- **Tasks:** 3
- **Files modified:** 11

## Accomplishments

- Measured current checked-in public sample artifact sizes before optimizing.
- Reduced detail derivative bounds from `1200x1600` to `960x1280`.
- Added publisher regressions for derivative bounds and sample byte reduction.
- Recorded sample reduction from `2,615,114` bytes to `1,777,658` bytes at `960x1276`.
- Added Caddy `Cache-Control` headers: `no-store` for admin routes, short-lived HTML/JSON, and moderate public assets/media caching.
- Documented Cloudflare/CDN as deferred for v1 with Full Strict TLS, Cache Rules, admin/API bypass, purge, and rollback prerequisites.
- Added post-Phase 6 runtime cleanup checklist for VM-local images, retired service absence, static release retention, failed candidates, route shape, and cache headers.
- Updated codebase maps and security review for optimized static delivery/cache posture.

## Task Commits

1. **Task 1: Measure and reduce generated public artifact size where safe** - `b09ef55` (`feat`)
2. **Task 2: Review CDN, cache headers, and origin privacy posture** - `1854235` (`docs`)
3. **Task 3: Clean deployed instance and codebase posture** - `d5ae25c` (`docs`)

## Files Created/Modified

- `controller/src/derivatives.rs` - Reduces detail derivative max dimensions to `960x1280`.
- `controller/tests/publisher.rs` - Adds derivative dimension and sample byte-reduction regressions.
- `controller/tests/caddy_static_routes.rs` - Covers public/admin Caddy cache header contract.
- `deploy/ansible/roles/autographs_deploy/files/Caddyfile` - Adds rollback-aware cache headers.
- `docs/dns-runbook.md` - Adds deferred Cloudflare/CDN decision and checklist.
- `docs/deployment-runbook.md` - Adds cache header behavior and post-Phase 6 cleanup checklist.
- `docs/static-runtime-runbook.md` - Records artifact size measurements and cache verification.
- `docs/security-review.md` - Marks CDN/cache/image-size optimization reviewed.
- `.planning/codebase/ARCHITECTURE.md` - Updates optimized static delivery posture.
- `.planning/codebase/INTEGRATIONS.md` - Updates Caddy cache/CDN integration notes.
- `.planning/codebase/TESTING.md` - Updates validation map for Caddy cache and cleanup posture.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Plan referenced a stale Caddy template path**
- **Found during:** Task 2
- **Issue:** The plan listed `deploy/ansible/roles/autographs_deploy/templates/Caddyfile.j2`, but the active deploy role copies `deploy/ansible/roles/autographs_deploy/files/Caddyfile`.
- **Fix:** Updated the active Caddyfile and added route-contract coverage in `controller/tests/caddy_static_routes.rs`.
- **Verification:** `cargo test --manifest-path controller/Cargo.toml --test caddy_static_routes -- --nocapture` and Ansible deploy syntax check passed.
- **Committed in:** `1854235`

---

**Total deviations:** 1 auto-fixed (Rule 2).
**Impact on plan:** The change kept the implementation aligned with the actual deploy role instead of editing a non-existent template path.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` passed.
- `cargo test --manifest-path controller/Cargo.toml --test publisher -- --nocapture` passed.
- `cargo test --manifest-path controller/Cargo.toml --test static_contract -- --nocapture` passed.
- `cargo test --manifest-path controller/Cargo.toml --test caddy_static_routes -- --nocapture` passed.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings` passed.
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook -i deploy/ansible/inventory/ci.ini deploy/ansible/playbooks/deploy.yml --syntax-check` passed.
- `rg -n "Cloudflare|CDN|Cache-Control|post-Phase 6|cleanup" docs .planning/codebase` passed.
- `! rg -n "Next.js runtime is active|/api/operator/catalog.*current|public accounts implemented|AI-assisted ingest is implemented" docs .planning/codebase` passed.

## Self-Check: PASSED

- Optimization preserved public `/media/...` paths and static privacy scans.
- No direct Object Storage, Oracle, admin session, image UUID, or unpublished catalog details were exposed.
- Cloudflare remains deferred with concrete prerequisites and rollback steps.
- Live VM cleanup is documented as operator-run only and was not claimed locally.

## Phase 6 Readiness

Phase 6 is complete. Realignment note: Phase 7 has since completed metadata taxonomy/public facets; the current roadmap places admin media/posture in Phase 8, taxonomy media cues in Phase 9, and advisory AI-assisted ingest in Phase 10.

---
*Phase: 06-admin-collection-workflow*
*Completed: 2026-07-02*
