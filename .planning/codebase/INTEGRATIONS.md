# External Integrations

**Analysis Date:** 2026-07-02

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

**OCI Object Storage**
- Private original autograph images are stored in a private Object Storage bucket.
- Public users receive generated derivatives through static release paths, not
  direct object URLs or object keys.
- Controller media behavior lives in `controller/src/media.rs` and
  `controller/src/oci_media.rs`.
- Phase 6 image replacement/removal uses media cleanup compensation and retry
  events so private originals and Oracle metadata stay reconciled.

**GitHub Actions and GHCR**
- CI/deploy workflows live in `.github/workflows/`.
- Controller images are built/published to GHCR and deployed by digest.
- Image cleanup has a dedicated scheduled/manual workflow.
- Production security patching has dedicated scan/apply workflows.
- Renovate is configured through `renovate.json`.

**Caddy and Podman**
- Caddy serves generated public static output and routes private admin/API
  surfaces.
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

**Pending**
- Advisory Phase 7 AI/OCR provider integration.
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

No OCR or AI-assisted metadata suggestion integration is implemented yet. That
remains Phase 7 scope and should be advisory, with manual entry remaining fully
functional.

## Practical Interpretation

The repo contains real Rust/controller, static publishing, infrastructure,
delivery, maintenance, Phase 6 admin workflow, edit history, media cleanup,
release retention, and operator integration surfaces. Future work should extend
these boundaries rather than treating OCI, Oracle, Object Storage, GitHub
Actions, Caddy, or `/admin` as prompt-only intent.

---

*Integration audit refreshed: 2026-07-02 after Phase 6 admin docs/security closeout*
