---
phase: 08-admin-media-review-and-operational-posture
pr: 261
finding: WR-05
reviewed: 2026-10-01
status: revisions_required
reviewed_artifacts:
  - 08-PR-261-REVIEW-ITER3.md
  - 08-PR-261-CONVERGENCE.md
  - 08-PR-261-WR-05-REVISED-PLAN.md
---

# WR-05 Revised Plan Review

## Decision

**Revisions required. Do not resume coder work yet.** The reassessment records the required finding lineage, shared invariant, consumer and mutation inventory, assumption audit, failure matrix, revised requirements, and explicit resume criteria. The proposed implementation is coherent, preserves the staged constraint replacement states, uses a deliberately restricted parser rather than a general SQL parser, and includes non-vacuous parser and live migration regressions. However, its migration normalization rule still does not prove the exact event allowlist.

## Required change

**[BLOCKER] `scope` — Migration recognition must preserve case-sensitive event literal values while normalizing SQL identifiers and keywords.**

- **Evidence:** The revised plan requires migration recognition to compare a “lowercased/whitespace-stripped/identifier-quote-stripped” condition to the canonical predicate (revised plan, line 30). Event literals are case-sensitive strings: the canonical set includes `metadataUpdated`, `imageAdjustmentChanged`, `primaryImageChanged`, and `cleanupChanged` (migration lines 92–100; `EditEventKind::as_str` in `controller/src/catalog.rs`). Lowercasing the whole condition also lowercases these literals. For example, a constraint containing `'metadataupdated'` instead of `'metadataUpdated'` can normalize to the canonical text even though it rejects the event value produced by the Rust enum. That lets runtime migration-state recognition classify a wrong set as current, violating WR-05's exact allowlist invariant.
- **Required property:** The migration recognizer must be insensitive to harmless differences in SQL keyword/identifier casing and formatting while preserving the byte-exact decoded values of SQL string literals. It must reject a condition whose literal set differs in case, even if all other normalized text matches.
- **Example route:** Normalize outside quoted string literals only, or parse the restricted predicate and compare decoded literal values exactly. Add a migration recognition regression with a case-altered mixed-case event literal and prove it is repaired; retain the widened `OR 1=1` canonical and temporary recovery cases.

## Review evidence

- **Round-three issue:** `08-PR-261-REVIEW-ITER3.md` identifies WR-05 as the incomplete fix: the runtime and migration accept predicates with all supported literals plus a widening predicate. The revised plan addresses that specific failure at both boundaries.
- **Runtime preflight:** The proposed grammar permits only `event_type IN ('…', ...)`, rejects trailing predicates, handles doubled SQL quotes, rejects duplicates, and requires the decoded values to equal `EditEventKind::ALL` (revised plan, lines 19–39, 51–52). This is appropriately scoped and does not require a general SQL parser.
- **Migration recovery:** The plan requires exact temporary and canonical recognition and retains old-only, both, temporary-only, and canonical-new recovery states (lines 41–47). The live fixture extension for weakened canonical and temporary constraints containing all values plus `OR 1=1` is non-vacuous and checks both accepted supported values and rejected unsupported values (lines 53–55). The case-preservation gap above also needs coverage in this state-recognition path.
- **Convergence artifact:** The reassessment includes lineage classifications, invariant, producer/consumer/persistence/mutation/test inventory, assumption audit, failure matrix, revised requirements, and explicit evidence-based resume criteria (`08-PR-261-CONVERGENCE.md`, lines 17–69). Its pending evidence fields and `reassessment_required` status are correct until this review is incorporated and the plan is revised.
- **Test scope:** Unit parser cases and source-contract checks provide credential-free assertions; the scratch-table execution checks real Oracle DDL behavior when credentials are available. The plan also requires reporting credentialed smoke execution separately. No source or plan files were modified in this review.

## Resume condition

Revise the migration normalization contract so literal values remain case-sensitive, add the case-altered-literal regression to migration state recognition, and have an independent reviewer confirm the corrected rule and test. Then record this review and the revised-plan evidence in the convergence artifact before changing its status to `ready_to_resume`.
