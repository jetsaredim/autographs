---
status: resolved
trigger: v0.2.0 was published and recorded as deployed even though every production deployment step was skipped
created: 2026-09-06
updated: 2026-09-06
---

## Symptoms

- Expected behavior: a controller-changing release runs Terraform, Ansible deployment, and health verification before recording production status or publishing the GitHub Release.
- Actual behavior: retry run `34070822703` completed successfully and published `v0.2.0`, while all production mutation and health steps were skipped.
- Error messages: the manifest step logged a jq compile error for `.impact != \"repo-only\"`, but the step still concluded successfully.
- Timeline: observed immediately after the `v0.2.0` retry on 2026-09-06.
- Reproduction: reconcile a non-repository-only manifest with the escaped jq expression nested inside `echo` command substitution.

## Current Focus

- hypothesis: invalid jq quoting is masked by the successful outer `echo`, leaving an empty mutation output; downstream conditions skip deployment while unconditional status/publication steps still run.
- test: calculate and validate the mutation output before writing it, require deployment-health evidence through a completion gate, and add structural regressions for both boundaries.
- expecting: an invalid or missing mutation classification fails the job, and a mutation-required release cannot record status or publish without successful health verification.
- next_action: complete.

## Evidence

- 2026-09-06: run `34070822703` logged a jq compile error in `Reconcile release manifest`, but that step concluded successfully.
- 2026-09-06: Setup Terraform, digest recheck, Terraform apply, full deployment, and deployment health were all skipped.
- 2026-09-06: commit `8157b41` nevertheless recorded `v0.2.0` as deployed, and the workflow published the GitHub Release.
- 2026-09-06: live `/admin/api/health` still reports controller `v0.1.4`, proving `v0.2.0` was not deployed.

## Resolution

- root_cause: the mutation classification used invalid escaped jq quotes inside an `echo` command substitution; jq failed, the outer echo returned success with an empty output, deployment conditions evaluated false, and unconditional status/publication steps still ran.
- fix: calculate and validate the mutation decision before emitting outputs, require exact release metadata from deployment health, add an explicit completion gate, and condition status/publication on that gate.
- verification: all 81 repository automation tests passed, repository hygiene validation passed, workflow YAML parsed successfully, and `git diff --check` passed. GitHub CI remains the authoritative actionlint gate.
- files_changed: `.github/workflows/deploy.yml`, `scripts/test_release_workflow.py`, `scripts/test-fixtures/release-workflow/cases.json`
