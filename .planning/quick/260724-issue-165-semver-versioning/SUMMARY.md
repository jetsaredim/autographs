---
status: complete
completed: 2026-07-24
task: issue-165-semver-versioning
---

# Issue 165 Semver Versioning Summary

Implemented automated repo semver tracking, semver-tagged controller image deployment, release status reporting, and semver-aware image cleanup.

## Changes

- Added `.release-status.json` and a generated README release status block.
- Added `scripts/release-version.py` with bump classification, version computation, README/status updates, and GHCR semver tag preflight.
- Updated deploy workflow to prepare a repo version/tag first, build/deploy semver-tagged controller images only for controller image-impacting changes, redeploy runtime config with the last deployed controller version when needed, and skip repo-only deploys.
- Replaced SHA image tags in Docker Bake with `RELEASE_VERSION` tags plus OCI labels.
- Added runtime release metadata env vars and surfaced them through admin health/status plus generated static `manifest.json`.
- Updated image cleanup to preserve the deployed semver image and newest semver-tagged images without assuming every repo `v*` tag has a GHCR image.
- Updated operator/static artifact docs for the new version model.

## Verification

- `python3 scripts/test_release_version.py`
- `python3 scripts/test_cleanup_ghcr_images.py`
- `python3 -m py_compile scripts/release-version.py scripts/cleanup-ghcr-images.py`
- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo test --manifest-path controller/Cargo.toml`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`
- `python3 scripts/validate-terraform-version-alignment.py`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/system-cleanup.yml`
- Parsed deploy, CI, and image-cleanup workflow YAML with PyYAML.

Local `actionlint` is not installed, so full GitHub Actions semantic linting remains for CI.
