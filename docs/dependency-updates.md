# Dependency Updates

Phase 4 selected Renovate as the repository dependency automation path. The goal is conservative visibility and review, not unattended production change.

## Workflow Permission Review

| Workflow | Permissions | Review |
|----------|-------------|--------|
| `.github/workflows/ci.yml` | `contents: read` | Least privilege for pull request validation. |
| `.github/workflows/deploy.yml` | `contents: read`, `packages: write` | Required to read repository contents and publish GHCR app/tools images. `actions: write` was removed because current deploy steps do not need Actions API mutation. |
| `.github/workflows/data-smoke.yml` | `contents: read` | Least privilege for manual VM smoke checks. |
| `.github/workflows/image-cleanup.yml` | `contents: read`, `packages: write` | Required to read cleanup scripts/playbooks and delete old GHCR package versions. |

## Action And Image Surfaces

Tracked by Renovate:

- npm and pnpm dependencies in the root and app manifests.
- GitHub Actions such as `actions/checkout`, `docker/*`, `hashicorp/setup-terraform`, and `dawidd6/action-ansible-playbook`.
- Docker images in `app/Dockerfile`, including Node base images and pinned Corepack pnpm usage.
- Terraform providers and modules in `infra/terraform`.
- Ansible Galaxy collections in `deploy/ansible/collections/requirements.yml`.
- The Caddy runtime image default in `deploy/ansible/roles/autographs_deploy/defaults/main.yml`.

Accepted current posture:

- Third-party actions are pinned to stable tags instead of immutable SHAs for readability and maintainability in this personal project.
- Major dependency updates require manual review through the Renovate dependency dashboard.
- Production deploy and cleanup changes must be reviewed with the same care as app code because they can affect the live VM and GHCR packages.

## Cleanup Reliability

The scheduled Image Cleanup workflow failed in GitHub run `26355096380` because Podman refused to delete a selected stale image ID that still had multiple old tags. The cleanup selector already preserves the active image, `latest`, protected tags, and the newest retained images. Runtime cleanup now removes selected stale IDs with `podman rmi --force` so multi-tag stale images do not fail the scheduled job.

## Review Policy

Use this checklist for Renovate PRs:

1. Read the Renovate release notes and package diff before merging.
2. For npm or pnpm updates, run `corepack pnpm --filter app lint`, `corepack pnpm --filter app typecheck`, `corepack pnpm --filter app test`, and `corepack pnpm --filter app build`.
3. For GitHub Actions updates, require the PR CI workflow checks to pass and inspect permission changes carefully.
4. For Terraform provider or module updates, run `terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff` and `terraform -chdir=infra/terraform validate`; review any plan output before applying.
5. For Docker or runtime image updates, confirm the app image builds and review whether the deployed VM needs a manual smoke check.
6. For Ansible collection or deployment-action updates, run Ansible syntax checks and prefer a manual deploy or data smoke check before treating the update as production-safe.

Manual smoke review is required when an update touches deployment behavior, Terraform runtime resources, Ansible roles, data smoke behavior, Object Storage media delivery, or GHCR image publication/cleanup. The manual `Data Smoke` workflow remains the operator-facing live Oracle/Object Storage proof.

## Lifecycle Notes

- Phase 5 must update this policy when real admin authentication, edit history, media cleanup, and operator-bridge retirement add new dependencies or validation gates.
- Phase 6 must update this policy when OCR/AI providers, prompt tooling, model SDKs, or image/text extraction dependencies are introduced.
- Major updates should stay separate and should not be merged only because automated checks pass.
