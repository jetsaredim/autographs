# Production security update workflow

This runbook describes the production OS security update scanner and one-click approval workflow.

## Overview

The automation is split into two GitHub Actions workflows:

- `Weekly Security Scan` runs on a weekly schedule or manually and invokes `deploy/ansible/playbooks/security-scan.yml`.
- `Apply Security Updates` runs when the `approved-production-update` label is added to a scanner-created issue and invokes `deploy/ansible/playbooks/security-patch.yml`.

The workflows are intentionally thin wrappers around Ansible. Runtime host behavior, GitHub issue rendering, approval validation, drift checks, and update application live in the `security_patching` Ansible role.

## Approval model

The scanner creates an issue when production hosts report available security updates from `dnf updateinfo`.

To approve the proposed update set, apply this label to the issue:

```text
approved-production-update
```

The apply workflow validates that:

1. the actor is listed in `.github/production-patch-approvers.yml`,
2. the issue is open,
3. the issue has the scanner label `security-patching`,
4. the issue has the approval label,
5. the issue contains the scanner metadata block,
6. the target group matches the workflow target, and
7. the fresh pre-apply scan exactly matches the package specs embedded in the issue.

If the package set has drifted, the workflow refuses to apply updates. Run the scanner again to generate a fresh issue.

## Host inventory

Both workflows use the same host alias to avoid exposing the raw production IP address in GitHub issues:

```ini
[runtime]
production ansible_host=<VM_PUBLIC_IP> ansible_user=<DEPLOY_SSH_USER or opc>
```

The workflows expect `VM_PUBLIC_IP` to exist as a repository variable unless the inventory is later replaced by a Terraform-output or OCI-inventory step.

## Update behavior

The apply playbook uses `ansible.builtin.dnf` with:

- `security: true`
- `update_only: true`
- `state: latest`
- the exact package specs captured by the scanner issue

The workflow runs hosts serially and re-scans after applying updates. It removes the approval label after the run starts, comments the result back to the issue, and closes the issue only when the post-update scan has no remaining findings.

## Control-plane files

The sensitive control-plane files are CODEOWNED:

- `.github/workflows/weekly-security-scan.yml`
- `.github/workflows/apply-security-updates.yml`
- `.github/production-patch-approvers.yml`
- `deploy/ansible/playbooks/security-scan.yml`
- `deploy/ansible/playbooks/security-patch.yml`
- `deploy/ansible/roles/security_patching/`

CODEOWNERS only requests ownership by default. Require CODEOWNER review through branch protection if this should be enforced before merging future changes.
