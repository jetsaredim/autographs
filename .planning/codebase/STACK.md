# Technology Stack

**Analysis Date:** 2026-06-19

## Languages

**Primary**
- Rust for the private controller, static publisher, persistence adapters,
  media handling, routes, and tests.
- Markdown for planning, runbooks, and repository documentation.
- HCL for Terraform infrastructure.
- YAML for GitHub Actions and Ansible deployment/maintenance automation.
- HTML, CSS, and JavaScript for generated/static public and admin surfaces.
- Jinja templates for Ansible-managed runtime files and security patching
  issue/comment bodies.

**Historical**
- The former TypeScript/Next.js app was retired during the static runtime
  migration. Do not treat `app/`, root pnpm workspace commands, or Next.js
  route files as current implementation surfaces.

## Runtime

- Public runtime: Caddy serves generated static releases.
- Private runtime: Rust controller container runs admin/API and publishing
  behavior behind Caddy/private routes.
- Persistence: Oracle Autonomous Database for catalog metadata.
- Media: private OCI Object Storage originals plus generated public-safe
  derivatives.
- Container runtime: Podman quadlets managed by Ansible on the OCI VM.
- Image registry: GHCR controller image published by GitHub Actions.

## Frameworks and Tooling

- Rust standard toolchain: `cargo fmt`, `cargo test`, `cargo check`,
  `cargo clippy`, and image builds.
- Controller features include `production-persistence` for Oracle/OCI-backed
  production checks.
- Terraform manages OCI infrastructure and state-backed runtime resources.
- Ansible manages VM configuration, Caddy/controller quadlets, deployment,
  image cleanup, and production security patching.
- Renovate tracks maintained dependency surfaces with conservative review.

## Key Dependencies and Integrations

- Oracle Autonomous Database Free for catalog metadata.
- OCI Object Storage for private autograph media.
- Caddy as the public HTTP(S) edge and static-file server.
- Podman as the OCI VM container runtime.
- GHCR as the controller image registry.
- GitHub Actions for PR validation, image build/publish, deploy, cleanup, and
  production security patching workflows.

## Configuration

- `.env.example` documents local/controller reference variables.
- `.github/.env.github.example` documents GitHub Actions secret/variable
  expectations.
- `docs/configuration-contract.md` documents the committed configuration and
  secret contract.
- Ansible renders controller/Caddy runtime files from
  `deploy/ansible/roles/autographs_deploy/`.
- Secrets such as Oracle wallet material, ADB password, OCI private key,
  operator/admin tokens, GHCR token, deploy SSH key, and GitHub tokens must
  stay in GitHub/environment/operator secret stores.

## Validation

- PR CI checks Rust formatting, tests, production-persistence compile, build,
  clippy, controller image build, Dockerfile linting, Terraform formatting and
  validation, workflow linting, secret scanning, and Ansible syntax/lint.
- Live Oracle/Object Storage proof remains an operator-run smoke path because
  real secrets and tenancy state are required.
- Production security patching playbooks are syntax/lint covered and documented
  in `docs/security-patching.md`.

## Project Maturity

- Phases 1-4 are complete.
- Phase 5 plans 05-01 through 05-06 are done, and the static runtime
  migration foundation is implemented in code and operator docs.
- Phase 5 05-07 live static publish proof and closure summary remain pending
  before closing the phase.
- Phase 6 polished admin workflow remains pending.
- Phase 7 advisory AI-assisted ingest remains pending.

## Practical Guidance

- Treat `controller/`, `deploy/ansible/`, `infra/terraform/`, `.github/workflows/`,
  and `docs/` as current implementation surfaces.
- Treat historical Next.js references as old phase evidence unless a document
  explicitly marks them current.
- Do not re-scaffold the retired Node/Next.js app or pnpm workspace.
- Keep production security patching changes under the same review standard as
  deploy/runtime changes because they can affect the live VM.

---

*Stack analysis refreshed: 2026-06-19 after Phase 5 static runtime implementation and PR 129 production security patching merge*
