---
status: implementation_complete
issue: 286
commit: d1df901
---

# Terraform CLI Renovate grouping

- Restricted the custom regex manager to the composite-action default, leaving workflow install pins with the built-in `github-actions` manager.
- Added a shared `hashicorp/terraform` group across `terraform`, `github-actions`, and `custom.regex` managers, with no update-type restriction.
- Added `extractVersion` for workflow Terraform versions so GitHub release tags are written without `v`.

## Verification

- Renovate 44.181.6 local `--dry-run=extract` found both Terraform `required_version` constraints, both workflow pins, and the composite-action default. All five extracted dependencies use `packageName: hashicorp/terraform`.
- Renovate config validator passed.
- `python3 scripts/validate-terraform-version-alignment.py` passed on the unchanged 1.16.5 pins and constraints.
- `git diff --check` passed.

## Follow-up after merge

The repository configuration must reach `main` before Renovate can regenerate its open branches. Use Dependency Dashboard #69 to request rebases or another run, then verify the dedicated Terraform PR contains all five locations and the routine PR retains unrelated updates. Run the alignment validator and Terraform root/tenancy checks on those regenerated PRs. Leave all resulting PRs unmerged for manual review.
