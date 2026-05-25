# Security Review

Phase 4 reviewed the current public gallery, deployment, runtime, and repository surfaces before adding the Phase 5 admin workflow or Phase 6 AI ingest. This review covers the system that exists now: anonymous public browsing, app-mediated private image delivery, the temporary operator bridge, OCI-backed runtime configuration, GitHub Actions delivery, and scheduled cleanup.

## Scope

Reviewed:

- Public routes: `/`, `/collection`, `/collection/{id}`, `/architecture`, `/api/catalog/*`, `/health`, and `/health/data`.
- Temporary operator routes: `/api/operator/*`.
- Private media delivery through app-mediated catalog image routes.
- Oracle and OCI Object Storage configuration boundaries.
- Caddy public ingress and runtime Podman deployment.
- GitHub Actions CI, deploy, data-smoke, and image-cleanup workflows.
- Repository hygiene for secrets, ignored runtime state, and public documentation.

Out of scope:

- Phase 5 admin authentication, edit history, and operator-bridge retirement.
- Phase 6 OCR/AI providers, prompts, privacy review, and model configuration.

## Findings

| ID | Surface | Disposition | Notes |
|----|---------|-------------|-------|
| SEC-04-01 | Public response headers | Fixed | Added baseline Next.js headers: `X-Content-Type-Options`, `Referrer-Policy`, `Permissions-Policy`, and a frame-blocking CSP. |
| SEC-04-02 | Anonymous `/health/data` | Fixed | Production anonymous data health now omits detailed config check names and errors. Detailed live readiness remains operator-token guarded. |
| SEC-04-03 | Public operator ingress | Fixed | Caddy continues to return `404` for `/api/operator` and `/api/operator/*`; regression coverage checks the committed Caddyfile. |
| SEC-04-04 | Public catalog/media privacy | Accepted | Existing public DTO and image-route tests verify public responses stay app-mediated and do not expose private storage identifiers. Continue running these gates in CI. |
| SEC-04-05 | Temporary operator bridge | Accepted | Operator mutation routes remain bearer-token guarded in-app and blocked at public ingress. This is acceptable only as a temporary SSH-tunnel procedure until Phase 5 replaces it. |
| SEC-04-06 | OCI and runtime secrets | Accepted | Secrets are supplied through GitHub Secrets, Terraform variables, VM-local files, and Ansible-managed environment files rather than committed source. Public docs must continue to use placeholders only. |
| SEC-04-07 | CI/CD permissions | Accepted | Workflows use explicit permissions for read, package publish/cleanup, Actions operations, and deploy jobs. Phase 4 dependency automation will document any accepted exceptions. |
| SEC-04-08 | Runtime image cleanup | Deferred to Plan 04-02 | Scheduled image cleanup has an observed Podman multi-tag deletion failure. Plan 04-02 owns the Ansible cleanup fix and supply-chain workflow review. |
| SEC-04-09 | Admin authentication | Deferred to Phase 5 | Real admin auth, edit history, and operator-bridge retirement are intentionally Phase 5 security work. |
| SEC-04-10 | AI/OCR security | Deferred to Phase 6 | Provider selection, prompts, privacy boundaries, and AI/OCR failure modes are intentionally Phase 6 security work. |

## Verification

Automated checks added or preserved:

- Public security header regression in `app/src/gallery/public-surface.test.ts`.
- Production `/health/data` anonymous-detail regression in `app/src/gallery/public-surface.test.ts`.
- Public Caddy operator block regression in `app/src/gallery/public-surface.test.ts`.
- Existing public surface privacy tests for private storage identifiers and image route anchors.

Validation commands:

```bash
corepack pnpm --filter app lint
corepack pnpm --filter app typecheck
corepack pnpm --filter app test
corepack pnpm --filter app build
```

## Follow-Up

- Plan 04-02 must fix the scheduled Image Cleanup failure and document dependency/workflow review expectations.
- Phase 5 must perform a fresh review for admin auth, edit history, media cleanup, and operator-bridge retirement.
- Phase 6 must perform a fresh review for OCR/AI provider configuration, prompt handling, privacy boundaries, and failure modes.
