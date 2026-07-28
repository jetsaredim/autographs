# Security Review

This document records the current security posture and keeps the historical
Phase 4 review for context. Current production verification should use the Rust
controller/static runtime checks, deploy runbook, static runtime runbook, and
production security patching runbook rather than retired Next.js app commands.

## Current Static Runtime Posture

Current surfaces:

- Public static release served by Caddy.
- Rust private controller under `/admin` and `/admin/api/*`.
- Oracle Autonomous Database metadata.
- Private OCI Object Storage originals and generated public-safe derivatives.
- GitHub Actions CI/deploy/image-cleanup workflows.
- Production security patch scan/apply workflows.
- Terraform/Ansible runtime configuration and Podman quadlets.

Current accepted posture:

- Public catalog output is static and read-only.
- Public media is generated derivative output, not direct Object Storage access.
- Retired operator APIs remain blocked at the public Caddy edge.
- Admin/publish behavior uses the Rust controller foundation.
- Production security patching requires an allowlisted label actor, scanner
  metadata validation, package-set drift refusal, result comments, and stale
  approval-label cleanup on failure.

Current follow-up scope:

- Phase 6 admin workflow security review is recorded below.
- Phase 7 Metadata Taxonomy Security Review is recorded below.
- Phase 8 must review production security patching repair plus admin image
  preview/adjustment surfaces. Phase 9 must review taxonomy thumbnail/media
  processing dependencies, generated public derivatives, and
  copyright/publication risks. Phase 10 must review OCR/AI providers, prompts,
  privacy boundaries, and model configuration before advisory AI-assisted ingest
  ships.

## Phase 6 Admin Collection Workflow

Reviewed areas: session-cookie admin auth, absence of public accounts, route
authorization, CSRF/origin checks, secret handling, redacted diagnostics, edit
history privacy, media cleanup compensation, release retention/pruning,
operator-bridge retirement, static public privacy boundary, and live-smoke
guidance.

| ID | Area | Disposition | Notes |
|----|------|-------------|-------|
| SEC-06-01 | Session-cookie admin auth | Fixed | Collection-management item, image, publication, and publish routes require the HTTP-only admin session created by `/admin/api/login`; bearer-token compatibility is limited to non-management diagnostics. |
| SEC-06-02 | Public account surface | Accepted | v1 keeps a single-admin private controller and does not add public accounts, roles, or anonymous write paths. |
| SEC-06-03 | Route authorization and CSRF/origin checks | Fixed | Browser mutations require a valid session cookie plus configured same-origin `Origin` or `Referer`, and tests cover bearer rejection for collection-management routes. |
| SEC-06-04 | Secret handling | Accepted | Production uses `AUTOGRAPHS_ADMIN_PASSWORD_HASH`; plaintext `AUTOGRAPHS_ADMIN_PASSWORD` is documented as local/ignored-smoke-only, and real credentials stay in GitHub, VM-local, or operator secret stores. |
| SEC-06-05 | Redacted diagnostics | Fixed | Admin health, item responses, publish status, and cleanup warnings avoid bucket names, namespaces, direct Object Storage URLs, raw object keys, Oracle internals, and original private filenames. |
| SEC-06-06 | Edit history privacy | Fixed | Edit history records operational metadata, image events, cleanup events, publication changes, and publish snapshots without exposing private original bytes or direct storage coordinates. |
| SEC-06-07 | Media cleanup compensation | Fixed | Image removal/replacement records cleanup events with retryable private targets, surfaces cleanup warnings to the admin workflow, and keeps public output generated from sanitized derivatives. |
| SEC-06-08 | Release retention/pruning | Fixed | Promoted and failed static release retention are controller-configured, reported through redacted status, and keep the last valid `current` release when candidate validation fails. |
| SEC-06-09 | Operator-bridge retirement | Fixed | The old Node `/api/operator/*` data-entry bridge is documented as retired; routine create/edit/upload/delete/publish work now uses `/admin` and `/admin/api/*`. |
| SEC-06-10 | Static public privacy boundary | Fixed | Public output remains generated static HTML, JSON, and WebP derivatives; private Oracle, Object Storage, object-key, and original-file details stay inside the controller/runtime boundary. |
| SEC-06-11 | Live smoke guidance | Accepted | Local/CI checks verify the ignored live static smoke remains gated by default; operator-run live proof still requires real Oracle, private Object Storage, deployed controller, Caddy preview, and runtime admin credentials. |
| SEC-06-12 | CDN/cache and image-size optimization | Fixed | Phase 6 plan `06-09` reduced detail derivative bounds, added Caddy cache headers, and kept admin/API responses out of cache. Phase 8 now owns the CDN/cache contract before enablement so adjusted media, purge behavior, and rollback are proven together. |
| SEC-06-13 | Post-Phase 6 runtime cleanup posture | Accepted | Deployment docs now include operator-run checks for VM-local image cleanup dry-run, retired service absence, static release retention, failed candidate retention, route shape, and cache headers; destructive live cleanup remains operator-approved only. |

No high-severity Phase 6 admin finding remains without a fixed or documented
mitigation. Phase 7 metadata taxonomy is reviewed below; Phase 8 admin
media/posture planning now covers CDN/cache behavior before enablement, while
Phase 9 taxonomy media cues and Phase 10 AI-assisted ingest remain future scope
and are not implemented by this review.

## Phase 7 Metadata Taxonomy Security Review

Reviewed with ASVS L1 framing for the Phase 7 metadata taxonomy and public
facet changes: admin auth, signer merge tampering, taxonomy input validation,
public static privacy, Oracle migration/backfill, private Object Storage
identifiers, generated artifact fail-closed behavior, and operator review of
live PL/SQL.

| ID | Area | Disposition | Notes |
|----|------|-------------|-------|
| SEC-07-01 | admin auth | Fixed | Signer suggestions, signer profile edits, signer merge repair, taxonomy suggestions, item save, image, publication, and publish routes stay behind the HTTP-only admin session and same-origin mutation checks. Bearer tokens remain diagnostic-only and cannot manage collection metadata. |
| SEC-07-02 | Signer merge tampering | Fixed | Merge repair is a private admin operation, validates source/target signer IDs through the repository boundary, records item-level edit history for affected items, and keeps public output unchanged until an explicit publish. |
| SEC-07-03 | Taxonomy input validation | Fixed | The controller validates required signer credits, format, origin, and supported language values, normalizes string lists, and keeps loose tags secondary instead of using arbitrary tags as primary public facets. |
| SEC-07-04 | Public artifact information disclosure | Fixed | Schema version 2 public JSON/HTML exposes display taxonomy only: signer text/names/roles, public-safe signer links, characters, franchise, productLine, setName, format, origin, language, and tags. It does not expose private Oracle internals, unpublished records, raw image IDs, original filenames, object keys, bucket names, or direct Object Storage URLs. |
| SEC-07-05 | Oracle migration and backfill | Accepted | Rollout requires a generated report, mapping review, generated PL/SQL review, optional SQL Developer application, deploy, full static publish, and verification. Duplicate physical items remain report-only, and legacy fields are retained temporarily for rollback/reference. |
| SEC-07-06 | Migration artifact privacy | Fixed | Backfill mapping/report/PLSQL artifacts are reviewed for credential-like values, Object Storage identifiers, bucket names, object keys, Oracle connection strings, and private keys before use. |
| SEC-07-07 | Live PL/SQL operator review | Accepted | Generated PL/SQL is intentionally not an automatic deployment side effect. The operator may run the reviewed script manually through SQL Developer against live Oracle when ready. |
| SEC-07-08 | Generated artifact fail-closed behavior | Fixed | Static candidates must validate schema version 2 artifacts, required derivatives, manifest entries, and privacy deny-list terms before promotion; failed candidates leave the last valid `current` release in place. |
| SEC-07-09 | Live static smoke taxonomy checks | Fixed | The ignored live smoke remains gated by `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true`; when enabled it publishes through the current session-cookie admin path and checks schema version 2 collection/facets, required taxonomy facets, absence of the category facet, generated derivatives, and stale artifact cleanup. |

No unmitigated high-severity Phase 7 taxonomy finding remains. Phase 8 admin
media/posture work, Phase 9 taxonomy media cues, and Phase 10 AI-assisted
ingest are still future scope and are not implemented by this review.

## Current Verification

Routine checks:

```bash
cargo fmt --manifest-path controller/Cargo.toml --check
cargo test --manifest-path controller/Cargo.toml
cargo check --manifest-path controller/Cargo.toml --features production-persistence
cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings
node --check controller/static-admin/admin.js
cargo test --manifest-path controller/Cargo.toml --features live-persistence live_static_publish_smoke -- --ignored --nocapture
terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/system-cleanup.yml deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-patch-cleanup.yml
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/
```

Phase 6 admin closeout on 2026-07-02 ran the Rust/admin subset of that bundle:

```bash
cargo fmt --manifest-path controller/Cargo.toml --check
cargo test --manifest-path controller/Cargo.toml
cargo check --manifest-path controller/Cargo.toml --features production-persistence
cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings
node --check controller/static-admin/admin.js
cargo test --manifest-path controller/Cargo.toml --features live-persistence live_static_publish_smoke -- --ignored --nocapture
```

All local commands exited zero. The live static publish smoke compiled and
reported the expected default skip because `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE`
was not set to `true`; this is not a claim that the operator-run Oracle/Object
Storage smoke passed.

Live proof:

- Use `docs/static-runtime-runbook.md` for live static publish smoke with real
  Oracle/Object Storage credentials.
- Use `docs/deployment-runbook.md` for deployed controller/static runtime
  verification.
- Use `docs/security-patching.md` for production OS security update scan/apply
  behavior.

## Historical Phase 4 Review

The following findings covered the pre-cutover Next.js runtime. They are kept
for traceability, not as the current implementation map.

Reviewed then:

- Public routes: `/`, `/collection`, `/collection/{id}`, `/architecture`,
  `/api/catalog/*`, `/health`, and `/health/data`.
- Temporary operator routes: `/api/operator/*`.
- Private media delivery through app-mediated catalog image routes.
- Oracle and OCI Object Storage configuration boundaries.
- Caddy public ingress and runtime Podman deployment.
- GitHub Actions CI, deploy, and image-cleanup workflows.
- Repository hygiene for secrets, ignored runtime state, and public documentation.

| ID | Surface | Disposition | Notes |
|----|---------|-------------|-------|
| SEC-04-01 | Public response headers | Fixed historically | Added baseline Next.js headers before static-runtime cutover. |
| SEC-04-02 | Anonymous `/health/data` | Fixed historically | Production anonymous data health omitted detailed config check names and errors. |
| SEC-04-03 | Public operator ingress | Fixed/current | Caddy returns `404` for retired `/api/operator` and `/api/operator/*` paths. |
| SEC-04-04 | Public catalog/media privacy | Replaced by static artifact privacy boundary | Former app-mediated DTO/image-route tests were superseded by static artifact and derivative validation. |
| SEC-04-05 | Temporary operator bridge | Retired/replaced | Former bearer-token bridge is historical; normal admin/publish behavior uses the Rust controller. |
| SEC-04-06 | OCI and runtime secrets | Accepted/current | Secrets are supplied through GitHub Secrets, Terraform variables, VM-local files, and Ansible-managed environment files rather than committed source. |
| SEC-04-07 | CI/CD permissions | Accepted/current | Workflows use explicit permissions for validation, package publish/cleanup, deploy, issue-writing security patch scans, and apply workflows. |
| SEC-04-08 | Runtime image cleanup | Fixed historically | Cleanup behavior was hardened after the multi-tag Podman deletion failure. |
| SEC-04-09 | Static/admin foundation | Complete for Phase 5 foundation | Rust private controller, static publisher, generated derivatives, and operator-bridge retirement landed in Phase 5. |
| SEC-04-10 | Admin workflow security | Superseded by Phase 6 review | Phase 6 admin workflow, edit-history UX, media cleanup ergonomics, cache posture, and runtime cleanup guidance are reviewed above. |
| SEC-04-11 | AI/OCR and taxonomy media security | Deferred to Phases 9 and 10 | Taxonomy thumbnail/media processing, generated public derivatives, and copyright/publication risks remain Phase 9 security work; provider selection, prompts, privacy boundaries, and AI/OCR failure modes remain Phase 10 security work. |
