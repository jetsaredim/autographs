---
gsd_task_version: 1.0
task: remove-obsolete-tenancy-split-doc
created_at: "2026-06-13T01:55:20Z"
status: complete
---

# Quick Task Plan: Remove Obsolete Tenancy Split Doc

## Goal

Remove the historical Terraform tenancy split migration runbook now that the
split-state migration is complete and no operator should follow it for current
bootstrap work.

## Scope

- Delete `docs/terraform-tenancy-split.md`.
- Remove active-doc references to the historical split migration.
- Refresh nearby stale Phase 5 wording found during the docs audit.

## Verification

- `rg` confirms no active references to `terraform-tenancy-split.md`.
- `git diff --check`.
