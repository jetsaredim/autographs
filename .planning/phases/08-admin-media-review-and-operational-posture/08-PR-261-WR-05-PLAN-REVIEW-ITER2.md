---
phase: 08-admin-media-review-and-operational-posture
pr: 261
finding: WR-05
reviewed: 2026-10-01
status: approved
reviewed_artifacts:
  - 08-PR-261-REVIEW-ITER3.md
  - 08-PR-261-CONVERGENCE.md
  - 08-PR-261-WR-05-REVISED-PLAN.md
---

# WR-05 Revised Plan Review — Iteration 2

## Decision

**Approved.** The corrected plan now describes a coherent contract that proves exact inclusion and exclusion of edit-event values at both runtime preflight and migration state recognition. It preserves case-sensitive string literal bytes during normalization, rejects unsupported predicate syntax, and has non-vacuous regressions for widened and wrong-case canonical and temporary constraints. No general SQL parser is needed.

## Evidence

- **Runtime exactness:** The restricted grammar permits only an `event_type IN (...)` predicate, with optional balanced outer parentheses and the supported identifier spelling variations. The parser decodes SQL literals and compares the resulting values against `EditEventKind::ALL`, rejecting missing, extra, or duplicate values and all trailing predicates. Therefore all supported values must be present exactly and any unsupported/case-altered value fails preflight (revised plan, Contract Decision and Runtime exact-predicate parser).
- **Migration exactness and case preservation:** The quote-aware PL/SQL scanner normalizes whitespace, identifier quotes, and syntax case only outside single-quoted literals. It preserves literal bytes and case, recognizes doubled single quotes, and classifies malformed quoting as non-current. Strict normalized equality against the canonical predicate consequently cannot treat `'metadataupdated'` as `'metadataUpdated'`, and additional predicates such as `OR 1=1` cannot match the canonical representation (revised plan, Contract Decision and Migration exact-state recognizer).
- **Recovery and regressions:** The plan retains old-only, canonical-plus-temporary, temporary-only, and canonical-new state handling while requiring a valid enabled constraint to remain through staged replacement. It adds live scratch-table cases for widened and wrong-case canonical and temporary predicates, then verifies all supported values are accepted and an unsupported value is rejected. Unit parser tests and migration source-contract checks cover credential-free behavior (revised plan, Migration exact-state recognizer and Regression coverage).
- **Scope:** The parser and scanner are deliberately narrow; equivalent but unrecognized syntax is safely rebuilt by migration. The plan keeps the change to one shared contract across preflight, migration recognition, and regressions.
- **Convergence reassessment:** The convergence artifact contains the required finding lineage, shared invariant, consumer and mutation inventory, assumption audit, failure matrix, revised plan requirements, and explicit resume criteria. Its status remains `reassessment_required` and evidence fields remain pending in the copy reviewed; the plan review evidence must be recorded there before coder work resumes (`08-PR-261-CONVERGENCE.md`, frontmatter and Resume Criteria).

## Gate status

The revised plan passes this independent plan-review gate. This approval is not a status change to the convergence artifact: record both the revised-plan and this review evidence in its frontmatter, then apply its stated resume criteria before assigning coder work. No source or plan files were modified in this review; no tests were run.
