<!-- GSD:project-start source:PROJECT.md -->
## Project

**Autographs**

Autographs is a production-lean personal autograph collection website where you can publish your own signed memorabilia for anonymous public browsing. The first release pairs a single self-hosted `Next.js` application with private OCI Object Storage for images and Oracle Autonomous Database Free for metadata, while also establishing the OCI bootstrap, CI/CD, and operator guidance needed to run the collection as a real, durable personal project.

**Core Value:** A collector can reliably browse and manage a high-quality autograph catalog where private images and useful metadata stay connected end to end.

### Constraints

- **Tech stack**: Use a single `Next.js` full-stack application for v1 — keeps implementation and operations simpler than a split-service design.
- **Cloud**: Prefer OCI Always Free services wherever feasible — the product should be realistic for a fresh low-cost tenancy.
- **Database**: Prefer Oracle Autonomous Database Free — the prompt explicitly selects it unless implementation friction forces a justified fallback.
- **Storage**: Keep autograph images private in OCI Object Storage — access should be centralized through the app rather than direct public buckets.
- **Delivery**: Auto-deploy from GitHub Actions on merge to `main` — CI/CD is part of project bootstrap, not optional polish.
- **Operations**: One developer should be able to understand and run the system — avoid enterprise sprawl and multi-service complexity.
- **Scope**: v1 must stay narrow — no staging environment, no bulk import, no public accounts, and no advanced search platform, but multi-image items and edit history are in scope because they matter directly for managing a personal collection well.
- **Security**: Use least-privilege OCI access and explicit secret handling — routine deploy workflows should not rely on tenancy-wide admin power.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- TypeScript for the `Next.js` application, service layer, scripts, and tests.
- Markdown for planning, operator runbooks, and repository documentation.
- HCL for Terraform infrastructure.
- YAML for GitHub Actions and Ansible deployment automation.
- Shell scripts for deployment and validation helpers.
- CSS for global application styling.
## Runtime
- Single full-stack `Next.js` application under `app/`.
- Node.js runtime managed through Corepack and pnpm.
- Containerized app image published through GitHub/GHCR and deployed to OCI.
- App package commands are executed with `corepack pnpm --filter app ...`.
## Frameworks
- `Next.js` App Router for public pages and API routes.
- React components for the public gallery, detail pages, image viewer, and supporting UI.
- Oracle-backed catalog service with app-mediated private media delivery.
- Terraform under `infra/terraform/` for OCI baseline resources and state guidance.
- GitHub Actions for PR validation, image build/publish, deployment, and data smoke workflows.
- Ansible under `deploy/ansible/` for VM runtime configuration.
- Podman quadlets for long-lived app and Caddy containers on the runtime VM.
## Key Dependencies
- Oracle Autonomous Database Free for catalog metadata.
- OCI Object Storage for private autograph media.
- Local filesystem media mode for local/CI smoke paths without live OCI credentials.
- Caddy as the public HTTP(S) edge in front of the app container.
- Podman as the container runtime on the OCI VM.
- GHCR as the container image registry.
## Configuration
- `.env.example` documents local/runtime variables.
- `.github/.env.github.example` documents GitHub Actions environment/secret expectations.
- `docs/configuration-contract.md` documents the committed configuration and secret contract.
- Ansible renders the deployed app environment file from `deploy/ansible/roles/autographs_deploy/templates/app.env.j2`.
- Secrets such as Oracle DB password, wallet material, OCI private key, OCI identifiers, GHCR token, and operator token must be supplied through GitHub/environment/operator secret stores.
## Platform Requirements
- Development uses Node.js/Corepack/pnpm for app work and Terraform CLI for infrastructure work.
- Production targets OCI tenancy resources, Oracle Autonomous Database Free, private OCI Object Storage, and an OCI VM runtime capable of Podman, Caddy, the app container, and configured swap.
## Project Maturity
- Phases 1-3 are complete: delivery spine, OCI bootstrap, Oracle/private media core, and public gallery MVP.
- Phase 4 admin collection workflow is next for planning.
- The repository is no longer planning-only; it contains application, infrastructure, deployment, testing, and operator documentation artifacts.
- Do not re-scaffold the app or infra; Phase 4 should build on the existing service boundaries, public gallery, and temporary operator bridge.
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Naming Patterns
- Phase directories use zero-padded numeric prefixes plus kebab-case slugs, for example `.planning/phases/03-public-gallery-mvp/`.
- Phase plan and summary files use `{phase}-{plan}-PLAN.md` and `{phase}-{plan}-SUMMARY.md`.
- Codebase map docs use uppercase concern names in `.planning/codebase/`.
- Next.js route files follow App Router conventions: `page.tsx`, `layout.tsx`, `route.ts`, `not-found.tsx`.
- React components use PascalCase filenames under `app/app/components/`.
- Domain modules use descriptive TypeScript names under `app/src/`, for example `public-view-models.ts`, `repository.ts`, and `service.ts`.
- Tests live beside related source modules as `*.test.ts`.
- Prefer established domain terms: autograph item, signer, category, tags, primary image, supporting images, publication status, operator bridge, admin workflow, and edit history.
## Code Style
- TypeScript is the implementation language for app, service, scripts, and tests.
- The app uses native CSS in `app/app/globals.css`; Phase 3 explicitly avoided Tailwind, shadcn, decorative gradients, and icon libraries.
- Use existing service/repository/media boundaries rather than placing persistence details directly in route components.
- Keep public DTOs free of private storage identifiers.
- Keep admin/operator terminology precise: current operator APIs are temporary, token-guarded, and blocked by the public Caddy route; Phase 4 admin UX is not implemented yet.
## Import Organization
- Prefer relative imports within the app package unless a local alias is introduced intentionally.
- Use type-only imports for TypeScript-only contracts where possible.
- Keep framework route/page files thin and delegate behavior to `app/src/*` modules.
## Error Handling
- Public routes should avoid leaking internal storage, OCI, or database details.
- Operator routes may return operational errors, but must remain token-guarded and accessible only through the documented tunnel/procedure until Phase 4.
- Service-layer methods throw explicit not-found errors for missing catalog items/images; API routes translate expected not-found cases to HTTP responses.
## Logging
- No project-specific logging abstraction exists yet.
- Runtime and deploy diagnostics currently rely on Next.js, Podman, Caddy, GitHub Actions, and Ansible logs.
## Comments
- Add comments only where a non-obvious operational or domain boundary needs context.
- Prefer self-explanatory TypeScript names and existing service/repository/media separation over explanatory comments for ordinary flow.
## Testing Habits
- Use Node's built-in test runner through `node --import tsx --test src/**/*.test.ts`.
- Keep privacy regression tests mandatory for public-surface changes.
- Prefer service/view-model tests for behavior that can be validated without live OCI credentials.
- Use live data smoke workflows only for real ADB/Object Storage verification.
## Documentation Habits
- Distinguish current implementation from planned/future phases.
- Keep operator docs procedural and explicit about manual prerequisites, secret handling, and tunnel-only temporary routes.
- Update `.planning/codebase/*` after substantial codebase drift so future agents do not resurrect planning-only assumptions.
## Current Guidance
- Phase 4 should add admin authentication, create/edit/publish workflows, edit history, and media cleanup guarantees on top of the existing catalog/media service boundaries.
- Do not introduce public accounts, multi-admin roles, direct Object Storage URLs, or a split frontend/backend service architecture for v1.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## Pattern Overview
- Autographs is an implemented single-application system, not a planning-only repository.
- The current architecture is a full-stack `Next.js` App Router application under `app/`, backed by Oracle Autonomous Database for catalog metadata and private OCI Object Storage for autograph images.
- Public visitors browse only published items.
- Temporary operator-only mutation routes remain token-guarded and blocked at the public Caddy edge until Phase 4 replaces them with the real single-admin workflow.
## Layers
- Public web layer: `app/app/` contains anonymous landing, collection, detail, image viewer, architecture, not-found, and shared component surfaces.
- Public API layer: `app/app/api/catalog/` provides published-only catalog list/detail access and app-mediated image delivery.
- Temporary operator API layer: `app/app/api/operator/catalog/` provides transitional token-guarded create, update, image attach, image delete, and item delete workflows for production data entry before Phase 4. It is not the v1 admin UX.
- Catalog service layer: `app/src/catalog/` contains domain types, Oracle repository, catalog service orchestration, public-safe view models, and tests.
- Media layer: `app/src/media/` abstracts OCI Object Storage and local filesystem-backed media modes.
- Database layer: `app/src/db/`, `app/db/migrations/`, and `app/scripts/` handle Oracle configuration, schema, migrations, seed data, and data smoke helpers.
- Infrastructure and delivery layer: `infra/terraform/`, `deploy/ansible/`, and `.github/workflows/` provide OCI infrastructure, runtime VM configuration, Podman quadlets, PR validation, image publishing, deploy, data smoke, and image cleanup.
- Planning and operator documentation: `.planning/`, `docs/`, and `.prompts/` hold GSD state, roadmap, phase artifacts, codebase intelligence, bootstrap/runbook docs, and original product prompt.
## Data Flow
- Anonymous visitors request `/`, `/collection`, or `/collection/{id}`.
- Public pages call the catalog service, which lists/reads only `published` records by default.
- Public view models convert private catalog records into public-safe DTOs and route image access through `/api/catalog/{itemId}/images/{imageId}`.
- Image API routes resolve the published item and stream bytes from the configured private media store without exposing Object Storage URLs or object identifiers in the public UI.
- Temporary operator API calls use `AUTOGRAPHS_OPERATOR_API_TOKEN`, then create/update Oracle rows and upload/delete private media through the same service layer. Production operator use goes through the documented SSH tunnel path because public Caddy blocks `/api/operator/*`.
- GitHub Actions validates changes and deploys the containerized app/runtime changes to OCI on the documented path.
## Key Abstractions
- `CatalogService`: Coordinates metadata and media operations.
- `CatalogRepository`: Persists catalog records, tags, and image metadata through Oracle.
- `PrivateMediaStore`: Abstracts OCI Object Storage and local media modes.
- Public view models: Strip private storage fields and build app-mediated image routes.
- Operator API bridge: Temporary mutation surface used until Phase 4 admin workflow exists.
## Current Phase Boundary
- Phases 1-3 are complete.
- Phase 4 should build on the existing catalog service, media abstraction, public gallery, and operator bridge.
- Do not re-scaffold the application or replace the delivery spine.
## Notable Absences
- Real single-admin authentication and admin UX are not implemented yet.
- Edit history persistence/rendering is not implemented yet.
- AI-assisted metadata suggestions are not implemented yet.
- Final public-readiness hardening, repository badges, and README polish remain Phase 6 work.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

## Git Commit Branch Guardrails

- Never commit directly to `main` or `master`.
- If work starts on `main` or `master`, create or switch to a dedicated work branch before editing files that will be committed.
- Keep all commits for a task inside the current work branch. Merge back to `main` only through the project's normal PR/merge path.
- If a commit command would run on `main` or `master`, stop and report the current branch plus the branch that should contain the work.

## Connectivity and Publishing Failures

- If `git push`, `git fetch`, `gh`, SSH, DNS, or GitHub API calls fail because of local connectivity, local SSH configuration, credentials, network restrictions, or sandbox/network access, stop and tell the user immediately.
- Do not spend time inventing alternate push/fetch workarounds such as overriding `GIT_SSH_COMMAND`, switching remotes, bypassing SSH config, or retrying through a different protocol unless the user explicitly asks for that approach.
- Prefer a concise report with the failing command, the important error text, and the suggested user-side action. The user can often resolve host connectivity issues faster outside the Codex environment.
- After the user confirms the issue is fixed, retry the original straightforward command.

## Pull Request Review Findings

- When a review agent is deployed on a GitHub PR, every actionable finding from that agent must be written back to the PR as a GitHub comment.
- Prefer inline review comments when a finding maps to a specific changed line; otherwise post a single PR-level review/comment that groups the remaining findings by severity.
- Do not leave review-agent findings only in chat, local files, or agent summaries. The PR must contain the review feedback so it is visible during code review.
- If GitHub comment creation fails because of connectivity, credentials, permissions, or API errors, stop and report the failed command/API action plus the finding text that still needs to be posted.



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
