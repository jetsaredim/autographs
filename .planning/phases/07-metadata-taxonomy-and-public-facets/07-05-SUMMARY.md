---
phase: 07-metadata-taxonomy-and-public-facets
plan: "05"
subsystem: docs-security
tags: [rollout-docs, security-review, live-smoke, codebase-maps, taxonomy]

requires:
  - phase: 07-metadata-taxonomy-and-public-facets
    provides: 07-01 through 07-04 schema, persistence, admin workflow, and public schema version 2 facets
provides:
  - schema version 2 public artifact contract and taxonomy rollout runbook
  - Phase 7 ASVS L1 taxonomy security review
  - live static publish smoke assertions for schema version 2 facets
  - refreshed codebase maps identifying Phase 7 as metadata taxonomy/public facets and Phase 8 as advisory AI-assisted ingest
  - source coverage audit record for DATA-03, ADMIN-02, ADMIN-03, research findings, and D-07-01 through D-07-29
affects: [phase-08-ai-assisted-ingest, operator-rollout, public-static-contracts]

tech-stack:
  added: []
  patterns:
    - documentation-first rollout checklist for report review, PL/SQL review, optional SQL Developer application, deploy, full static publish, and verification
    - gated live smoke assertions for schema version 2 taxonomy facets and privacy terms

key-files:
  created:
    - .planning/phases/07-metadata-taxonomy-and-public-facets/07-05-SUMMARY.md
    - .planning/phases/07-metadata-taxonomy-and-public-facets/deferred-items.md
  modified:
    - docs/static-artifact-contract.md
    - docs/static-runtime-runbook.md
    - docs/controller-walkthrough.md
    - docs/deployment-runbook.md
    - docs/security-review.md
    - controller/tests/live_static_publish_smoke.rs
    - .planning/codebase/ARCHITECTURE.md
    - .planning/codebase/CONVENTIONS.md
    - .planning/codebase/STACK.md
    - .planning/codebase/STRUCTURE.md
    - .planning/codebase/TESTING.md
    - .planning/codebase/INTEGRATIONS.md

key-decisions:
  - "Phase 7 rollout requires reviewed migration report/PLSQL, optional SQL Developer application, deploy, full static publish, and admin/public verification."
  - "Category remains a temporary legacy database/reference field but is not a schema version 2 public facet."
  - "Phase 8, not Phase 7, is the pending advisory AI-assisted ingest phase."

patterns-established:
  - "Codebase maps must identify Phase 7 as implemented metadata taxonomy/public facets to prevent stale AI-ingest assumptions."
  - "Live static publish smoke remains credential-gated locally while asserting schema version 2 facets when enabled."

requirements-completed: [DATA-03, ADMIN-02, ADMIN-03]

duration: 11min
completed: 2026-07-10
---

# Phase 07-05: Rollout Docs, Security Review, and Codebase Map Summary

**Phase 7 taxonomy rollout documentation, ASVS L1 review, schema v2 live-smoke assertions, and refreshed codebase maps for Phase 8 handoff**

## Performance

- **Duration:** 11 min
- **Started:** 2026-07-10T01:51:09Z
- **Completed:** 2026-07-10T02:01:39Z
- **Tasks:** 3
- **Files modified:** 14

## Accomplishments

- Documented the schema version 2 public artifact contract, including signer/taxonomy fields and the explicit rule that category is not a schema version 2 public facet.
- Added the Phase 7 Taxonomy Rollout runbook path: migration report, PL/SQL review, optional SQL Developer application, deploy, full static publish, admin verification, public verification, and legacy-field cleanup planning.
- Recorded the Phase 7 Metadata Taxonomy Security Review with ASVS L1 framing and fixed/accepted dispositions for admin auth, signer merge repair, taxonomy validation, Oracle migration, Object Storage privacy, and fail-closed publishing.
- Updated the live static publish smoke so enabled runs validate schema version 2 collection/facets, signer/franchise/productLine/format/language/origin/role/tag facets, absence of category, public detail taxonomy values, derivative generation, and privacy-deny terms.
- Refreshed codebase maps so Phase 7 is the implemented metadata taxonomy/public facets layer and Phase 8 is pending advisory AI-assisted ingest.

## Task Commits

1. **Task 1: Document schema version 2, migration rollout, and legacy cleanup path** - `4151032` (docs)
2. **Task 2 RED: Add failing live smoke schema v2 assertions** - `bc09fb2` (test)
3. **Task 2 GREEN: Add Phase 7 security review and live smoke assertions** - `048a4f6` (docs)
4. **Task 3: Refresh codebase maps and record source coverage** - `9408de9` (docs)

## Files Created/Modified

- `docs/static-artifact-contract.md` - Documents schema version 2 fields, facet ids, and category deprecation from public facets.
- `docs/static-runtime-runbook.md` - Adds `## Phase 7 Taxonomy Rollout` and local schema version 2 admin payload guidance.
- `docs/controller-walkthrough.md` - Documents Phase 7 Identity/Classification/Details workflow, duplicate warnings, and signer merge repair.
- `docs/deployment-runbook.md` - Clarifies schema/taxonomy migration deploy order and required runtime full rebuild.
- `docs/security-review.md` - Adds Phase 7 ASVS L1 taxonomy security review.
- `controller/tests/live_static_publish_smoke.rs` - Adds schema version 2 facet/detail/privacy assertions while preserving the live credential gate.
- `.planning/codebase/*.md` - Refreshes architecture, conventions, stack, structure, testing, and integrations maps for the implemented Phase 7 taxonomy state.
- `.planning/phases/07-metadata-taxonomy-and-public-facets/deferred-items.md` - Records an out-of-scope clippy follow-up found during plan-level verification.

## Decisions Made

- Used the repository-established live smoke command shape because `live_static_publish_smoke` is an ignored test filter, not a Cargo feature.
- Kept legacy `signer`/`category` compatibility in live smoke create payloads because the current admin API still requires them, while public assertions verify schema version 2 taxonomy output.
- Deferred unrelated clippy cleanup in prior Phase 7 source files instead of modifying files outside the Plan 07-05 scope.

## Source Coverage Audit

The plan source audit remains covered:

- **GOAL:** Schema/domain, admin workflow, public facets, rollout docs, and codebase maps now describe modeling real items without duplicate records or overloaded public tags.
- **DATA-03:** Edit-history relevance for taxonomy/signer changes is covered by prior implementation summaries, security review, and codebase-map refresh.
- **ADMIN-02 / ADMIN-03:** Admin create/edit workflow with Identity, Classification, Details, image behavior, save/publish separation, duplicate warnings, and merge repair is documented.
- **Research findings:** Signer profiles, signer credits, public schema v2 facets, reviewed backfill artifacts, and no-new-package scope are all documented.
- **D-07-01 through D-07-29:** Rollout docs, public contract docs, security review, live smoke assertions, and codebase maps preserve the locked signer, taxonomy, migration, public, and admin decisions.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Normalized the live smoke verification command**
- **Found during:** Task 2 (live smoke verification)
- **Issue:** The plan's comma-separated feature form treated `live_static_publish_smoke` as a Cargo feature, but the crate only defines `live-persistence`; `live_static_publish_smoke` is the test filter used by existing docs.
- **Fix:** Ran the repository-established command shape: `cargo test --manifest-path controller/Cargo.toml --features live-persistence live_static_publish_smoke -- --ignored --nocapture`.
- **Files modified:** None.
- **Verification:** Command passed and skipped safely because `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE` was not set to `true`.
- **Committed in:** Procedural fix, no code commit.

---

**Total deviations:** 1 auto-fixed (1 blocking/procedural).
**Impact on plan:** No scope expansion or dependency change. The command normalization preserved the intended credential-gated smoke check.

## Issues Encountered

- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings` failed on pre-existing Phase 7 files outside this plan's file set: a derivable `Default` implementation in `controller/src/catalog.rs` and consecutive `str::replace` calls in `controller/src/taxonomy_migration.rs`. Per scope rules, these were not fixed in Plan 07-05 and are recorded in `deferred-items.md`.

## Verification

- `rg -n "schemaVersion 2|Category is not a schemaVersion 2 facet|productLine|origin|role" docs/static-artifact-contract.md` - passed.
- `rg -n "Phase 7 Taxonomy Rollout|SQL Developer|full static publish|taxonomy-backfill-mapping.json" docs/static-runtime-runbook.md` - passed.
- `rg -n "Phase 7 Metadata Taxonomy Security Review|ASVS L1|Object Storage identifiers|Oracle migration|fail-closed" docs/security-review.md` - passed.
- `! rg -n "HIGH: unmitigated" docs/security-review.md` - passed.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence live_static_publish_smoke -- --ignored --nocapture` - passed with the live smoke safely skipped because `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE` was not set to `true`.
- `! rg -n "Phase 7 AI-assisted ingest|Phase 7 advisory AI" .planning/codebase` - passed.
- `rg -n "Phase 7.*metadata taxonomy|schema version 2|signer profile|Phase 8.*AI-assisted ingest" .planning/codebase` - passed.
- `cargo fmt --manifest-path controller/Cargo.toml --check` - passed.
- `cargo test --manifest-path controller/Cargo.toml` - passed.
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` - passed.
- `node --check controller/static-admin/admin.js` - passed.
- `node --check controller/static-public/assets/browse.js` - passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings` - failed on deferred out-of-scope issues listed above.

## Known Stubs

None - stub scan found no TODO/FIXME/placeholder or hardcoded empty UI data paths in the Plan 07-05 files.

## Threat Flags

None - this plan updated documentation, codebase maps, and an ignored credential-gated live smoke. The security-relevant rollout, public artifact, Oracle migration, Object Storage privacy, and fail-closed surfaces are covered by the plan threat model and security review.

## User Setup Required

None for local closeout. Operator-run live static publish proof still requires real Oracle, private Object Storage, deployed controller/Caddy preview, and `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true`.

## Next Phase Readiness

Phase 7 is documented as complete from schema through public facets and rollout guidance. Phase 8 can start advisory AI-assisted ingest against stable signer profiles, item signer credits, taxonomy fields, schema version 2 public facets, and the manual admin workflow.

## Self-Check: PASSED

- Key files exist: all 12 plan-listed files, `07-05-SUMMARY.md`, and `deferred-items.md`.
- Task commits exist: `4151032`, `bc09fb2`, `048a4f6`, and `9408de9`.
- Required task acceptance greps passed.
- Plan-level verification passed except the documented out-of-scope clippy warnings.
- No accidental file deletions were detected after task commits.

---
*Phase: 07-metadata-taxonomy-and-public-facets*
*Completed: 2026-07-10*
