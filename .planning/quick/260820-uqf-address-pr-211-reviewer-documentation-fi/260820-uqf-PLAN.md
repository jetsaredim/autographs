---
quick_id: 260820-uqf
status: complete
mode: quick
description: Address PR 211 reviewer documentation findings and prepare the pre-merge migrated controller live gate
must_haves:
  truths:
    - The protected smoke environment example includes both required Oracle passwords.
    - Spike 002 commands cannot be mistaken for runnable post-migration instructions.
    - Operators have a safe candidate-controller procedure that exercises the migrated controller before merge without replacing the live service.
  artifacts:
    - docs/static-runtime-runbook.md
    - .planning/spikes/002-oracledb-oci-persistence-smoke/README.md
---

# Quick Task 260820-uqf Plan

## Task 1: Correct smoke configuration and retire stale commands

Add explicit database and wallet password placeholders to the protected smoke environment example. Mark Spike 002's deleted test/image commands as a historical execution record and direct current operators to the surviving smoke runbook.

## Task 2: Document the pre-merge candidate gate

Add a procedure to load and start a candidate controller alongside the live controller on the private Podman network using copied wallet/secrets and the shared static volume. Point the established static-publish smoke at the candidate container, require cleanup, and record the result on the PR before merge.

## Verification

Run repository hygiene, documentation reference searches, diff checks, and confirm every referenced Dockerfile/test/env/container path exists at the PR head.
