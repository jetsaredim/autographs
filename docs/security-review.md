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
- Phase 7 must review OCR/AI providers, prompts, privacy boundaries, and model
  configuration.

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
| SEC-06-12 | CDN/cache and image-size optimization | Follow-up | Phase 6 plan `06-09` will review Cloudflare/CDN posture, generated image sizing, deployed instance hygiene, and runtime/codebase cleanup before Phase 6 closes. |

No high-severity Phase 6 admin finding remains without a fixed or documented
mitigation. Phase 7 AI-assisted ingest is still future scope and is not
implemented by this review.

## Current Verification

Routine checks:

```bash
cargo fmt --manifest-path controller/Cargo.toml --check
cargo test --manifest-path controller/Cargo.toml
cargo check --manifest-path controller/Cargo.toml --features production-persistence
cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings
terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/system-cleanup.yml deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-patch-cleanup.yml
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/
```

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
| SEC-04-10 | Admin workflow security | Deferred to Phase 6 | Polished admin workflow, edit-history UX, and advanced media cleanup ergonomics remain Phase 6 security work. |
| SEC-04-11 | AI/OCR security | Deferred to Phase 7 | Provider selection, prompts, privacy boundaries, and AI/OCR failure modes remain Phase 7 security work. |
