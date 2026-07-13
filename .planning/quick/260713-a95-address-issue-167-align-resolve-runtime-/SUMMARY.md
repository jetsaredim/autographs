---
status: complete
completed: 2026-07-13
issue: 167
---

# Summary

Updated the shared `resolve-runtime-ip` action default Terraform version from `1.15.7` to `1.15.8`, matching the production Terraform root lower bound.

Added `scripts/validate-terraform-version-alignment.py` and wired it into the CI workflow checks so future drift between `infra/terraform/versions.tf` and the shared action default fails in PR validation.

## Verification

- `python3 scripts/validate-terraform-version-alignment.py`
- `python3 -m py_compile scripts/validate-terraform-version-alignment.py`

`actionlint` was not installed locally; CI still runs it through `raven-actions/actionlint@v2`.
