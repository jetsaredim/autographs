# Testing Patterns

**Analysis Date:** 2026-05-28

## Validation Contract

The repository now has an implemented validation surface for the application and deployment flow.

### Primary Commands

```bash
corepack pnpm --filter app lint
corepack pnpm --filter app typecheck
corepack pnpm --filter app test
corepack pnpm --filter app build
```

These commands are used as the baseline verification contract during Phase 3 completion and deployment validation.

## Test Scope

### Application Tests

**Covered Areas:**
- Catalog data access and transformation paths.
- Public gallery listing and filtering behavior.
- Item detail metadata rendering.
- Multi-image public presentation behavior.
- App-mediated image route protections.
- Public-surface privacy regression coverage.
- Public security-header coverage.
- Production `/health/data` detail-redaction coverage.
- Caddy public operator-route block coverage.

### Infrastructure and Deployment Validation

**Covered Areas:**
- GitHub Actions workflow validation.
- OCI deployment pipeline validation.
- Environment contract verification.
- Runtime container deployment behavior.
- Schema and migration drift checks where applicable.
- Ansible image-cleanup playbook syntax validation.
- Renovate configuration JSON validation.

## Current Testing Strategy

### Unit and Service-Level Validation

- Service-layer logic is tested independently from live OCI integrations where possible.
- Local-mode and representative data paths are used to reduce dependency on live cloud credentials during routine development.

### Integration Validation

- OCI-backed smoke verification exists for real database and media integration paths.
- Public gallery validation includes regression checks to ensure Object Storage details and operator-only surfaces are not leaked publicly.

### Deployment Validation

- PR validation is GitHub-driven.
- Merge-to-main deployment is part of the intended operational workflow.
- Runtime deployment validation includes container startup and app accessibility checks.

## Test Organization

**Patterns Observed:**
- Validation is currently centered around app-level commands rather than a large multi-package matrix.
- The project favors end-to-end operational verification over isolated micro-unit coverage.
- Deployment and runtime validation are treated as first-class quality gates.

## Mocking and Fixtures

### Mocking Strategy

- OCI integrations are abstracted so local and CI paths can avoid requiring live tenancy access for every test run.
- Media workflows support local-mode validation where Object Storage is unavailable.

### Fixtures

- Representative sample data exists for local development and validation.
- Media/image test assets are used for upload and rendering verification.

## Coverage Gaps

### Phase 4 Coverage Status

Phase 4 coverage is complete for the current public-gallery/deployment surface:

- Current-surface security and attack-vector review notes in `docs/security-review.md`.
- Header, production health redaction, and Caddy operator-route regression tests in `app/src/gallery/public-surface.test.ts`.
- Renovate config validation for package, workflow, container/Corepack, Terraform, and runtime image update surfaces.
- Cleanup workflow hardening verified through the Ansible cleanup playbook syntax check.
- Public README/showcase and architecture-doc reconciliation checks.
- Final public-readiness signoff captured in `.planning/phases/04-public-showcase-and-hardening/04-05-SUMMARY.md` and `docs/public-readiness.md`.

### Pending Phase 5 Areas

The following capabilities are not yet implemented or covered because Phase 5 has not started:

- Static publisher contract tests for gallery, detail, search/facet data, generated media paths, and publish manifests.
- Leak checks ensuring generated output does not expose private OCI object identifiers, bucket names, namespaces, object keys, Oracle details, private URLs, image UUIDs, or unpublished records.
- Published image derivative validation, including metadata stripping, completeness checks, and public-safe path generation.
- Caddy side-by-side static preview checks before public cutover.
- Thin private admin/publisher API foundation checks for health, private access enforcement, minimal content seeding into Oracle/Object Storage, and publish trigger/status behavior.
- End-to-end seeded content smoke: submit minimal metadata and one image through the private admin/API boundary, persist to Oracle/Object Storage, generate static output, and verify the generated public page plus derivative image.
- Cutover and retirement validation for the public Next.js runtime, public catalog APIs, app-mediated image streaming, data smoke path, and temporary operator bridge.

### Pending Phase 6 Areas

- Polished admin session UX and hardening beyond the Phase 5 minimal single-admin/private controller foundation.
- Polished end-to-end admin create/edit/publish workflow.
- Edit history persistence and rendering.
- Media replacement/orphan cleanup guarantees.
- Admin/public boundary hardening.
- Admin route, secret, and edit-history documentation checks.

### Pending Phase 7 Areas

- OCR/AI-assisted ingest validation.
- AI/OCR provider, prompt, failure-mode, privacy-boundary, and configuration/secret review.

## Practical Guidance

- Treat public-surface privacy regression tests as mandatory gates before exposing new routes.
- Treat app-mediated media delivery as a sensitive path requiring both correctness and security validation.
- Maintain a distinction between:
  - local-mode developer validation,
  - CI validation,
  - live OCI smoke validation.
- Keep deployment validation tied to the actual runtime model. If the project pivots to static generation, replace live-app data smoke checks with publisher, static-output, and private-admin-API validation.

---

*Testing analysis refreshed: 2026-05-28 after Phase 5 static-runtime context gathering*
