---
quick: 261025-e8u
plan: teach-renovate-to-update-the-terraform-version-in-resolve-runtime-ip
subsystem: dependency-automation
tags: [renovate, terraform, github-actions, regex-manager]
status: complete
requires:
  - existing-hashicorp-terraform-custom-manager
  - shared-resolve-runtime-ip-action
provides:
  - unified-renovate-terraform-pin-updates
affects:
  - github-workflows
  - resolve-runtime-ip-action
tech-stack:
  added: []
  patterns: [single-dependency-custom-regex-manager]
key-files:
  created: []
  modified:
    - renovate.json
decisions:
  - Keep workflow pins and the composite-action default in one hashicorp/terraform custom manager.
actuals:
  tokens: 234
  tasks: 1
  commits: 1
plan_head_before: cc6462e2dc72380a21b87282d90213a817cc436a
plan_head_after: cf2786107340eceedde0ac9ad1a58507ea9ed350
duration: 1min
completed: 2026-10-25
---

# Quick 261025-e8u: Renovate Terraform Composite-Action Coverage Summary

Renovate now updates the resolve-runtime-ip Terraform default together with the existing CI and deploy workflow pins through one dependency surface.

## Completed Work

- Extended the existing `hashicorp/terraform` custom regex manager to scan nested `action.yml` and `action.yaml` definitions under `.github/actions` while retaining workflow YAML coverage.
- Added an input-scoped matcher that captures only `inputs.terraform-version.default` from the composite action.
- Preserved the existing datasource, dependency name, workflow matcher, and `custom.regex` package-rule behavior.

## Verification

- `python3 -m json.tool renovate.json` — passed.
- The plan's Node manager contract check — passed; CI, deploy, and composite-action files each produced exactly one aligned `1.16.5` capture.
- `python3 scripts/validate-terraform-version-alignment.py` — passed.
- `git diff --check -- renovate.json` — passed.
- Scoped diff inspection confirmed that no workflow, action implementation, validator, or unrelated Renovate rule changed.

## Commit

- `cf27861` — `chore(renovate): update composite action Terraform pin`

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None - the configuration-only change introduces no endpoint, authentication, file-access, schema, or other trust-boundary surface.

## Self-Check: PASSED

The implementation commit exists, `renovate.json` is the only committed file changed by the task, and all planned verification commands pass.
