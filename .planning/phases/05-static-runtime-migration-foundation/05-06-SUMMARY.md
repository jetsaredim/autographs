---
phase: 05-static-runtime-migration-foundation
plan: 06
subsystem: runtime-deployment
tags: [rust, oracle, oci-object-storage, podman, caddy, ansible, github-actions]
requires:
  - phase: 05-04
    provides: validated static publisher and atomic release promotion
  - phase: 05-05
    provides: minimal static admin seed and publish shell
provides:
  - Deployable Oracle-enabled Rust controller image with OCI S3 media adapter
  - Ansible-managed private controller, shared static release volume, and staged Caddy routes
  - CI, deployment, and image-cleanup support for the third runtime image
affects: [05-07-live-proof, 06-admin-workflow]
tech-stack:
  added: [oracle-instant-client, rust-s3]
  patterns: [private-controller-container, staged-static-cutover, vm-local-controller-secrets]
key-files:
  created:
    - controller/Dockerfile
    - controller/src/oracle_catalog.rs
    - controller/src/oci_media.rs
    - controller/tests/caddy_static_routes.rs
    - deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2
    - deploy/ansible/roles/autographs_deploy/templates/autographs-static.volume.j2
    - deploy/ansible/roles/autographs_deploy/templates/controller.env.j2
  modified:
    - deploy/ansible/roles/autographs_deploy/files/Caddyfile
    - .github/workflows/ci.yml
    - .github/workflows/deploy.yml
    - .github/workflows/image-cleanup.yml
key-decisions:
  - "Keep the public hostname on the existing Next.js runtime until the explicit 05-07 live proof and cutover checkpoint; expose generated releases only through the localhost candidate listener for now."
  - "Build production Oracle and OCI S3 adapters into the controller image so the 05-07 smoke can prove the real source-of-truth path inside the OCI boundary."
  - "Keep controller persistence overrides in the VM-local protected controller.env file; GitHub Actions deploys code artifacts and configuration shape only."
requirements-completed: [STATIC-02, STATIC-04, STATIC-05, STATIC-06]
duration: 30m
completed: 2026-06-02
---

# Phase 05 Plan 06: Runtime Deployment Wiring Summary

**Deployable private Rust controller, shared static release storage, staged Caddy validation routes, and CI image integration**

## Accomplishments

- Added a multi-stage Rust controller image with Oracle Instant Client, wallet alias support through `TNS_ADMIN`, native Oracle catalog persistence, and OCI S3-compatible private media storage.
- Added Ansible-managed controller and static-volume quadlets. The controller remains private on the Podman network, mounts wallet/secrets/static storage, and receives production persistence overrides through a VM-local protected environment file.
- Added staged Caddy routes for `/admin`, `/admin/api/*`, old operator-route blocking, and a host-local static candidate preview. Public catalog traffic intentionally remains on Next.js until the live 05-07 checkpoint.
- Extended GitHub Actions and cleanup automation to validate, publish, deploy, retain, and prune the controller image without running catalog generation on GitHub-hosted workers.

## Verification

- `cargo test --manifest-path controller/Cargo.toml -- --nocapture`
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence`
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings`
- `cargo clippy --manifest-path controller/Cargo.toml --features production-persistence --all-targets -- -D warnings`
- `ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/data-smoke.yml deploy/ansible/playbooks/system-cleanup.yml`
- `ansible-lint deploy/ansible`
- `docker build --file controller/Dockerfile --tag localhost/autographs-controller:phase-05 .`
- Rebuilt controller container health probe returned `{"ok":true,"service":"autographs-controller"}`.
- `caddy validate --config /etc/caddy/Caddyfile`
- `git diff --check`

## Deviations

- The literal plan action described switching the public root to generated files during this plan. The implementation deliberately stages that switch: localhost candidate preview is live-ready, while the public hostname remains on Next.js until 05-07 proves Oracle, Object Storage, publishing, static browsing, and unpublish behavior on the VM.
- The production persistence adapters were added here because the live proof cannot validate the controller end to end while it is limited to local-memory adapters.

## Next Phase Readiness

- `05-07` can deploy the staged runtime, switch the protected VM-local controller environment to Oracle and OCI S3-compatible storage, run the live candidate smoke, and document the public cutover and legacy-runtime retirement criteria.
- The temporary OCI S3 customer key must remain on the VM/operator path and should be revoked after the live checkpoint.

---
*Phase: 05-static-runtime-migration-foundation*
*Completed: 2026-06-02*
