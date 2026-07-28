# External Integrations

**Analysis Date:** 2026-07-17

## APIs and External Services

**Oracle Cloud Infrastructure**
- Terraform provisions and manages OCI app infrastructure under `infra/terraform/`.
- Tenancy/root bootstrap guidance lives under `infra/terraform/tenancy/` and
  operator docs.
- Runtime deploys target an OCI VM using Ansible-managed Podman quadlets.
- The runtime uses OCI-native credentials/instance-principal direction for
  private Object Storage access.

**Oracle Autonomous Database**
- Catalog metadata is stored in Oracle Autonomous Database Free.
- Schema lives in `controller/db/schema.sql`.
- Production persistence lives in `controller/src/oracle_catalog.rs` and
  related schema/config modules.
- Phase 6 persists edit history, publication changes, cleanup events,
  pending-change timestamps, and publish snapshot/status data through the same
  controller-owned Oracle boundary.
- Phase 7 introduced signer profile tables, item signer credits,
  character/franchise joins, item taxonomy fields, signer profile edit history,
  signer merge history, and reviewed taxonomy backfill artifacts for live
  rollout.

**OCI Object Storage**
- Private original autograph images are stored in a private Object Storage bucket.
- Public users receive generated derivatives through static release paths, not
  direct object URLs or object keys.
- Controller media behavior lives in `controller/src/media.rs` and
  `controller/src/oci_media.rs`.
- Phase 6 image replacement/removal uses media cleanup compensation and retry
  events so private originals and Oracle metadata stay reconciled.
- Phase 7 public schema version 2 artifacts and live smoke assertions continue
  to exclude Object Storage identifiers, object keys, bucket names, and original
  filenames from public JSON/HTML/media paths.

**GitHub Actions and GHCR**
- CI/deploy workflows live in `.github/workflows/`.
- Controller images are built/published to GHCR and deployed by digest.
- Image cleanup has a dedicated scheduled/manual workflow.
- Production security patching has dedicated scan/apply workflows.
- Renovate is configured through `renovate.json`.

**Caddy and Podman**
- Caddy serves generated public static output and routes private admin/API
  surfaces.
- Caddy sets `Cache-Control: no-store` for `/admin` and `/admin/api/*`, short
  cache lifetimes for public HTML/JSON/manifest paths, one-hour public asset
  caching, and one-day generated media caching.
- Podman quadlets manage the controller/Caddy runtime on the OCI VM.
- Ansible renders and deploys runtime files.
- `deploy/ansible/roles/autographs_deploy/templates/controller.env.j2` renders
  static release retention settings, including
  `AUTOGRAPHS_STATIC_PROMOTED_RELEASE_RETAIN_COUNT` and
  `AUTOGRAPHS_STATIC_FAILED_CANDIDATE_RETAIN_COUNT`.

## Authentication and Identity

**Current**
- Public gallery is anonymous and static.
- Private admin/publish behavior uses the Rust controller under `/admin` and
  `/admin/api/*`.
- Collection-management routes use the single-admin HTTP-only session-cookie
  path from `/admin/api/login`; bearer-token compatibility is limited to
  non-management diagnostics.
- Retired operator APIs remain blocked at the public Caddy edge.
- Production security update approval is GitHub-label based and restricted to
  `.github/production-patch-approvers.yml`.
- Cloudflare/CDN fronting is documented as deferred. If enabled later, admin/API
  caching must be bypassed and rollback must be protected by conservative
  HTML/JSON freshness or content-addressed public paths.

**Pending**
- Phase 8 admin image preview/adjustment integration and production security
  patching repair.
- Phase 9 taxonomy thumbnail/media processing integration.
- Phase 10 advisory AI/OCR provider integration.
- There is intentionally no public account system, multi-admin role hierarchy,
  or social identity flow for v1.

## CI/CD and Deployment

**Implemented**
- Pull-request validation through GitHub Actions.
- Merge-to-main deployment path.
- GHCR controller image publishing.
- OCI runtime configuration through Terraform and Ansible.
- Podman quadlets for controller and Caddy services.
- Production security patch scan/apply workflows through GitHub Issues and
  Ansible.

**Operator Docs**
- `docs/configuration-contract.md`
- `docs/controller-walkthrough.md`
- `docs/deployment-runbook.md`
- `docs/dependency-updates.md`
- `docs/oci-bootstrap.md`
- `docs/security-patching.md`
- `docs/security-review.md`
- `docs/static-artifact-contract.md`
- `docs/static-runtime-runbook.md`
- `docs/terraform-state.md`
- `docs/temporary-production-data-entry.md`

## Environment Configuration

**Committed Contracts**
- `.env.example` for local/controller variables.
- `.github/.env.github.example` for GitHub Actions secrets and variables.
- `docs/configuration-contract.md` for human-readable configuration guidance.

**Secret Handling**
- Real OCI identifiers, API keys, Oracle wallet material, ADB password, GHCR
  token, deploy SSH key, admin/operator tokens, and Terraform tfvars/state must
  stay in environment/GitHub/operator secret stores.

## AI Integrations

No admin image adjustment, taxonomy thumbnail/media processing, OCR, or
AI-assisted metadata suggestion integration is implemented yet. Admin image
preview/adjustment is Phase 8 scope, taxonomy media cue work is Phase 9 scope,
and OCR/AI provider integration is Phase 10 scope. Manual entry and text-only
taxonomy display must remain fully functional against the Phase 7
signer/taxonomy model.

## Practical Interpretation

The repo contains real Rust/controller, static publishing, infrastructure,
delivery, maintenance, Phase 6 admin workflow, edit history, media cleanup,
release retention, Phase 7 metadata taxonomy/public facets, and operator
integration surfaces. Future work should extend these boundaries rather than
treating OCI, Oracle, Object Storage, GitHub Actions, Caddy, or `/admin` as
prompt-only intent.

---

*Integration audit refreshed: 2026-07-28 after Phase 8/9/10 roadmap split*
