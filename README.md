# Autographs

[![CI](https://github.com/jetsaredim/autographs/actions/workflows/ci.yml/badge.svg)](https://github.com/jetsaredim/autographs/actions/workflows/ci.yml)
[![Deploy](https://github.com/jetsaredim/autographs/actions/workflows/deploy.yml/badge.svg)](https://github.com/jetsaredim/autographs/actions/workflows/deploy.yml)
[![Data Smoke](https://github.com/jetsaredim/autographs/actions/workflows/data-smoke.yml/badge.svg)](https://github.com/jetsaredim/autographs/actions/workflows/data-smoke.yml)
[![Image Cleanup](https://github.com/jetsaredim/autographs/actions/workflows/image-cleanup.yml/badge.svg)](https://github.com/jetsaredim/autographs/actions/workflows/image-cleanup.yml)
![Renovate configured](https://img.shields.io/badge/Renovate-configured-1f8b4c)

Autographs is a production-lean personal autograph collection site. The current application lets anonymous visitors browse published memorabilia, filter the public catalog, open item details, and view private OCI Object Storage images only through app-mediated routes.

The project is also a showcase of lifecycle thinking in a small solo system: architecture, CI/CD, OCI deployment, private media boundaries, security review, dependency automation, and human+AI planning are all part of the repository rather than afterthoughts.

## Current Scope

Implemented:

- Next.js full-stack app with public gallery, filters, detail pages, image viewer, and approved quote states.
- Oracle Autonomous Database metadata access through a typed catalog service.
- Private media storage through OCI Object Storage with app-mediated public image delivery.
- Terraform and Ansible deployment path for an OCI Always Free style runtime.
- GitHub Actions for CI, image build/deploy, manual data smoke, and scheduled image cleanup.
- Current-surface security review and Renovate dependency automation.

Planned, not current:

- Phase 5: single-admin create/edit/publish workflow, real admin authentication, edit history, media cleanup UX, and retirement of the temporary operator bridge.
- Phase 6: advisory OCR/AI metadata suggestions that speed up cataloging while preserving manual control.

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
    -> Next.js app container
      -> public pages and /api/catalog/*
        -> Oracle metadata
        -> private OCI Object Storage media through app routes

Operator workstation
  -> GitHub Actions / SSH tunnel / documented runbooks
    -> Terraform, Ansible, data smoke, and temporary operator procedures
```

Useful docs:

- [Configuration contract](docs/configuration-contract.md)
- [Deployment runbook](docs/deployment-runbook.md)
- [Temporary production data entry](docs/temporary-production-data-entry.md)
- [Security review](docs/security-review.md)
- [Dependency updates](docs/dependency-updates.md)

## Local Development

Requirements:

- Node.js 22 or newer
- Corepack with pnpm 11.3.0

Common commands:

```bash
corepack pnpm install
corepack pnpm --filter app lint
corepack pnpm --filter app typecheck
corepack pnpm --filter app test
corepack pnpm --filter app build
corepack pnpm --filter app dev
```

Local development can use local/mock media and catalog paths where the app supports them. Production Oracle, Object Storage, OCI API keys, wallet material, operator tokens, and deploy SSH keys must stay in local secret stores, GitHub Secrets, or VM-local files. Do not commit real credentials.

## Deployment And Operations

Merges to `main` run the deploy workflow. The workflow builds and publishes app/tools images to GHCR when needed, resolves OCI runtime state with Terraform, deploys with Ansible-managed Podman quadlets, and verifies the public `/health` route through Caddy.

Operational checks:

- CI is automatic on pull requests.
- Deploy runs on pushes to `main` and can be manually dispatched.
- Data Smoke is manual because it proves live Oracle and private media behavior against real credentials.
- Image Cleanup is scheduled and manual; it prunes old GHCR and VM-local images while preserving protected/current images.

## Security And Privacy

The public gallery is intentionally read-only. Public catalog responses use published-safe view models, and public image routes stream media through the app rather than exposing Object Storage URLs or object keys.

Temporary operator APIs exist only as a pre-admin bridge. They require a bearer token in the app and are blocked at the public Caddy edge. Use the documented SSH tunnel procedure until Phase 5 replaces this path with the real admin workflow.

See [Security review](docs/security-review.md) for the current fixed, accepted, and deferred findings.

## Human + AI / GSD

This project is being built with a human+AI workflow using GSD: discussion, phase planning, execution plans, review, validation, and PR-based merge discipline. The point is not to hide the planning process; it is to make the repository legible as a real product lifecycle with constraints, tradeoffs, and follow-through.

The current Phase 4 focus is making the repository safe and clear enough to review publicly before expanding the surface area with admin and AI features.
