# Testing Patterns

**Analysis Date:** 2026-05-24

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

### Infrastructure and Deployment Validation

**Covered Areas:**
- GitHub Actions workflow validation.
- OCI deployment pipeline validation.
- Environment contract verification.
- Runtime container deployment behavior.
- Schema and migration drift checks where applicable.

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

### Pending Phase 4 Areas

The following capabilities are not yet fully implemented or covered because Phase 4 has not started:

- Real admin authentication flow.
- End-to-end admin create/edit workflow.
- Edit history persistence and rendering.
- Media replacement/orphan cleanup guarantees.
- Final admin/public boundary hardening.

### Pending Phase 5/6 Areas

- OCR/AI-assisted ingest validation.
- Final public showcase hardening.
- Dependency automation validation.
- Final public-readiness security review.

## Practical Guidance

- Treat public-surface privacy regression tests as mandatory gates before exposing new routes.
- Treat app-mediated media delivery as a sensitive path requiring both correctness and security validation.
- Maintain a distinction between:
  - local-mode developer validation,
  - CI validation,
  - live OCI smoke validation.
- Keep deployment validation tied to real runtime behavior rather than static-only checks.

---

*Testing analysis refreshed: 2026-05-24 after Phase 3 completion*
