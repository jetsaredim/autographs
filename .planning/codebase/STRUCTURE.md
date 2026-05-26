# Codebase Structure

**Analysis Date:** 2026-05-25

## Directory Layout

```text
autographs/
├── app/                         # Full-stack Next.js application package
│   ├── app/                     # App Router pages, layouts, API routes, components
│   ├── db/                      # SQL migrations and seed fixtures
│   ├── scripts/                 # Migration, seed, and smoke scripts
│   └── src/                     # Catalog, database, media, and test modules
├── deploy/ansible/              # OCI runtime VM configuration and Podman quadlets
├── docs/                        # Operator runbooks and configuration docs
├── infra/terraform/             # OCI infrastructure as code
├── .github/workflows/           # CI, deploy, data smoke, and image cleanup workflows
├── renovate.json                # Conservative dependency automation policy
├── .planning/                   # GSD project state, roadmap, phases, and codebase maps
├── .prompts/                    # Original implementation prompt artifacts
├── package.json                 # Root pnpm workspace commands
└── pnpm-workspace.yaml          # Workspace definition
```

## Key File Locations

**Application Entry Points**
- `app/app/layout.tsx`: root metadata/layout.
- `app/app/page.tsx`: public landing page.
- `app/app/collection/page.tsx`: public collection grid and URL-backed filters.
- `app/app/collection/[id]/page.tsx`: published item detail page.
- `app/app/admin/page.tsx`: placeholder only; Phase 5 owns real admin workflow.

**API Routes**
- `app/app/api/catalog/`: public published-only catalog and app-mediated images.
- `app/app/api/operator/catalog/`: temporary token-guarded operator mutation bridge.
- `app/app/health/`: runtime health endpoints.

**Domain Logic**
- `app/src/catalog/`: catalog types, service, Oracle repository, public view models, tests.
- `app/src/media/`: private media store abstraction and OCI/local implementations.
- `app/src/db/`: Oracle connection/config/migration helpers.

**Infrastructure and Runtime**
- `infra/terraform/`: OCI networking, compute, ADB, Object Storage, DNS, and supporting resources.
- `infra/terraform/tenancy/`: tenancy-level/manual-bootstrap guidance and examples.
- `deploy/ansible/`: VM runtime setup, app/Caddy quadlets, deployment roles.
- `.github/workflows/`: repository validation, deployment, smoke, and cleanup workflows.

**Planning and Documentation**
- `.planning/PROJECT.md`, `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md`: high-level GSD truth.
- `.planning/phases/01-*`, `02-*`, `03-*`: completed phase plans/summaries.
- `.planning/codebase/*.md`: current codebase maps for future agents.
- `README.md`: public showcase, status, architecture, local-development, operations, security, and human+AI/GSD framing.
- `docs/`: operator-facing setup, deploy, DNS, Terraform, dependency-update, security-review, and temporary data-entry runbooks.
- `docs/architecture.drawio` and `app/public/architecture-diagram.svg`: current public architecture diagram sources.

## Test Organization

- `app/src/**/*.test.ts`: Node test runner coverage for service logic, public view models, approved quote inventory, and public-surface privacy regressions.
- Primary commands are run through pnpm workspace filters: `corepack pnpm --filter app test`, `lint`, `typecheck`, and `build`.

## Where to Add New Code

**Phase 4 Public Showcase and Hardening**
- Root README, badges, public metadata, dependency automation, security review notes, and public-facing docs now live in repository docs/configuration surfaces rather than changing app architecture.
- Current-surface hardening should preserve the existing public gallery, deployment, media, and operator-route boundaries.

**Next Phase Static Runtime Pivot**
- Treat the researched static-runtime direction as planning context, not implemented architecture: static public catalog, static admin shell, and a thin private admin/publisher API.
- Start with the public static artifact contract and publisher preview before replacing the current public Next.js runtime.
- Keep private OCI Object Storage identifiers, Oracle data, image UUIDs, and object URLs inside the OCI/runtime boundary; do not move catalog content generation into GitHub-hosted workflows.
- Caddy and Ansible are likely retained, but Caddy would shift from reverse-proxying all public traffic to serving generated public files plus a private admin/API boundary.

**Phase 5 Admin Workflow**
- Replan before adding significant code: Phase 5 may become a static publishing foundation rather than direct expansion of `app/app/admin/`.
- If the pivot is accepted, admin pages should become a static shell served by Caddy and privileged mutations should move behind a thin private API.
- Reuse the existing catalog/media field model and public DTO contracts where practical, but avoid recreating the current dynamic public app under a different language runtime.
- Edit history remains a Phase 5 requirement, but its storage/API shape should be decided during the static-runtime planning pass.

**Public Gallery Changes**
- Keep public DTOs in `app/src/catalog/public-view-models.ts`.
- Keep public privacy regressions in `app/src/gallery/public-surface.test.ts`.

**Infrastructure Changes**
- Use `infra/terraform/` for app infrastructure and `infra/terraform/tenancy/` only for tenancy-level bootstrap concerns.
- Keep runtime VM behavior in `deploy/ansible/` unless a Terraform resource boundary truly belongs in infrastructure.

## Current Layout Guidance

- Do not re-scaffold the app, pnpm workspace, workflows, or Terraform baseline.
- Treat `.prompts/001-autograph-gallery-bootstrap-do/` as historical product intent, not the current implementation map.
- Treat Phase 4 as complete current-surface showcase and hardening work on top of the completed public/data/media foundation.
- Treat Phase 5 as the next planning decision point: either continue the original admin workflow plan or insert the static runtime migration foundation before full admin CRUD.

---

*Structure analysis refreshed: 2026-05-26 after Phase 4 completion reconciliation*
