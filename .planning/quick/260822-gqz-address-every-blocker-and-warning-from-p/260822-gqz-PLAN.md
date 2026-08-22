---
quick_id: 260822-gqz
status: complete
mode: quick
description: Address every blocker and warning from PR 213 counter-review, add regression tests, update the spike evidence and plan, and prepare for clean re-review
must_haves:
  truths:
    - The selected Vault boundary accurately accounts for persistent swap and core-dump exposure and requires production gates before cutover.
    - The SQL checker rejects repeated and out-of-order numeric bind labels.
    - The Vault cutover has an executable rollback contract for a pre-Vault controller image.
    - Repository-to-VM comparison uses authoritative contract declarations rather than incidental mentions.
    - Inventory output creation is exclusive, no-follow, and mode 0600.
    - Persistent-secret detection covers passwords, tokens, private keys, wallets, secret keys, and API keys while allowing secret OCID references.
  artifacts:
    - .planning/spikes/003-ecosystem-inventory/inventory.py
    - .planning/spikes/003-ecosystem-inventory/test_inventory.py
    - .planning/spikes/004-configuration-secret-boundary/README.md
    - .planning/spikes/007-enforceable-cleanup-plan/quality_contract.py
    - .planning/spikes/007-enforceable-cleanup-plan/test_quality_contract.py
    - .planning/spikes/007-enforceable-cleanup-plan/CLEANUP-PLAN.md
---

# Quick Task 260822-gqz Plan

## Task 1: Correct the secret-memory and rollback boundaries

Narrow the current off-disk claim to application-managed persistence, document
the production swap and core-dump risks, and make disabled core dumps plus a
verified non-persistent swap strategy cutover gates. Define a tested rollback
helper/process that rematerializes the exact legacy env and wallet inputs from
retained Vault versions before starting the previous immutable image, and do not
retire that compatibility path until rollback has succeeded.

## Task 2: Harden inventory collection and comparison

Derive repository contract keys only from authoritative example, deployment,
workflow, and Terraform roles while reporting incidental mentions separately.
Write inventory artifacts with exclusive, no-follow, mode-0600 semantics and
cover documentation-only keys and symlink pre-creation with regression tests.

## Task 3: Harden enforceable quality rules

Require numeric SQL bind labels to appear exactly as `:1..:N` in occurrence
order. Expand persistent-secret classification to the inventory vocabulary,
explicitly allow secret OCID references, and add fixtures for every reviewed
secret form.

## Task 4: Regenerate, verify, and re-review

Regenerate the checked-in repository and quality reports, run all spike suites
and repository validation, commit and push the fixes, reply on PR #213 with the
mapping from findings to fixes, and request a clean counter-review.
