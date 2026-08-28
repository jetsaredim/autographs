---
quick_id: 260825-ocd
status: in_progress
description: Address PR 222 review findings for Vault runtime secrets
created: 2026-08-25
---

# Quick Task 260825-ocd: Address PR 222 Review Findings for Vault Runtime Secrets

## Goal

Resolve both actionable review findings on PR 222, preserve the fixes and review cycle on the PR, and re-run deep review against the updated head.

## Tasks

1. Replace startup environment mutation with typed Vault secret overrides that flow through `ControllerConfig` and the Oracle connection lifecycle; add regression coverage proving resolved values reach configuration without `std::env::set_var`.
2. Make the production Ansible deploy role hash-only for admin authentication, remove generated/plaintext fallback behavior, and add contract coverage for the missing-Vault-coordinate failure path.
3. Run repository validation, commit and push the fixes, reply directly to both inline review comments, and trigger a fresh deep review agent against the new PR head.

## Verification

- `cargo fmt --check`
- `cargo test --features production-persistence`
- `cargo clippy --all-targets --features production-persistence -- -D warnings`
- Terraform tenancy format and validation
- Ansible deploy syntax check
- `git diff --check`
- Direct GitHub replies on both original inline findings
- Fresh reviewer comment(s) or clean-review confirmation on PR 222
