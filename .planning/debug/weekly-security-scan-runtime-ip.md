---
status: resolved
trigger: weekly security scan workflow failed because it did not know which runtime host to SSH to
created: 2026-06-30
updated: 2026-06-30
---

## Symptoms

- Expected behavior: weekly production security scan resolves the current runtime VM address before running Ansible.
- Actual behavior: the scan inventory used `vars.VM_PUBLIC_IP` directly, so a missing or stale variable left Ansible without a usable SSH target.
- Related behavior: the apply-security-updates workflow used the same direct variable lookup and would fail similarly after a scan-created issue was approved.

## Current Focus

- hypothesis: security workflows missed the Terraform state output resolution already used by deploy/image cleanup workflows.
- test: inspect workflows for direct `VM_PUBLIC_IP` inventory use and compare against Terraform output resolution in existing workflows.
- expecting: security workflows should resolve `terraform output -raw runtime_public_ip`, fall back to `VM_PUBLIC_IP`, and fail with a clear message when neither exists.
- next_action: complete.

## Evidence

- 2026-06-30: `.github/workflows/image-cleanup.yml` resolves `runtime_public_ip` from Terraform state and only falls back to `VM_PUBLIC_IP`.
- 2026-06-30: `.github/workflows/weekly-security-scan.yml` used `production ansible_host=${{ vars.VM_PUBLIC_IP }}` without Terraform resolution.
- 2026-06-30: `.github/workflows/apply-security-updates.yml` used the same direct `vars.VM_PUBLIC_IP` inventory value.

## Resolution

- root_cause: production security workflows were added with a static VM variable assumption while other runtime workflows had moved to Terraform output resolution.
- fix: add a local `.github/actions/resolve-runtime-ip` composite action for Terraform setup, OCI key materialization, and `runtime_public_ip` output resolution; use it from image cleanup, weekly security scan, and apply-security-updates.
- verification: composite action and workflow YAML parsed successfully with Python/PyYAML, security patching and image cleanup Ansible playbooks passed syntax check, and `git diff --check` passed.
- files_changed: `.github/workflows/weekly-security-scan.yml`, `.github/workflows/apply-security-updates.yml`, `docs/security-patching.md`
