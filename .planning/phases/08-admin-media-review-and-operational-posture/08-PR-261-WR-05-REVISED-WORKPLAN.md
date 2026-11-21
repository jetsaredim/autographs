---
phase: 08-admin-media-review-and-operational-posture
pr: 261
finding: WR-05
status: approved
depends_on:
  - 08-PR-261-REVIEW-ITER3.md
  - 08-PR-261-CONVERGENCE.md
files_modified:
  - controller/src/oracle_schema.rs
  - controller/db/updates/08-01-image-adjustments.sql
  - controller/tests/live_persistence_smoke.rs
---

# WR-05 Revised Implementation Plan

## Contract Decision

Treat `USER_CONSTRAINTS.SEARCH_CONDITION_VC` as untrusted schema input. Runtime preflight accepts only a restricted check-predicate grammar:

1. optional balanced outer parentheses;
2. the `event_type` identifier, case-insensitive and optionally double-quoted;
3. the `IN` keyword;
4. one parenthesized comma-separated list of SQL single-quoted string literals;
5. no trailing tokens, conjunctions, disjunctions, function calls, or secondary predicates; and
6. the decoded literal set equals `EditEventKind::ALL` exactly, with no missing, extra, or duplicate values.

This is intentionally narrower than general SQL. Unsupported but equivalent predicates fail closed and are remediated by the migration.

For migration state recognition, compare `SEARCH_CONDITION_VC` to the exact canonical predicate through a quote-aware normalizer. The normalizer may remove insignificant whitespace, identifier quotes, and normalize identifier/keyword case only while outside SQL string literals. Inside single-quoted literals it must preserve byte-for-byte case and content, understand doubled single quotes, and reject unterminated quoting. Do not lowercase the whole predicate and do not use independent `LIKE` tests. A formatting variant may be rebuilt; a widened predicate or a case-altered event value must never be accepted. Script-created temporary constraints therefore have a deterministic representation that can be recognized during interruption recovery.

## Coherent Implementation

### 1. Runtime exact-predicate parser

- Replace substring validation in `oracle_schema.rs` with a small, testable parser for the restricted grammar above.
- Parse SQL literals safely, including doubled single quotes, and reject malformed quoting.
- Compare the parsed values with the exact `EditEventKind::ALL` set and report missing, unsupported, or duplicate values using the existing redacted remediation message style.
- Keep the database lookup restricted to the enabled canonical named check constraint.

### 2. Migration exact-state recognizer

- Define one canonical normalized predicate string inside the PL/SQL block.
- Implement one small PL/SQL character scanner for state recognition that tracks whether it is inside a single-quoted literal. Remove whitespace/double quotes and lowercase syntax only outside literals; copy literal bytes and doubled single quotes exactly. Treat malformed quoting as non-current.
- Replace every `LIKE '%literal%'` state check for canonical and temporary constraints with strict normalized equality.
- Preserve the existing staged state machine: add/validate exact temporary constraint while another enabled constraint exists, then drop the old canonical and rename the exact temporary constraint.
- Preserve recoverability for old-only, canonical-plus-temporary, temporary-only, and canonical-new states.
- Treat weakened canonical or temporary predicates as non-current and converge them without dropping the last enabled constraint first.

### 3. Regression coverage

- Unit-test runtime acceptance of canonical text plus harmless case/whitespace/outer-parenthesis variations.
- Unit-test rejection of: missing literal, extra literal, duplicate literal, malformed literal, all nine values plus `OR 1=1`, and a second predicate.
- Source-contract test that migration recognition uses strict normalized equality and no per-literal `LIKE` classification.
- Extend the credentialed scratch-table migration fixture with weakened canonical and weakened temporary states containing all nine values plus `OR 1=1`; prove migration replaces them with the exact allowlist.
- Add canonical and temporary migration states with a case-altered mixed-case literal such as `metadataupdated`; prove neither is recognized as current and both are repaired to the byte-exact `metadataUpdated` value.
- Prove the resulting constraint accepts all nine supported values and rejects an unsupported value.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check`
- `cargo test --manifest-path controller/Cargo.toml --features production-persistence oracle_ -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke --no-run`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo clippy --manifest-path controller/Cargo.toml --features production-persistence -- -D warnings`
- `cargo clippy --manifest-path controller/Cargo.toml --features live-persistence --test live_persistence_smoke -- -D warnings`
- `git diff --check`
- Record separately whether the credentialed Oracle/OCI smoke was executed.

## Non-Goals

- Do not implement a general Oracle SQL parser.
- Do not broaden public APIs or media behavior.
- Do not change event names or make history decoding permissive.
- Do not start another sibling point-fix sequence; this is one shared contract change across preflight, migration recognition, and tests.

## Done

WR-05 is ready for final source review when runtime and migration recognition both prove an exact allowlist, widened predicates are covered as regressions, focused checks pass, and the fix is recorded as one coherent lineage-preserving change.
