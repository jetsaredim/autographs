# Technology Stack

**Analysis Date:** 2026-05-24

## Languages

**Primary:**
- TypeScript for the `Next.js` application, service layer, scripts, and tests.
- Markdown for planning, operator runbooks, and repository documentation.
- HCL for Terraform infrastructure.
- YAML for GitHub Actions and Ansible deployment automation.

**Secondary:**
- Shell scripts for deployment and validation helpers.
- CSS for global application styling.
- Jinja templates for Ansible-managed runtime files and Podman quadlets.

## Runtime

**Application Runtime:**
- Single full-stack `Next.js` application under `app/`.
- Node.js runtime managed through Corepack and pnpm.
- Containerized app image published through GitHub/GHCR and deployed to OCI.

**Package Manager:**
- pnpm via Corepack.
- Root workspace plus app package commands are executed with `corepack pnpm --filter app ...`.

## Frameworks and Tooling

**Core:**
- `Next.js` App Router for public pages and API routes.
- React components for the public gallery, detail pages, image viewer, and supporting UI.
- Oracle-backed catalog service with app-mediated private media delivery.

**Testing and Validation:**
- App lint, typecheck, test, and build commands are part of the current validation contract.
- Public-surface privacy regression tests protect against leaking storage identifiers, direct Object Storage URLs, and premature admin workflow exposure.

**Infrastructure and Deployment:**
- Terraform under `infra/terraform/` for OCI baseline resources and state guidance.
- GitHub Actions for PR validation, image build/publish, deployment, and data smoke workflows.
- Ansible under `deploy/ansible/` for VM runtime configuration.
- Podman quadlets for long-lived app and Caddy containers on the runtime VM.

## Key Dependencies and Integrations

**Application:**
- Oracle Autonomous Database Free for catalog metadata.
- OCI Object Storage for private autograph media.
- Local filesystem media mode for local/CI smoke paths without live OCI credentials.

**Runtime:**
- Caddy as the public HTTP(S) edge in front of the app container.
- Podman as the container runtime on the OCI VM.
- GHCR as the container image registry.

## Configuration

**Environment Contract:**
- `.env.example` documents local/runtime variables.
- `.github/.env.github.example` documents GitHub Actions environment/secret expectations.
- `docs/configuration-contract.md` documents the committed configuration and secret contract.
- Ansible renders the deployed app environment file from `deploy/ansible/roles/autographs_deploy/templates/app.env.j2`.

**Secrets:**
- Oracle DB password, wallet material, OCI private key, OCI identifiers, GHCR token, and operator token are supplied through GitHub/environment secrets rather than committed values.
- Production Object Storage credentials are mounted/read through runtime environment and secret files.

## Platform Requirements

**Development:**
- Node.js/Corepack/pnpm for local app work.
- Terraform CLI for infrastructure work.
- Access to representative env files or local-mode settings for data/media smoke paths.

**Production:**
- OCI tenancy and Always Free-compatible resources where feasible.
- Oracle Autonomous Database Free.
- OCI Object Storage private bucket.
- OCI VM runtime capable of Podman, Caddy, app container, and configured swap.

## Project Maturity

**Current State:**
- Phases 1-3 are complete: delivery spine, OCI bootstrap, Oracle/private media core, and public gallery MVP.
- Phase 4 admin collection workflow is next for planning.
- The repository is no longer planning-only; it contains application, infrastructure, deployment, testing, and operator documentation artifacts.

**Practical Guidance:**
- Treat `.planning/STATE.md`, `.planning/ROADMAP.md`, and `.planning/PROJECT.md` as the current high-level planning sources of truth.
- Treat `.planning/codebase/*` as a current-state codebase map, not as historical prompt intent.
- Do not re-scaffold the app or infra; Phase 4 should build on the existing service boundaries, public gallery, and temporary operator bridge.

---

*Stack analysis: 2026-05-24 after Phase 3 completion*
