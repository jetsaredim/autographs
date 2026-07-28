# Testing Patterns

**Analysis Date:** 2026-07-10

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
node --check controller/static-admin/admin.js
terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/system-cleanup.yml deploy/ansible/playbooks/security-scan.yml deploy/ansible/playbooks/security-patch.yml deploy/ansible/playbooks/security-patch-cleanup.yml
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-lint deploy/ansible/
```

Do not use retired pnpm/Next.js commands as current gates.

## Test Scope

### Rust Controller and Static Runtime

Covered areas include:

- Auth and health behavior.
- `auth_and_health`: session-cookie login, bearer rejection for collection
  management, CSRF/origin checks, lockout behavior, and redacted health.
- `admin_workflow`: item list/detail/create/edit, field-level history,
  pending-change status, explicit publish batching, retention status, and
  redacted diagnostics; Phase 7 signer/taxonomy route auth, signer
  suggestions, signer profile edits, signer merge repair, taxonomy suggestions,
  and taxonomy-aware item filters.
- `media_cleanup`: image primary selection, replacement/removal, retryable
  cleanup events, rollback behavior, and cleanup warning visibility.
- Static artifact contracts for collection/detail/facet data and manifests.
- Schema version 2 static contract tests for signer text, signer names, signer
  roles, signer credits, characters, franchises, productLine, setName, format,
  origin, language, tags, and public facet ids.
- `static_admin`: static admin source privacy, same-origin privileged calls,
  shared save/publish action paths, accessibility labels, and no browser-storage
  credential assumptions.
- `seed_content`: local create/upload/publication behavior through the current
  session-cookie route shape.
- `publisher`: generation, validation, promotion, privacy scans, incremental
  stale cleanup, promoted/failed release retention behavior, schema version 2
  public facets, detail signer rows, optional profile links, and default
  language/origin hiding.
- `caddy_static_routes`: public/admin route shape, retired operator blocking,
  localhost preview binding, and Cache-Control header contract.
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
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence live_static_publish_smoke -- --ignored --nocapture`
  is a local/CI compile-and-skip gate unless `AUTOGRAPHS_LIVE_STATIC_PUBLISH_SMOKE=true`
  and real runtime credentials are supplied by an operator.
- Live static publish smoke taxonomy assertions check schema version 2
  collection/facets, signer/franchise/productLine/format/language/origin/role/tag
  facet groups, absence of a category facet, public detail taxonomy values,
  generated WebP derivatives, stale artifact cleanup, and exclusion of private
  Object Storage identifiers or original filenames when the smoke is enabled.
- Public output privacy is validated at artifact/publisher boundaries rather
  than through retired app-mediated image routes.
- Deployment validation is tied to the actual runtime model: controller image,
  static release, Caddy, Podman, and health checks.
- Run Ansible deploy syntax checks when env templates, quadlets, Caddy wiring,
  retention variables, cleanup roles, or security patching roles change.
- Operator-run post-Phase 6 VM cleanup checks are documented in
  `docs/deployment-runbook.md`; local validation does not claim live VM cleanup
  ran.

## Fixtures

- `controller/fixtures/` contains representative catalog fixtures.
- `controller/static-public/` and `controller/static-admin/` provide the static
  surfaces under test.
- Live smoke tests require operator-supplied production-like credentials and
  should not be treated as routine PR checks.

## Coverage Gaps

### Pending Phase 8 Areas

- Security patching scan/apply repair validation, including local syntax checks,
  mocked or live-safe role behavior where practical, and operator-run workflow
  verification for production.
- Admin image preview route/UI validation behind the session-cookie boundary.
- Image adjustment metadata, derivative transform, cache invalidation, static
  contract/privacy, and publish behavior validation.
- Repo-wide posture pass findings should either ship with focused tests/checks
  or be explicitly tracked when they are outside the Phase 8 implementation.
- Phase 9 adds taxonomy thumbnail/media cue validation; Phase 10 adds OCR/AI
  provider, prompt, failure-mode, privacy-boundary, and configuration review.

## Practical Guidance

- Treat static artifact privacy checks as mandatory for public output changes.
- Treat production security patching playbook changes like deploy/runtime
  changes, not ordinary docs-only updates.
- Keep local-mode, CI, and live OCI smoke evidence distinct.
- Re-run this map after major Phase 8 implementation shifts.

---

*Testing analysis refreshed: 2026-07-28 after Phase 8/9/10 roadmap split*
