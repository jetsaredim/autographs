---
status: complete
quick_id: 260802-jm4
completed: 2026-08-02
---

# Quick Task 260802-jm4 Summary

Hardened the apply-side production security update approval parser after workflow run 30748961696 failed before touching the runtime VM.

## Changes

- Moved scanner metadata extraction into `extract_issue_metadata.yml` and avoided the folded `set_fact` form that returned an empty match list under the runner Ansible path.
- Added marker/body-count diagnostics to the metadata assertion so future failures explain whether the issue body was missing markers or parser extraction failed.
- Added `security-request-metadata-validate-test.yml` and CI wiring so the approval metadata path is exercised without mutating GitHub issues.
- Updated the report render fixture extraction to use the same resilient metadata regex shape.

## Verification

- `python3 -m unittest scripts/test_release_version.py scripts/test_cleanup_ghcr_images.py scripts/test_oracle_linux_advisory_enrichment.py scripts/test_security_patching_create_issue_tasks.py`
- `ansible-playbook --syntax-check` for security scan, patch, cleanup, report-render, and metadata validation playbooks
- `ansible-playbook deploy/ansible/playbooks/security-request-metadata-validate-test.yml`
- `ansible-playbook -i deploy/ansible/test-fixtures/security-report-inventory.yml deploy/ansible/playbooks/security-report-render-test.yml`
- `ansible-lint deploy/ansible/`
- Ansible 2.21.2 fixture runs for both report rendering and metadata validation
- Read-only Ansible 2.21.2 parse of live issue #198 confirmed scan `security-scan-30748694006` and 264 package specs
