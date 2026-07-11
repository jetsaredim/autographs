---
phase: 07-metadata-taxonomy-and-public-facets
verified: 2026-07-10T21:31:00Z
status: passed
score: 7/7 must-haves verified
---

# Phase 07: Metadata Taxonomy and Public Facets Verification Report

**Phase Goal:** The collection owner can model real autograph items without duplicating records or overloading tags, and anonymous visitors can browse by natural collector facets generated from that richer metadata.
**Verified:** 2026-07-10T21:31:00Z
**Status:** passed

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Catalog metadata supports multiple signers on one physical item and reusable signer records across items. | VERIFIED | `controller/db/schema.sql`, `controller/db/updates/07-01-taxonomy-schema.sql`, `controller/src/catalog.rs`, `controller/src/oracle_catalog.rs`, and `07-01-SUMMARY.md`/`07-02-SUMMARY.md` show signer profiles plus item signer credits. Tests cover duplicate-credit rejection, stale signer ID failure, merge repair, and non-mutating signer reuse. |
| 2 | Signer records can carry optional enrichment links without requiring links for every signer. | VERIFIED | `controller/src/catalog.rs`, `controller/src/routes/admin_items.rs`, `controller/static-admin/admin.js`, and `controller/src/publisher.rs` support optional Wikipedia/IMDb profile fields, profile route updates, and detail-only icon links. Validation rejects unsafe hosts and existing-signer item-editor link fields are disabled. |
| 3 | Item metadata separates display title/character-style names from signer records and supports first-class format, origin, franchise, product line, signer role, language, set, and loose tags. | VERIFIED | Domain contracts and admin/static DTOs in `controller/src/catalog.rs`, `controller/src/contracts.rs`, `controller/src/routes/admin_items.rs`, and `controller/static-admin/admin.js`; public schema v2 fixtures and tests cover these fields. |
| 4 | Current tag/category drift can be reviewed and backfilled into the new taxonomy. | VERIFIED | `controller/src/taxonomy_migration.rs`, `controller/src/bin/taxonomy_backfill.rs`, `taxonomy-backfill-mapping.json`, `07-02-taxonomy-backfill-review.sql`, and `07-03-taxonomy-backfill-apply.sql` provide reviewed/report/apply artifacts mapping `custom` to Custom origin and `Tr`/`Tra`/`Trading Card` to Trading Card format. |
| 5 | Public static JSON and pages expose natural collector facets while preserving privacy and fail-closed generated-static behavior. | VERIFIED | `controller/src/contracts.rs`, `controller/src/publisher.rs`, `controller/static-public/assets/browse.js`, `controller/static-public/data/facets.json`, and `controller/tests/publisher.rs`/`static_contract.rs` cover schema v2, signer/franchise/productLine/format/language/origin/role/tag facets, detail metadata, fingerprinted derivatives, and privacy validation. |
| 6 | Admin create/edit workflow remains efficient with controlled taxonomy fields, repeatable signer rows, optional profile links, duplicate warnings, and advanced loose tags outside the primary path. | VERIFIED | `controller/static-admin/index.html`, `controller/static-admin/admin.js`, `controller/static-admin/admin.css`, `controller/src/routes/admin_items.rs`, `controller/src/catalog_admin.rs`, and `07-03-SUMMARY.md` implement the Identity/Classification/Details workflow, signer suggestions, duplicate warnings, merge repair, and taxonomy payload saving. |
| 7 | Migration, backfill review, static rebuild docs, security/privacy review, and codebase handoff are complete before AI-assisted ingest. | VERIFIED | `docs/static-artifact-contract.md`, `docs/static-runtime-runbook.md`, `docs/deployment-runbook.md`, `docs/security-review.md`, `controller/tests/live_static_publish_smoke.rs`, `.planning/codebase/*.md`, and `07-05-SUMMARY.md` record rollout order, schema v2 smoke checks, security review, and Phase 8 handoff. |

**Score:** 7/7 truths verified

## Requirements Coverage

| Requirement | Status | Evidence |
|-------------|--------|----------|
| DATA-03 | SATISFIED | Taxonomy, signer profile, signer merge, and profile link edits record metadata history in repository tests and route behavior. |
| ADMIN-02 | SATISFIED | Admin create workflow accepts signer rows, controlled taxonomy fields, image workflow, and save/publish separation. |
| ADMIN-03 | SATISFIED | Admin edit workflow supports existing item taxonomy/signer updates, signer profile repair, image behavior, and history. |

**Coverage:** 3/3 phase requirements satisfied

## Key Link Verification

| From | To | Via | Status |
|------|----|----|--------|
| Admin taxonomy form | Catalog repositories | `/admin/api/items`, signer profile, merge, and taxonomy suggestion routes | VERIFIED |
| Catalog repositories | Oracle schema | signer/taxonomy tables, joins, schema preflight, and additive update scripts | VERIFIED |
| Legacy tags/category | Phase 7 taxonomy | mapping JSON, review SQL, generated PL/SQL, and backfill CLI | VERIFIED |
| Catalog metadata | Public artifacts | publisher schema v2 projection, facets JSON, collection JSON, detail pages, and browse filters | VERIFIED |
| Public profile links | Safe external destinations | HTTPS host validation plus detail-only icon rendering | VERIFIED |

## Automated Checks

- `node --check controller/static-admin/admin.js`
- `node --check controller/static-public/assets/browse.js`
- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo test --manifest-path controller/Cargo.toml --test static_contract checked_in_static_fixtures_are_schema_v2_taxonomy_examples`
- `cargo test --manifest-path controller/Cargo.toml --test static_admin static_admin_signer_payload_uses_row_scoped_fields_and_item_role_only`
- `cargo test --manifest-path controller/Cargo.toml`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `node /home/jgreenwa/.codex/gsd-core/bin/gsd-tools.cjs query verify.schema-drift 07` returned `drift_detected: false`

## Review And Drift Gates

- Code review status: clean in `07-REVIEW.md`.
- Regression gate: full controller test suite passed, including prior Phase 5 static runtime, Caddy route, static admin, seed, publisher, and static contract coverage.
- Codebase drift gate: non-blocking warning remains for structural map freshness outside Phase 7 closeout: `.dockerignore`, `.env.example`, `.gitignore`, `AGENTS.md`, `renovate.json`, and `scripts/cleanup-ghcr-images.py`.

## Human Verification Required

None required for phase closeout. The live static publish smoke remains credential-gated and documented for operator-run production verification after applying reviewed schema/backfill changes.

## Gaps Summary

No gaps found. Phase 7 goal achieved and ready for PR review.

## Verification Metadata

**Verification approach:** Inline verifier pass after the verifier subagent hit the session usage limit. Evidence was checked against the ROADMAP Phase 7 success criteria, all five plan summaries, requirement IDs, review/fix artifacts, schema drift output, and fresh automated test results.
**Must-haves source:** ROADMAP Phase 7 success criteria.
**Automated checks:** Rust controller tests, static admin/public JavaScript checks, formatting, production-persistence check, static fixture regressions, schema drift check, and clean code review artifact.
**Human checks required:** 0 for local closeout.
**Security gate:** Phase 7 ASVS L1 taxonomy review is recorded in `docs/security-review.md`; no open threat flags in plan summaries.

---
*Verified: 2026-07-10T21:31:00Z*
*Verifier: Codex inline verifier after subagent usage-limit interruption*
