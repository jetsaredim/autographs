# External Integrations

**Analysis Date:** 2026-05-25

## APIs and External Services

**Oracle Cloud Infrastructure**
- Terraform provisions and manages OCI app infrastructure under `infra/terraform/`.
- Tenancy/root bootstrap guidance lives under `infra/terraform/tenancy/` and docs.
- Runtime deploys target an OCI VM using Ansible-managed Podman quadlets.

**Oracle Autonomous Database**
- Catalog metadata is stored in Oracle Autonomous Database Free.
- Schema lives in `app/db/migrations/001_catalog_core.sql`.
- Data access is implemented through `app/src/db/*` and `app/src/catalog/repository.ts`.

**OCI Object Storage**
- Autograph images are stored in a private Object Storage bucket.
- Media access is abstracted through `app/src/media/`.
- Public visitors receive images only through app-mediated routes, not direct object URLs.

**GitHub Actions and GHCR**
- CI/deploy workflows live in `.github/workflows/`.
- App images are built/published to GHCR and deployed by digest.
- Data smoke and image cleanup have dedicated workflows.

**Local Development Modes**
- Local filesystem media mode supports development and tests without requiring live OCI media credentials.
- Seed/smoke scripts live in `app/scripts/`.

## Authentication and Identity

**Current**
- Public gallery is anonymous.
- Temporary operator mutation routes require `AUTOGRAPHS_OPERATOR_API_TOKEN`.
- Operator routes are documented as SSH-tunnel/procedure-only until Phase 5, with public Caddy routing blocking `/api/operator/*`.

**Pending**
- Real single-admin authentication is Phase 5 scope.
- There is intentionally no public account system, multi-admin role hierarchy, or social identity flow for v1.

## CI/CD and Deployment

**Implemented**
- Pull-request validation through GitHub Actions.
- Merge-to-main deployment path.
- GHCR image publishing.
- OCI runtime configuration through Ansible.
- Podman quadlets for app and Caddy services.

**Operator Docs**
- `docs/configuration-contract.md`
- `docs/deployment-runbook.md`
- `docs/oci-bootstrap.md`
- `docs/terraform-state.md`
- `docs/terraform-tenancy-split.md`
- `docs/temporary-production-data-entry.md`

## Environment Configuration

**Committed Contracts**
- `.env.example` for local/runtime app variables.
- `.github/.env.github.example` for GitHub Actions secrets and variables.
- `docs/configuration-contract.md` for human-readable configuration guidance.

**Secret Handling**
- Real OCI identifiers, API keys, Oracle wallet material, ADB password, GHCR token, and operator token must stay in environment/GitHub/operator secret stores.
- Terraform state and override files are ignored by `.gitignore`; do not commit live state or real tfvars.

## AI Integrations

No OCR or AI-assisted metadata suggestion integration is implemented yet. That remains Phase 6 scope and should be advisory, with manual entry remaining fully functional.

## Practical Interpretation

The repo now contains real app, infrastructure, delivery, and operator integration surfaces. Future work should extend these boundaries rather than treating OCI, Oracle, Object Storage, or GitHub Actions as prompt-only intent.

---

*Integration audit refreshed: 2026-05-25 after repo-state reconciliation*
