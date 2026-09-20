---
quick_id: 260920-gz7
status: complete
completed: 2026-09-20
implementation_commit: 1bd3796
---

# Quick Task 260920-gz7 Summary

Production deployment and security patching now converge on an explicit UEK-only kernel posture without creating a patch/reboot loop.

## Delivered

- Deployment verifies both the running and default kernels are UEK before kernel package mutation.
- Deployment removes the managed RHCK boot/development packages, preserving shared `kernel-headers` and `kernel-tools*` packages, and persists exact DNF exclusions without discarding existing exclusions.
- Deployment removes stale boot entries only when the kernel image is missing and always preserves rescue entries.
- Security scans independently inventory installed RHCK packages and classify them as `configure` before considering DNF update or reboot evidence.
- `configure` issues and result comments contain no approval label and direct the operator through deployment convergence followed by a fresh scan.
- UEK remains the only kernel family eligible for the separately approved reboot cleanup path.

## Verification

- Full Ansible syntax-check set passed.
- Full Ansible validation-playbook set passed, including new RHCK drift, mixed-host precedence, issue rendering, and UEK-only reboot fixtures.
- `ansible-lint deploy/ansible/` passed at the production profile with zero findings.
- `python3 -m unittest scripts/test_security_patching_create_issue_tasks.py` passed all 14 tests.
- `git diff --check` passed.

## Commit

- `1bd3796` — `fix(patching): enforce UEK-only kernel posture`
