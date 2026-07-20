# Technology Stack

**Analysis Date:** 2026-07-10

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
- Admin runtime: plain static HTML/CSS/JavaScript under `controller/static-admin/`
  uses same-origin `/admin/api/*` requests and HTTP-only session-cookie
  authentication for collection management.
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
- Phase 6 controller/admin surfaces include private item APIs, field-level edit
  history, media cleanup compensation, pending-change status, explicit publish
  controls, bounded release retention, and session-cookie collection management.
- Phase 7 controller/admin/public surfaces include reusable signer profiles,
  item signer credits, taxonomy suggestions, signer merge repair, schema
  version 2 public DTOs/facets, and a no-new-dependency taxonomy backfill CLI.
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
- `controller/src/bin/taxonomy_backfill.rs` for Phase 7 migration report and
  operator-reviewable PL/SQL generation from the committed
  `taxonomy-backfill-mapping.json` artifact.

## Configuration

- `.env.example` documents local/controller reference variables.
- `.github/.env.github.example` documents GitHub Actions secret/variable
  expectations.
- `docs/configuration-contract.md` documents the committed configuration and
  secret contract.
- Ansible renders controller/Caddy runtime files from
  `deploy/ansible/roles/autographs_deploy/`.
- Runtime static release retention values are rendered into `controller.env`
  through Ansible, including `AUTOGRAPHS_STATIC_PROMOTED_RELEASE_RETAIN_COUNT`
  and `AUTOGRAPHS_STATIC_FAILED_CANDIDATE_RETAIN_COUNT`.
- Secrets such as Oracle wallet material, ADB password, OCI private key,
  operator/admin tokens, GHCR token, deploy SSH key, and GitHub tokens must
  stay in GitHub/environment/operator secret stores.

## Validation

- PR CI checks Rust formatting, tests, production-persistence compile, build,
  clippy, controller image build, Dockerfile linting, Terraform formatting and
  validation, workflow linting, secret scanning, and Ansible syntax/lint.
- Live Oracle/Object Storage proof remains an operator-run smoke path because
  real secrets and tenancy state are required.
- Schema version 2 static contract tests cover public signer/taxonomy DTOs and
  generated facets, and the ignored live static publish smoke now checks
  schema version 2 facets while remaining gated by
  `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true`.
- Production security patching playbooks are syntax/lint covered and documented
  in `docs/security-patching.md`.

## Project Maturity

- Phases 1-5 are complete.
- Phase 5 plans 05-01 through 05-07 are done, and the static runtime
  migration foundation is implemented in code, operator docs, live smoke proof,
  UAT, security review, and verification closeout artifacts.
- Phase 6 plans 06-01 through 06-07 are done, and the polished static admin
  workflow, edit history, media cleanup, release retention, session-cookie auth,
  operator docs, security review, and local closeout gates are implemented.
- Phase 7 metadata taxonomy and public facets are implemented, including
  signer profile tables, item signer credits, admin signer merge repair,
  taxonomy backfill artifacts, schema version 2 public facets, rollout docs,
  security review, and live smoke taxonomy assertions.
- Phase 8 taxonomy media cue exploration and advisory AI-assisted ingest remain
  pending, with taxonomy media work prioritized first.

## Practical Guidance

- Treat `controller/`, `deploy/ansible/`, `infra/terraform/`, `.github/workflows/`,
  and `docs/` as current implementation surfaces.
- Treat historical Next.js references as old phase evidence unless a document
  explicitly marks them current.
- Do not re-scaffold the retired Node/Next.js app or pnpm workspace.
- Keep production security patching changes under the same review standard as
  deploy/runtime changes because they can affect the live VM.
- Keep Phase 6 admin changes inside the Rust/static/Caddy/Oracle/Object Storage
  architecture; do not introduce public accounts, multi-admin roles, bulk import,
  direct Object Storage URLs, or a split public service for v1.
- Start Phase 8 with optional franchise/product-line/set/non-default-language
  taxonomy media cues grounded in the Phase 7 taxonomy model, then keep later
  AI work advisory; do not move cataloging control away from the manual admin
  workflow.

---

*Stack analysis refreshed: 2026-07-19 after Phase 8 taxonomy media prioritization*
