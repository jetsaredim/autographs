# Testing Patterns

**Analysis Date:** 2026-06-19

## Validation Contract

The current validation surface is centered on the Rust controller/static
runtime, Terraform, Ansible, Docker, workflow linting, and secret scanning.

### Primary Commands

```bash
cargo fmt --manifest-path controller/Cargo.toml --check
cargo test --manifest-path controller/Cargo.toml
cargo check --manifest-path controller/Cargo.toml --features production-persistence
cargo build --manifest-path controller/Cargo.toml --features production-persistence
cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings
terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/system-cleanup.yml deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-patch-cleanup.yml
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/
```

Do not use retired pnpm/Next.js commands as current gates.

## Test Scope

### Rust Controller and Static Runtime

Covered areas include:

- Auth and health behavior.
- Static artifact contracts for collection/detail/facet data and manifests.
- Static admin shell behavior.
- Seed content path.
- Publisher generation, validation, and promotion behavior.
- Caddy static route expectations.
- Production-persistence compile coverage.
- Live persistence and live static publish smoke paths where real credentials
  are available.

### Infrastructure and Deployment Validation

Covered areas include:

- GitHub Actions workflow validation.
- Controller Dockerfile lint/build.
- OCI Terraform formatting/validation/plan path.
- Ansible deployment and cleanup validation.
- Production security scan/apply playbook syntax/lint coverage.
- Renovate configuration and dependency policy.

## Current Testing Strategy

- Local/CI checks validate Rust/static behavior without requiring live OCI
  credentials for every run.
- Live smoke runbooks prove Oracle/Object Storage and deployed Caddy/controller
  behavior with real secrets.
- Public output privacy is validated at artifact/publisher boundaries rather
  than through retired app-mediated image routes.
- Deployment validation is tied to the actual runtime model: controller image,
  static release, Caddy, Podman, and health checks.

## Fixtures

- `controller/fixtures/` contains representative catalog fixtures.
- `controller/static-public/` and `controller/static-admin/` provide the static
  surfaces under test.
- Live smoke tests require operator-supplied production-like credentials and
  should not be treated as routine PR checks.

## Coverage Gaps

### Pending Phase 6 Areas

- Polished admin session UX and hardening beyond the Phase 5 foundation.
- Daily-use admin create/edit/publish workflow.
- Edit history persistence and rendering.
- Media replacement/orphan cleanup guarantees.
- Admin route, secret, and edit-history documentation checks.

### Pending Phase 7 Areas

- OCR/AI-assisted ingest validation.
- AI/OCR provider, prompt, failure-mode, privacy-boundary, and configuration
  secret review.

## Practical Guidance

- Treat static artifact privacy checks as mandatory for public output changes.
- Treat production security patching playbook changes like deploy/runtime
  changes, not ordinary docs-only updates.
- Keep local-mode, CI, and live OCI smoke evidence distinct.
- Re-run this map after major Phase 6/7 implementation shifts.

---

*Testing analysis refreshed: 2026-06-19 after Phase 5 static runtime implementation and PR 129 production security patching merge*
