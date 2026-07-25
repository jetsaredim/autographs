---
status: complete
created: 2026-07-24
task: issue-165-semver-versioning
---

# Issue 165 Semver Versioning

Implement automated repository semver tracking and semver-based controller image deployment.

## Decisions

- Every merge to `main` should advance a repo `vM.m.O` version.
- Major bumps require explicit intent; minor bumps represent compatible feature/capability changes; patch bumps are the default.
- Controller image deployment is separate from bump type: only controller image-impacting changes build and deploy a matching GHCR semver image tag.
- Runtime/deploy config-only changes may redeploy the last deployed controller semver image.
- Repo-only changes update version/status and tag only.
- SHA image tags should be removed from the deploy path.
- README/status must expose possible divergence between the latest repo version and the deployed controller image version.

## Implementation

- Add release status metadata and generated README release status block.
- Add release automation scripts for bump classification, version computation, status updates, and GHCR semver cleanup behavior.
- Update deploy workflow and Docker Bake inputs to use semver image tags.
- Update image cleanup to preserve deployed semver images and not assume every repo tag has a GHCR image.
- Add non-secret runtime/publisher metadata for repo/controller versions.
- Update docs/tests for the new version model.

## Verification

- Script unit tests for version bump/classification/status updates and cleanup selection.
- Rust tests for additive publisher manifest version metadata.
- CI/workflow syntax and existing Rust/Ansible checks where feasible.
