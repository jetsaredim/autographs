---
status: resolved
trigger: production security patching workflows leave the scanner issue stale or misleading after advisory drift and after updates leave only reboot-path findings
created: 2026-09-15
updated: 2026-09-16T00:00:00-04:00
---

## Symptoms

- Expected behavior: a fresh OpenSCAP scan that differs from the approved advisory set applies no packages, refreshes or closes the same issue from the authoritative current findings, removes the consumed approval, posts an actionable status comment, and completes successfully when reconciliation succeeds. Remaining findings should direct the operator to normal update approval, approved reboot cleanup, investigation, or closure according to their actual state.
- Actual behavior: update-path drift fails at an assertion before preserving refresh data, cleanup posts a generic failure comment, and the Actions run is forced red. After a partial successful update, the issue and comment always request `approved-production-update`, even when the remaining kernel/installonly findings belong on the approved reboot path.
- Error messages: issue #246 run 35035697374 failed with `Current security update set for production does not match the approved scanner issue`, then posted only `Production security update workflow did not complete successfully` and told the operator to rerun the scanner or reapply approval.
- Timeline: reproduced on 2026-09-15 after the approved set changed between issue generation and update execution; related reboot drift refresh behavior already exists but resets next action to the update label.
- Reproduction: approve a scanner issue, allow the live OpenSCAP advisory set to drift, then apply `approved-production-update`; separately, apply updates that leave only kernel/installonly findings for which advisory-scoped DNF is a no-op.

## Current Focus

reasoning_checkpoint:
  hypothesis: "Issue state becomes stale or misleading because scan/apply/reboot paths each infer their own transition: update drift aborts before exposing current facts, clean scanner runs do not reconcile an existing issue, and result renderers default all non-empty findings to the triggering/update label."
  confirming_evidence:
    - "Live issue #246 failed at the pre-DNF drift assertion and retained stale metadata; source order proves no package mutation or reconciliation facts occur before that assertion."
    - "Complete source traces show clean create_issue runs skip the existing-issue query, while result templates unconditionally recommend the current approval label for any remaining findings."
    - "All 16 Python tests and seven Ansible validation playbooks pass unchanged, proving these observed paths are absent from the regression surface."
  falsification_test: "This hypothesis would be false if a focused fixture could show current drift facts reach successful post_result reconciliation, a clean scan closes an existing issue, or reboot-only findings select approved-production-reboot in the current code."
  fix_rationale: "Compute an explicit close/update/reboot/investigate action from each authoritative complete scan, preserve it through drift and remediation, reconcile issue body/labels/state with idempotent PATCH operations, and serialize all issue writers; this repairs the transition source rather than masking failed runs."
  blind_spots: "Local tests cannot prove live Oracle/DNF output wording or GitHub API behavior, so classification treats unrecognized probe output as investigate and operator verification remains required after self-verification."
hypothesis: confirmed fragmented issue-state transition logic
test: implement focused fixtures for drift/clean reconciliation and four-way next-action classification, then rerun the full security-patching validation surface.
expecting: drift skips mutation and succeeds through post_result; complete scans deterministically select close/update/reboot/investigate; all writers share one lock; normalized DNF advisory IDs retain Oracle's authoritative release-suffixed links.
next_action: complete.

## Evidence

- 2026-09-15: `.planning/debug/knowledge-base.md` does not exist, so there is no known-pattern candidate to test first.
- 2026-09-15: neither `.codex/skills/` nor `.agents/skills/` contains a project `SKILL.md`; only repository conventions and GSD debugger rules apply.
- 2026-09-15: `patch.yml` performs `Refuse to apply if security update set drifted` before setting any pre/post-update or cleanup refresh facts, so update drift cannot reach `post_result.yml` and exposes no authoritative scan payload to workflow cleanup.
- 2026-09-15: `create_issue.yml` stops after a debug message when a complete scan is clean; it only queries open scanner issues inside `when: findings > 0`, leaving a previously open issue stale.
- 2026-09-15: `post_result.yml`, `post_reboot_result.yml`, and `security-update-result.md.j2` refresh with `security_patching_issue_labels` and instruct reuse of the triggering approval label without classifying package eligibility or DNF applicability.
- 2026-09-15: weekly scan uses concurrency group `production-security-scan` while update and reboot use `production-security-updates`, permitting scanner and maintenance issue writes to overlap.
- 2026-09-15: `ADVISORY_RE = r"\bELSA-\d{4}-\d+\b"` extracts `ELSA-YYYY-NNNNN` even when Oracle's reference is `ELSA-YYYY-NNNNN-0`; the parser then synthesizes the truncated errata URL instead of preserving `ref_url`.
- 2026-09-15: baseline Python validation passed all 16 existing parser/security-patching contract tests; none covered update drift reconciliation, stale clean closure, four-way next-action classification, shared concurrency, or release-suffixed errata IDs.
- 2026-09-15: baseline Ansible validation passed all seven security-patching fixture playbooks (with expected rescued fake-GitHub failures), confirming the defect is not detected by the current playbook suite.
- 2026-09-15: live issue #246 retained stale approved metadata after update drift because `patch.yml` asserted before producing a refresh payload.
- 2026-09-15: the later successful apply reduced 12 findings to four kernel advisories, refreshed the issue, and incorrectly requested `approved-production-update` again.
- 2026-09-15: `cleanup_failed_request.yml` accepts only `reboot_advisory_drift` refresh payloads; update drift has no equivalent path.
- 2026-09-15: `security-update-result.md.j2` always recommends the current update approval label when findings remain.
- 2026-09-15: scanner, update, and reboot workflows do not share a single production patching concurrency group.
- 2026-09-15: the OpenSCAP parser normalizes Oracle errata references to `ELSA-YYYY-NNNNN` and produces links that omit Oracle's trailing `-0` release suffix.
- 2026-09-16: takeover review kept normalized advisory IDs stable for DNF while preserving Oracle's full `ref_url`, preventing the errata-link repair from changing the approved package selector.
- 2026-09-16: aggregate preflight fixture proved one drifted host disables package mutation for the entire target group before the serial apply play begins.
- 2026-09-16: all 84 repository automation tests, 11 Ansible validation playbooks, Ansible syntax checks, production-profile Ansible lint, repository hygiene, Terraform version alignment, workflow YAML parsing, and `git diff --check` passed locally.

## Resolution

- root_cause: Scan findings had no authoritative state transition. Update drift asserted before publishing current scan facts, clean scans skipped existing issue reconciliation, remaining findings inherited a hard-coded approval label, and scanner writes could race update/reboot writes. Oracle errata links also discarded the authoritative release-suffixed reference URL.
- fix: Added a shared close/update/reboot/investigate classifier, aggregate all-host preflight, non-mutating drift reconciliation, desired-state issue PATCH operations, clean-scan closure, accurate update/reboot/investigation comments, one shared workflow concurrency group, authoritative Oracle errata URLs, regression fixtures, and updated operator/planning documentation. Ksplice remains report-only.
- verification: 84 Python automation tests passed; all 11 Ansible validation playbooks passed; all affected Ansible playbooks passed syntax checks; ansible-lint passed 58 files with the production profile; repository hygiene, Terraform version alignment, workflow YAML parsing, and git diff checks passed. GitHub CI remains authoritative for actionlint and live API behavior.
- files_changed: `.github/workflows/{weekly-security-scan,apply-security-updates,reboot-security-runtime,ci}.yml`, `deploy/ansible/playbooks/security-*.yml`, `deploy/ansible/roles/security_patching/`, `scripts/oracle_linux_oscap_results.py`, related Python tests, `docs/security-patching.md`, and `.planning/STATE.md`.
