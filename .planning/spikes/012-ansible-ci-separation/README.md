---
spike: "012"
idea: ansible-ci-separation
name: ansible-ci-separation
type: standard
validates: "Given the current CI workflow and two completed PR runs, when Ansible work is classified and timed, then a split can preserve every check while making fast and deep results separately visible."
verdict: PARTIAL
related: []
tags: [ci, ansible, github-actions, phase-08]
---

# Spike 012: Separate Ansible CI validation

Issue: [#298](https://github.com/jetsaredim/autographs/issues/298). Source: [PR #294](https://github.com/jetsaredim/autographs/pull/294) and `.github/workflows/ci.yml` at merge commit `43bdd290289ba1104d33bd931fa6b9757888ce3b`.

## What This Validates

The current `Validate Ansible` status hides a long publish integration suite and security scenario tests behind a syntax/lint name. This spike measures the work, inventories every existing invocation, and proposes a split. The proposal is unimplemented; the verdict is partial until CI proves the new statuses and coverage.

## How to Reproduce the Evidence

```bash
gh api repos/jetsaredim/autographs/actions/runs/40590195545/jobs --paginate \
  --jq '.jobs[] | select(.name=="Validate Ansible") | {started_at,completed_at,steps:[.steps[] | {name,started_at,completed_at}]}'
gh api repos/jetsaredim/autographs/actions/runs/40607453023/jobs --paginate \
  --jq '.jobs[] | select(.name=="Validate Ansible") | {started_at,completed_at,steps:[.steps[] | {name,started_at,completed_at}]}'
gh api repos/jetsaredim/autographs/actions/jobs/122492129400/logs > /tmp/ansible-job-1.log
gh api repos/jetsaredim/autographs/actions/jobs/122555333598/logs > /tmp/ansible-job-2.log
rg 'PLAY RECAP|\[group\]Run docker info|\[group\]Run ANSIBLE_CONFIG=.*ansible-lint' /tmp/ansible-job-{1,2}.log
```

The first `PLAY RECAP` ends `publish-freeze-deploy-test.yml`; the next play starts `security-report-render-test.yml`. The final recap ends `runtime-kernel-persistence-validate-test.yml`. GitHub's job-step API supplies step boundaries; the raw logs supply suite boundaries within the combined functional step.

## Timing Results

| PR #294 run | Tool install + collections | Syntax | Functional step | Publish suite within step | Remaining security/credentials/kernel scenarios | Lint | Whole job |
|---|---:|---:|---:|---:|---:|---:|---:|
| [40590195545](https://github.com/jetsaredim/autographs/actions/runs/40590195545) | 30s | 3s | 13m09s | about 11m05s | about 2m04s | 45s | 14m32s |
| [40607453023](https://github.com/jetsaredim/autographs/actions/runs/40607453023) | 18s | 2s | 13m24s | about 12m08s | about 1m15s | 23s | 14m11s |

Suite figures include process transition overhead and are rounded to seconds. They are observations from two successful runs, not performance guarantees. The publish suite consumes roughly 83–91% of functional time. Other PR runs on earlier code were much shorter, so estimates should use the recent post-merge shape and be revisited after the split.

## Proposed Layout

Keep a single `CI` workflow in `.github/workflows/ci.yml` with its existing `pull_request` and `workflow_dispatch` events. Replace `validate-ansible` with three independent, parallel jobs:

| Job ID / visible name | Responsibility | Expected duration from current evidence |
|---|---|---:|
| `ansible-static` / `Ansible syntax and lint` | Install Ansible/collections; syntax-check all 27 listed playbooks; lint `deploy/ansible/`. | about 45–90s, allowing setup and runner variation |
| `ansible-publish-integration` / `Ansible publish freeze/deploy integration` | Install Ansible/collections; check Docker; execute `publish-freeze-deploy-test.yml`. | about 11–13m plus setup; budget 20m timeout |
| `ansible-security-scenarios` / `Ansible security and runtime scenarios` | Install Ansible/collections; run all remaining 16 invocations as separately named steps, including the second `security-reboot-state` entry with `--start-at-task`. | about 1–3m plus setup; budget 10m timeout |

The security job name covers its security patching tests plus admin credential and runtime kernel checks. Each scenario gets its own step name, so the Actions page identifies the failing playbook and shows progress. The publish integration job gets a distinct status and timeout; its existing Ansible task log gives detail during its longer run. Do not use a matrix for the security scenarios: separate matrix check names complicate required-check policy and obscure the single suite result.

**Expected CI effect:** the fast Ansible result arrives in roughly a minute. The overall PR critical path remains governed by publish integration at roughly 12–14 minutes; this split is for ownership and visibility, not a claimed PR speedup. Separate installs add some runner time. Running the jobs concurrently can reduce the current sequential tail (security plus lint), subject to runner queue and load.

### Complete step map

| Current `Validate Ansible` step | Proposed destination |
|---|---|
| Checkout | Each of the three jobs |
| Install Ansible tooling | Each of the three jobs |
| Install Ansible collections | Each of the three jobs |
| Syntax check Ansible playbook (27 explicit paths, including the functional-test paths) | `ansible-static`, unchanged list |
| Test Ansible validation playbooks: `docker info`; `publish-freeze-deploy-test.yml` | `ansible-publish-integration` |
| Test Ansible validation playbooks: `security-report-render-test.yml` with its inventory; `security-request-metadata-validate-test.yml`; `security-create-issue-status-validate-test.yml`; `security-finding-classification-validate-test.yml`; `security-kernel-posture-validate-test.yml`; `security-post-result-status-validate-test.yml`; `security-post-result-refresh-validate-test.yml`; `security-target-scope-validate-test.yml` with its inventory; `security-update-reconciliation-validate-test.yml`; `security-reboot-preflight-validate-test.yml`; `security-reboot-kernel-selection-validate-test.yml`; `security-reboot-state-validate-test.yml` twice (full and `--start-at-task`); `security-reboot-result-validate-test.yml`; `deploy-admin-credentials-validate-test.yml` with empty credential env; `runtime-kernel-persistence-validate-test.yml` | `ansible-security-scenarios`, each invocation kept with the same inventory, flags, environment, and order |
| Lint Ansible playbook (`ansible-lint deploy/ansible/`) | `ansible-static`, unchanged scope |

The source has 17 functional `ansible-playbook` invocations: one publish and 16 others. Its syntax step lists 27 paths. These counts are migration checks, not a reason to remove an invocation.

## Trigger and Required-Check Policy

- Trigger all three jobs for every `pull_request`, including ordinary and Release PRs, and allow all three via the existing manual `workflow_dispatch` event. Do not add workflow or job path filters in the first split. A skipped job can leave required-check behavior ambiguous, and deployment/security fixtures depend on paths outside `deploy/ansible/`, including workflow, scripts, templates, and runtime contracts.
- Require all three named jobs for merging ordinary and Release PRs, alongside the repository's other mandatory CI checks. Treat each status as independently blocking; no `continue-on-error` or conditional skip. The existing `main` ruleset currently contains deletion and non-fast-forward rules but no required-status-check rule, and `gh pr checks 294 --required` reports none. Configuring required checks is a repository settings follow-up, not something this documentation commit can enforce.
- Keep `workflow_dispatch` as the manual deep-validation entry point. It runs the whole CI workflow today; a later optional input could target deep suites, but the initial migration should avoid conditionals that create skipped required checks.

## Migration and Verification Plan

1. In a separate follow-up PR, split the job into the three named jobs without changing playbook bodies. Preserve the complete step map above. Make each security invocation a distinct Actions step. Add explicit timeouts based on observed durations.
2. Run `actionlint` and compare the old and new command inventories mechanically: 27 syntax targets, one publish invocation, 16 scenario invocations, the same inventories/env/`--start-at-task`, and the same `ansible-lint` directory. Check for duplicate functional execution and missing commands.
3. Open the follow-up as a normal PR. Verify three successful, independently visible Ansible checks on that ordinary PR. Trigger `workflow_dispatch` on its branch and confirm all three jobs run. Confirm a representative controlled failure in a test branch makes the corresponding job fail and does not disappear behind a skipped condition.
4. Verify the same three checks run on a Release PR before merge. Configure the GitHub ruleset to require all three and verify they appear under `gh pr checks --required` for ordinary and Release PRs. Record the check names exactly as GitHub reports them after the PR run; avoid guessing names from YAML alone.
5. Compare the first several post-split run durations against the two baselines above. Reassess runner-minute cost and any path filtering only after the required-check contract has been proven. Do not merge this implementation into Wave 7's image cache PR.

## Investigation Trail

1. The initial job-level view showed that the combined functional step dominated the job. Two recent successful runs were selected to avoid drawing a conclusion from a single run.
2. Raw job logs separated publish from the remaining scenarios. Publish dominated both runs, but its duration varied by about a minute; security varied too. This supports estimates with ranges rather than a single target.
3. The workflow has no PR path filters, and the active `main` ruleset does not currently require checks. The proposed policy therefore distinguishes desired required checks from current repository settings.
4. Three jobs in the existing workflow preserve manual dispatch and avoid the risk of a second workflow silently skipping PRs. Separate jobs provide the needed status ownership without introducing cross-workflow coordination.

## Results

**PARTIAL:** The timing and command inventory are verified, and the proposed job layout is complete. The split, required-check settings, and ordinary/Release PR behavior require approval and a follow-up implementation PR.
