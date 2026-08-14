# Quick Task 260814-g3u Summary: Add Approved Production Reboot Workflow

## Outcome

Added a separately approved production reboot workflow for security findings that remain after package updates have already been applied. The workflow validates the scanner issue, performs a fresh pre-reboot OpenSCAP drift check, verifies DNF has no advisory-scoped package work left, rejects non-kernel-family findings, reboots only hosts with approved findings, waits for Autographs health, removes old installonly kernel packages, re-runs OpenSCAP, then refreshes or closes the same GitHub issue.

## Changes

- Added `.github/workflows/reboot-security-runtime.yml` and `deploy/ansible/playbooks/security-reboot.yml`.
- Added reboot role tasks for pre-reboot drift validation, DNF no-op/package-family gating, reboot/health/installonly cleanup, and post-reboot issue refresh/closure.
- Added the `approved-production-reboot` managed label and shared request/failure message defaults.
- Added reboot result comment rendering with kernel before/after, installonly cleanup status, and remaining finding counts.
- Added Ansible fixtures and Python assertions for the reboot state guard and post-reboot issue behavior.
- Updated CI, CODEOWNERS, and `docs/security-patching.md`.

## Verification

- `python3 -m unittest scripts/test_oracle_linux_oscap_results.py scripts/test_security_patching_create_issue_tasks.py scripts/test_oracle_linux_advisory_enrichment.py`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-reboot.yml deploy/ansible/playbooks/security-patch-cleanup.yml deploy/ansible/playbooks/security-report-render-test.yml deploy/ansible/playbooks/security-request-metadata-validate-test.yml deploy/ansible/playbooks/security-create-issue-status-validate-test.yml deploy/ansible/playbooks/security-post-result-status-validate-test.yml deploy/ansible/playbooks/security-post-result-refresh-validate-test.yml deploy/ansible/playbooks/security-reboot-state-validate-test.yml deploy/ansible/playbooks/security-reboot-result-validate-test.yml`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible`
- Fixture batch covering report rendering, metadata parsing, issue status handling, post-update refresh, reboot drift validation, and post-reboot refresh/clean-close behavior.

Local `actionlint` was not available, so workflow lint remains covered by CI.
