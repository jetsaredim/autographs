---
status: complete
quick_id: 260802-vx8
completed: 2026-08-03
---

# Quick Task 260802-vx8 Summary

Replaced the production security scan/apply design with OpenSCAP Oracle OVAL findings, advisory-ID approval metadata, and Ksplice-aware remediation.

## Changes

- Added runner-side `oscap-ssh` scanning against Oracle's OL10 ELSA OVAL XML and installed `openscap-scanner` on the runtime host through base deploy package defaults.
- Added `scripts/oracle_linux_oscap_results.py` to parse OpenSCAP result XML into ELSA advisory IDs, CVEs, affected package names, Oracle errata links, and Ksplice-aware advisory flags.
- Changed scanner issues to store approved `advisory_ids` instead of package NEVR specs and updated report/result templates for OpenSCAP findings and Ksplice/DNF signals.
- Reworked the apply path to drift-check advisory IDs, run `ksplice -y all upgrade`, re-scan, apply remaining approved advisories with `dnf -y upgrade-minimal --security --advisories=...`, then re-scan for closure.
- Updated fixtures, CI unit coverage, and the production security patching runbook for the new scanner and remediation contract.

## Verification

- `python3 -m unittest scripts/test_oracle_linux_oscap_results.py scripts/test_security_patching_create_issue_tasks.py scripts/test_oracle_linux_advisory_enrichment.py`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-patch-cleanup.yml deploy/ansible/playbooks/security-report-render-test.yml deploy/ansible/playbooks/security-request-metadata-validate-test.yml`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/roles/security_patching deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-patch-cleanup.yml deploy/ansible/playbooks/security-report-render-test.yml deploy/ansible/playbooks/security-request-metadata-validate-test.yml`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook deploy/ansible/playbooks/security-report-render-test.yml`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook deploy/ansible/playbooks/security-request-metadata-validate-test.yml`

## Notes

- Local `actionlint` was not available in this workspace (`actionlint: command not found`), so workflow YAML will still rely on the repository's CI actionlint job.
