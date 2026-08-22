---
spike: 007
name: enforceable-cleanup-plan
type: standard
validates: "Given the inventory, secret-boundary, Rust consistency, and efficiency findings, when proposed standards are exercised by a fixture-tested contract checker and mapped to bounded Phase 8 work, then broad cleanup can prevent new debt without turning media waves into an ecosystem rewrite."
verdict: VALIDATED
related: [003, 004, 005, 006]
tags: [style-guide, ci, cleanup, phase-08, rust, sql, configuration, operations]
---

# Spike 007: Enforceable Cleanup Plan

## What This Validates

The broad ecosystem concerns can be expressed as semantic rules, enforced at
different levels, and delivered in bounded PRs. Consistency does not require
making every SQL string, bind style, error, or module look identical.

## Prototype

`quality_contract.py` exercises deterministic rules against the current
controller and fixture tests:

- direct production env reads outside `config.rs`;
- production `todo!`, `unimplemented!`, and `dbg!` macros;
- numeric SQL bind placeholders that repeat or do not appear exactly as
  `:1..:N` in left-to-right occurrence order;
- password, password-hash, token, private-key, wallet, secret-key, and API-key
  values rendered into persistent deploy env templates, while explicitly
  allowing non-secret Vault secret OCIDs.

The prototype is intentionally report-only. It identifies existing debt before
the cleanup establishes a baseline; it is not wired into CI from `.planning`.
The production version belongs beside `scripts/validate_repo_hygiene.py` and
must add persistent-secret-template checks using the configuration inventory.

## How to Run

```bash
python3 .planning/spikes/007-enforceable-cleanup-plan/test_quality_contract.py
python3 .planning/spikes/007-enforceable-cleanup-plan/quality_contract.py \
  --root . \
  --output .planning/spikes/007-enforceable-cleanup-plan/quality-contract-report.json
```

## Results

- Fixture tests distinguish allowed named repeated binds from forbidden
  repeated and out-of-order numeric positional binds, including the prior
  `sync_signer_credits` regression shape.
- The current report exposes four distributed configuration-read owners and
  five persistent secret env sinks as expected from Spikes 003-005. It finds no
  unsafe numeric SQL bind order or production placeholder/debug macros.
- The style guide assigns Block, Warn, Measure, or Document treatment so CI
  does not turn architectural preferences into noisy failures.
- The cleanup plan places feature-supporting changes in Waves 5-7, closeout
  evidence in Wave 8, and risky runtime/Vault changes in independent posture
  PRs.

## Verdict

**VALIDATED.** A broad cleanup is feasible without pausing Phase 8 for a blanket
rewrite. The next implementation is C1, followed by runtime configuration/key
removal and a live Vault proof. The VM portion of the inventory and the live
Vault retrieval remain separate partial evidence in Spikes 003 and 004; this
planning verdict does not claim those production checks have run.
