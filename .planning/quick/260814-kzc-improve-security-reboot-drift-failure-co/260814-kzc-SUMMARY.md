---
status: complete
quick_id: 260814-kzc
slug: improve-security-reboot-drift-failure-co
completed_at: "2026-08-14T19:34:00Z"
---

# Quick Task 260814-kzc Summary

Improved failed `approved-production-reboot` handling so pre-reboot validation failures leave actionable operator feedback on the issue.

## Completed

- Reboot drift validation now computes added and removed advisory IDs, writes a markdown failure context file, and writes a JSON refresh payload with the current OpenSCAP findings before failing closed.
- Failed-request cleanup now reads the context/payload, refreshes the scanner issue body and hidden metadata from current findings when reboot advisory drift is detected, resets the issue approval instruction to `approved-production-update`, and still removes the failed reboot label.
- Added fixture coverage for drift context, refresh payload generation, and cleanup-side refreshed issue rendering.

## Verification

- `python3 -m unittest scripts/test_security_patching_create_issue_tasks.py`
- `ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ansible-playbook deploy/ansible/playbooks/security-reboot-state-validate-test.yml`
- `ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ansible-playbook -vvv --syntax-check deploy/ansible/playbooks/security-reboot.yml deploy/ansible/playbooks/security-patch-cleanup.yml deploy/ansible/playbooks/security-reboot-state-validate-test.yml`
- `ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ansible-lint deploy/ansible/roles/security_patching/tasks/validate_reboot_state.yml deploy/ansible/roles/security_patching/tasks/cleanup_failed_request.yml deploy/ansible/playbooks/security-reboot-state-validate-test.yml .github/workflows/reboot-security-runtime.yml`
- Full local security-patching Ansible fixture sequence from CI.
