# Autographs

[![CI](https://github.com/jetsaredim/autographs/actions/workflows/ci.yml/badge.svg)](https://github.com/jetsaredim/autographs/actions/workflows/ci.yml)
[![Deploy](https://github.com/jetsaredim/autographs/actions/workflows/deploy.yml/badge.svg)](https://github.com/jetsaredim/autographs/actions/workflows/deploy.yml)
[![Image Cleanup](https://github.com/jetsaredim/autographs/actions/workflows/image-cleanup.yml/badge.svg)](https://github.com/jetsaredim/autographs/actions/workflows/image-cleanup.yml)
![Renovate configured](https://img.shields.io/badge/Renovate-configured-1f8b4c)

Autographs is a production-lean personal autograph collection site. The current public runtime serves a generated static catalog through Caddy, with the Rust private controller publishing public-safe pages, JSON, and derived media from Oracle metadata and private OCI Object Storage.

The project is also a showcase of lifecycle thinking in a small solo system: architecture, CI/CD, OCI deployment, private media boundaries, security review, dependency automation, and human+AI planning are all part of the repository rather than afterthoughts.

## Current Scope

Implemented:

- Generated static public gallery, filters, detail pages, architecture page, derived media, and approved quote states.
- Rust private controller for admin health, seed/publish operations, Oracle catalog access, private media access, and static release publishing.
- Oracle Autonomous Database metadata access and private OCI Object Storage media.
- Terraform and Ansible deployment path for an OCI Always Free style runtime.
- GitHub Actions for CI, controller image build/deploy, and scheduled image cleanup.
- Current-surface security review and Renovate dependency automation.

Planned, not current:

- Phase 6: polished single-admin collection workflow, edit history, richer media cleanup UX, and daily-use admin ergonomics.
- Phase 7: advisory OCR/AI metadata suggestions that speed up cataloging while preserving manual control.

Out of scope for v1:

- Public user accounts.
- Bulk import.
- A separate search service.
- A staging environment.

## Architecture

```text
Anonymous browser
  -> Caddy HTTPS edge
    -> blocks /api/operator/*
    -> generated static release
      -> public pages, JSON, and derived media

Operator workstation
  -> GitHub Actions / admin shell / documented runbooks
    -> Terraform, Ansible, Rust controller publish, and static release verification
      -> Oracle metadata
      -> private OCI Object Storage media
```

Useful docs:

- [Configuration contract](docs/configuration-contract.md)
- [Deployment runbook](docs/deployment-runbook.md)
- [Temporary production data entry](docs/temporary-production-data-entry.md)
- [Security review and current security posture](docs/security-review.md)
- [Dependency updates](docs/dependency-updates.md)
- [Production security patching](docs/security-patching.md)

## Local Development

Requirements:

- Rust stable toolchain.
- Terraform and Ansible for infrastructure and deployment validation.

Common checks:

```bash
cargo fmt --manifest-path controller/Cargo.toml --check
cargo test --manifest-path controller/Cargo.toml
cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings
cargo check --manifest-path controller/Cargo.toml --features production-persistence
terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff
ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml deploy/ansible/playbooks/system-cleanup.yml
```

Local development can use local/mock media and catalog paths where the controller supports them. Production Oracle, Object Storage, OCI API keys, wallet material, operator tokens, and deploy SSH keys must stay in local secret stores, GitHub Secrets, or VM-local files. Do not commit real credentials.

## Deployment And Operations

Merges to `main` run the deploy workflow. The workflow builds and publishes the Rust controller image to GHCR when needed, resolves OCI runtime state with Terraform, deploys with Ansible-managed Podman quadlets, and verifies the generated static release plus `/admin/api/health` through Caddy.

Operational checks:

- CI is automatic on pull requests.
- Deploy runs on pushes to `main` and can be manually dispatched.
- Live static publish smoke in the static runtime runbook proves Oracle, private media, generated artifacts, and Caddy static serving against real credentials.
- Image Cleanup is scheduled and manual; it prunes old GHCR and VM-local controller images while preserving protected/current images.
- Weekly Security Scan opens or updates managed production security update issues; applying `approved-production-update` triggers the guarded Ansible patch workflow for allowlisted operators.

## Security And Privacy

The public gallery is intentionally read-only. Static catalog JSON and generated pages use published-safe data, and public media is served as generated derivatives rather than exposing Object Storage URLs or object keys.

Retired operator APIs remain blocked at the public Caddy edge. Admin and publish operations use the Rust private controller through `/admin` and `/admin/api/*`.

See [Security review and current security posture](docs/security-review.md) for the current Rust/static runtime posture, historical Phase 4 findings, and follow-up boundaries.

## Human + AI / GSD

This project is being built with a human+AI workflow using GSD: discussion, phase planning, execution plans, review, validation, and PR-based merge discipline. The point is not to hide the planning process; it is to make the repository legible as a real product lifecycle with constraints, tradeoffs, and follow-through.

The current focus is expanding the static Rust/controller runtime into the polished Phase 6 admin workflow and Phase 7 AI-assisted ingest.
