# Coding Conventions

**Analysis Date:** 2026-05-25

## Naming Patterns

**Planning Artifacts**
- Phase directories use zero-padded numeric prefixes plus kebab-case slugs, for example `.planning/phases/03-public-gallery-mvp/`.
- Phase plan and summary files use `{phase}-{plan}-PLAN.md` and `{phase}-{plan}-SUMMARY.md`.
- Codebase map docs use uppercase concern names in `.planning/codebase/`.

**Application Files**
- Next.js route files follow App Router conventions: `page.tsx`, `layout.tsx`, `route.ts`, `not-found.tsx`.
- React components use PascalCase filenames under `app/app/components/`.
- Domain modules use descriptive kebab-free TypeScript names under `app/src/`, for example `public-view-models.ts`, `repository.ts`, `service.ts`.
- Tests live beside related source modules as `*.test.ts`.

**Domain Language**
- Prefer project terms already established in requirements: autograph item, signer, category, tags, primary image, supporting images, publication status, operator bridge, admin workflow, edit history.

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

---

*Conventions refreshed: 2026-05-25 after repo-state reconciliation*
