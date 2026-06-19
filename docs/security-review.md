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

- Phase 6 must review polished admin workflow, edit-history UX, media cleanup,
  and expanded admin route behavior.
- Phase 7 must review OCR/AI providers, prompts, privacy boundaries, and model
  configuration.

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
