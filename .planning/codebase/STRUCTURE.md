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
- `app/app/admin/page.tsx`: placeholder only; Phase 4 owns real admin workflow.

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
- `docs/`: operator-facing setup, deploy, DNS, Terraform, and temporary data-entry runbooks.

## Test Organization

- `app/src/**/*.test.ts`: Node test runner coverage for service logic, public view models, approved quote inventory, and public-surface privacy regressions.
- Primary commands are run through pnpm workspace filters: `corepack pnpm --filter app test`, `lint`, `typecheck`, and `build`.

## Where to Add New Code

**Phase 4 Admin Workflow**
- Admin pages/components: `app/app/admin/` and `app/app/components/` as needed.
- Admin API/server actions: prefer existing service boundaries in `app/src/catalog/` and `app/src/media/`; avoid duplicating persistence logic in UI routes.
- Auth/session helpers: add under a focused `app/src/admin/` or `app/src/auth/` module once the mechanism is chosen.
- Edit history: extend `app/db/migrations/` and `app/src/catalog/` rather than creating a parallel audit store.

**Public Gallery Changes**
- Keep public DTOs in `app/src/catalog/public-view-models.ts`.
- Keep public privacy regressions in `app/src/gallery/public-surface.test.ts`.

**Infrastructure Changes**
- Use `infra/terraform/` for app infrastructure and `infra/terraform/tenancy/` only for tenancy-level bootstrap concerns.
- Keep runtime VM behavior in `deploy/ansible/` unless a Terraform resource boundary truly belongs in infrastructure.

## Current Layout Guidance

- Do not re-scaffold the app, pnpm workspace, workflows, or Terraform baseline.
- Treat `.prompts/001-autograph-gallery-bootstrap-do/` as historical product intent, not the current implementation map.
- Treat Phase 4 as additive admin workflow work on top of the completed public/data/media foundation.

---

*Structure analysis refreshed: 2026-05-25 after repo-state reconciliation*
