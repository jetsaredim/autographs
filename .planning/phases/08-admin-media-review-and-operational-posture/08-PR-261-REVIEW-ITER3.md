---
phase: 08-admin-media-review-and-operational-posture
reviewed: 2026-10-01T01:16:51Z
depth: deep
files_reviewed: 7
files_reviewed_list:
  - controller/db/schema.sql
  - controller/db/updates/08-01-image-adjustments.sql
  - controller/src/oracle_schema.rs
  - controller/src/oracle_catalog.rs
  - controller/src/catalog.rs
  - controller/tests/admin_workflow.rs
  - controller/tests/live_persistence_smoke.rs
findings:
  critical: 0
  warning: 1
  info: 0
  total: 1
status: issues_found
---

# Phase 08 Plan 05: Code Review Report — Iteration 3 Convergence Checkpoint

**Reviewed:** 2026-10-01T01:16:51Z
**Depth:** deep
**Files Reviewed:** 7
**Status:** issues_found

## Summary

The branch tip reviewed is `c30da5b8ba81a8568d114d7b96500e25f5fbf61f`, which includes the iteration-two fixes through `668a828`. The migration retains an enabled allowed-values constraint through its staged DDL transitions, all seven item-returning Oracle mutations validate their readback before commit, adjustment CLOB save/reset/replacement paths agree, and the live smoke now performs explicit verified cleanup with `Drop` recovery diagnostics.

Convergence failed at round three. One Warning remains: event constraint preflight and migration state detection check for the presence of every supported literal, but do not reject a permissive predicate that also admits unsupported values. The current partial-constraint test covers missing supported literals only. Per the project convergence guard, this finding requires a durable reassessment and a revised, reviewed contract before coder work resumes; it is not a request for another point-fix round.

## Narrative Findings (AI reviewer)

## Warnings

### WR-05: Event constraint checks accept predicates that admit unsupported event types

**Classification:** WARNING
**Lineage classification:** **incomplete fix** — remaining gap in WR-03's runtime event-contract repair; the same literal-presence assumption also affects WR-02 migration convergence.
**File:** `controller/src/oracle_schema.rs:210-220`; related migration state detection: `controller/db/updates/08-01-image-adjustments.sql:26-58`

**Shared invariant:** The enabled Oracle edit-event constraint must admit every `EditEventKind::ALL` value and reject every unsupported value. Runtime readiness and migration convergence must establish both halves of that contract because history decoding rejects unknown values.

**Inspected boundary:** `EditEventKind::ALL` and `as_str` define the nine producers' values (`controller/src/catalog.rs:395-419`); `insert_edit_event` stores those strings; `load_history` decodes persisted values through `EditEventKind::from_str` (`controller/src/oracle_catalog.rs:1937` and `controller/src/catalog.rs:423-438`); publish-boundary and pending-change readers consume the resulting event rows. Canonical DDL enumerates the nine values (`controller/db/schema.sql:158-168`). Migration and runtime preflight both inspect the named enabled constraint, while tests cover the canonical expression and a partial expression with only two values (`controller/src/oracle_schema.rs:515-580`).

**Issue:** `validate_edit_event_constraint_condition` returns success whenever the condition contains each quoted supported literal. The migration's `canonical_new_count` and `temporary_new_count` use the same substring criterion. A widened expression such as `event_type in ('created', ... 'cleanupChanged') or 1 = 1` contains all nine literals but permits arbitrary event types. Runtime preflight would mark that schema ready, and a rerun would classify the weak canonical constraint as already migrated. An unsupported row can then be committed and later make `load_history` fail with `unsupported catalog edit event kind`, breaking history reads for that item. The current partial-constraint regression proves rejection of omitted values, but not rejection of a permissive condition containing all values.

**Fix:** Make the accepted predicate contract prove both inclusion and exclusion of event kinds. Validate the normalized allowed-values expression against the expected exact set, or use an equivalently strict schema contract that rejects additional disjuncts/predicates. Apply the same definition in migration state detection so a widened canonical or temporary constraint is repaired/converged rather than accepted. Add preflight and migration-state regressions for a condition containing all nine values plus a tautological widening (`or 1 = 1`), and confirm it is rejected/repaired while all nine supported values still pass and an unsupported value fails.

## Mandatory Convergence Checkpoint — Reassessment Required

**Convergence result:** Failed. Do not start another point-fix round until assumptions and the implementation plan are revised and reviewed, with both evidence references recorded in the convergence artifact before status changes to `ready_to_resume`.

### Finding lineage

| Finding | Round-three disposition | Evidence |
|---|---|---|
| CR-01 — Oracle rejected `imageAdjustmentChanged` | Resolved for the canonical successful path; incomplete strictness remains in WR-05 | All nine literals appear in canonical and migration DDL, and preflight iterates `EditEventKind::ALL`; predicates that additionally admit unsupported values are not rejected. |
| CR-02 — mutation could commit before fallible reload | Resolved | Create, update, attach, set-primary, remove, replace, and adjustment-update use `load_item_before_commit`; its reload-error path rolls back and preserves rollback errors. |
| WR-01 — tests did not execute Oracle persistence | Resolved, credentialed proof not run here | The ignored live fixture exercises save, reset, wrong IDs, replacement clearing, malformed-sibling rollback, persisted JSON, timestamp/history deltas, and cleanup. |
| WR-02 — migration could drop the only constraint | Resolved for represented interruption states; same predicate strictness gap remains in WR-05 | The migration stages the expanded constraint before dropping the old one and handles old-only, both, temporary-only, and canonical-new fixtures. Its `*_new_count` queries can mistake a widened predicate for the intended exact contract. |
| WR-03 — runtime preflight checked only two values | Partially resolved; WR-05 is the remaining incomplete-fix finding | Runtime now iterates all nine enum values, but presence-only checking does not prove unsupported values are rejected. The omission regression does not test this case. |
| WR-04 — live smoke could silently leak fixtures | Resolved | Success awaits OCI deletion/absence and Oracle deletion/commit/zero-row verification; fallback reports item, object key, step, and error. |
| WR-05 — permissive constraint passes readiness | Open, incomplete fix | `oracle_schema.rs:210-220`, migration `08-01-image-adjustments.sql:26-58`, decoder `catalog.rs:423-438`, and history loader `oracle_catalog.rs:1937`. |

### Shared invariants and consumer inventory

- **Event producers and storage:** `EditEventKind::ALL` / `as_str` define the nine event values; Oracle mutations and `record_event` store them through `insert_edit_event`. The DB constraint is the persistence boundary.
- **Consumers:** History decoding maps persisted values through `FromStr` and returns an error for unsupported strings. Pending-change counts and publish-boundary snapshots consume event rows by ID; the DB does not silently normalize unknown kinds.
- **Constraint authorities:** `schema.sql` is the fresh-install definition. The Phase 8 migration upgrades existing schemas and infers migration state from `USER_CONSTRAINTS.SEARCH_CONDITION_VC`. `verify_edit_event_constraint` is the runtime readiness gate. Their shared assumption is that literal inclusion alone identifies the intended allowlist.
- **Adjustment mutations:** Save binds serialized typed JSON; reset binds SQL NULL; image reads deserialize the CLOB; replacement explicitly clears adjustment metadata. The live fixture observes these results and exact history deltas.
- **Mutation boundaries:** The seven item-returning Oracle mutations read the complete item before commit and roll back on readback failure. The malformed sibling-image smoke asserts target CLOB, parent timestamp, and history count remain unchanged.
- **Cleanup and recovery:** The successful smoke path awaits cleanup and verifies OCI absence plus zero Oracle item/image/history rows. `Drop` remains best-effort and prints stable recovery details on failure.
- **Tests:** The new production preflight test rejects a constraint missing `created`; the migration live fixture exercises four represented states and rejects `unexpectedEvent` against the generated migration result. Neither test supplies a weakened predicate that contains all supported literals.

### Assumption audit

- The assumption that the text form of a check constraint containing every known literal is equivalent to the intended exact allowlist is false: SQL predicates can include additional disjunctions or other widening terms.
- The history decoder's strict enum mapping makes this schema drift observable as a runtime read failure, not just a cosmetic schema difference.
- The live migration fixture validates the intended DDL output and recovery states, but does not currently validate repair behavior for an already-present weakened constraint.
- Credentialed Oracle/OCI smoke execution was not performed in this review environment; source inspection and the recorded prior compile/check evidence do not demonstrate live behavior.

### Failure matrix

| Database constraint state | Current behavior | Risk |
|---|---|---|
| Correct exact nine-value allowlist | Preflight passes; migration treats it as current | Expected behavior |
| Missing one supported literal | Preflight rejects; migration replaces/converges | Covered failure path |
| All nine literals plus `OR 1=1` | Preflight passes; migration may classify canonical/temp as current | Unsupported events can be stored; later history parsing fails |
| Weakened temporary predicate containing all literals | Migration may rename it as canonical | Same event-integrity failure after upgrade |

### Revised plan requirements and resume criteria

1. Define one explicit contract for the exact allowed event set and use it in runtime preflight and migration state recognition; literal inclusion alone is insufficient.
2. Decide how Oracle's normalized `SEARCH_CONDITION_VC` will be validated robustly, including supported formatting/parentheses, while rejecting added disjunctions and other widening predicates.
3. Add a credential-free regression for permissive constraint rejection and migration text/state checks, plus a live scratch-table case proving the migration converges a weakened canonical or temporary predicate to the exact allowlist.
4. Re-run the focused schema and live-persistence test compilation/check set after the contract change; record separately whether credentialed Oracle/OCI execution occurred.
5. Resume coder work only after the revised assumptions and plan requirements are reviewed and both evidence references are recorded in the durable convergence artifact. The next review must preserve WR-05 lineage and may close convergence only if no Critical or Warning remains.

## Verification and review limits

- Confirmed source tip with `git rev-parse HEAD`: `c30da5b8ba81a8568d114d7b96500e25f5fbf61f`; branch status showed it matches `origin/gsd/phase-08-admin-media-review-and-operational-posture`.
- Inspected the seven configured files and the prior round-one/round-two review and fix artifacts. No source files were modified.
- No tests were run during this review. The ignored, credentialed Oracle/OCI smoke was not executed in this environment.

---

_Reviewed: 2026-10-01T01:16:51Z_
_Reviewer: the agent (gsd-code-reviewer)_
_Depth: deep_
