# Autographs

## What This Is

Autographs is a production-lean personal autograph collection website where you can publish your own signed memorabilia for anonymous public browsing. The current implementation includes a self-hosted `Next.js` application, OCI-backed private image delivery, Oracle Autonomous Database Free metadata storage, GitHub-driven CI/CD, and a deployed public gallery MVP running on OCI infrastructure managed through Terraform, GitHub Actions, Ansible, and Podman quadlets.

## Core Value

A collector can reliably browse and manage a high-quality autograph catalog where private images and useful metadata stay connected end to end.

## Requirements

### Validated

- [x] Phase 1 proved the OCI bootstrap and delivery spine: Terraform-managed baseline, explicit configuration contract, GitHub PR validation, GHCR image publishing, and live deploy from `main` to the OCI runtime.
- [x] Phase 2 proved Oracle-backed catalog persistence, private OCI media storage, app-mediated image delivery, and operator-safe verification seams.
- [x] Phase 3 delivered the anonymous public gallery MVP with searchable published records, detail pages, mediated image access, and privacy regression coverage.

### Active

- [x] Deliver a real end-to-end OCI-hosted personal autograph collection foundation with infrastructure, application scaffold, and deployment automation.
- [x] Support anonymous public browsing with searchable autograph records, private image delivery mediated by the app, and enough metadata to make the collection useful.
- [ ] Support a single-admin collection management workflow with AI-assisted metadata suggestions, multiple images per item, and edit history from v1.
- [ ] Keep the system operable by one developer using OCI Always Free services wherever practical.

### Out of Scope

- Public user accounts and social features — this is a personal collection site, not a platform for community participation.
- Multiple admin accounts or role hierarchies — the product only needs one admin path for the collection owner.
- Bulk import pipelines — these add complexity before the single-item workflow is proven.
- Advanced search beyond metadata filters such as signer, category, and tags — richer discovery can wait until the base catalog is working.
- Moderation systems — there is no public contribution model to moderate in v1.
- Separate frontend and backend services — v1 intentionally uses one `Next.js` full-stack application.

## Context

- The repository now contains a runnable OCI-hosted implementation with completed delivery-spine, data/media-core, and public-gallery phases.
- The deployed platform uses Oracle Cloud Infrastructure with an Always Free bias, including OCI Object Storage for private images and Oracle Autonomous Database Free for metadata.
- GitHub remains the source of truth for delivery, with pull-request validation, GHCR image publishing, and automated deployment on merge to `main`.
- Runtime deployment uses Podman quadlets managed through Ansible rather than compose-style orchestration.
- Public image access remains app-mediated through `/api/catalog/{itemId}/images/{imageId}` routes instead of direct Object Storage URLs.
- Temporary operator-only mutation routes remain intentionally token-guarded and excluded from public ingress until the dedicated Phase 4 admin workflow replaces them.
- The intended product remains a personal collection site rather than a reusable platform, so roadmap choices continue to prioritize collection quality, manageability, and presentation over multi-user extensibility.

## Constraints

- **Tech stack**: Use a single `Next.js` full-stack application for v1 — keeps implementation and operations simpler than a split-service design.
- **Cloud**: Prefer OCI Always Free services wherever feasible — the product should be realistic for a fresh low-cost tenancy.
- **Database**: Prefer Oracle Autonomous Database Free — the prompt explicitly selects it unless implementation friction forces a justified fallback.
- **Storage**: Keep autograph images private in OCI Object Storage — access should be centralized through the app rather than direct public buckets.
- **Delivery**: Auto-deploy from GitHub Actions on merge to `main` — CI/CD is part of project bootstrap, not optional polish.
- **Operations**: One developer should be able to understand and run the system — avoid enterprise sprawl and multi-service complexity.
- **Scope**: v1 must stay narrow — no staging environment, no bulk import, no public accounts, and no advanced search platform, but multi-image items and edit history are in scope because they matter directly for managing a personal collection well.
- **Security**: Use least-privilege OCI access and explicit secret handling — routine deploy workflows should not rely on tenancy-wide admin power.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` as the canonical product brief | The prompt captured concrete scope, architecture direction, and verification expectations before implementation work started | Validated through Phases 1-3 |
| Treat the project as greenfield despite existing planning artifacts | There was no runtime app, infra, or test code to preserve at project start | Validated in implementation |
| Start with GitHub-driven OCI bootstrap and deployment as first-class work | The prompt makes CI/CD and tenancy bootstrap foundational, so later phases should build on that instead of bolting it on | Validated in Phase 1 |
| Bias toward OCI Always Free-compatible primitives and a single `Next.js` app | This matches the product brief and keeps the first release operable for one developer | Validated through deployed runtime |
| Optimize for a personal collection rather than a general user platform | The site is meant to present and manage your own autograph collection, so features like multi-image support and edit history matter more than user systems or social capabilities | Ongoing guiding principle |
| Keep public image delivery app-mediated instead of exposing direct Object Storage URLs | Preserves private-media boundaries and centralized access control | Validated in Phases 2-3 |
| Use token-guarded operator endpoints only as a temporary verification seam | Phase 2 needed safe mutation verification before the Phase 4 admin workflow existed | Temporary bridge until Phase 4 |
| Manage runtime services with Podman quadlets through Ansible | Simplifies long-lived runtime operations compared to compose-style orchestration on the OCI VM | Adopted in runtime deployment |
| Treat multi-image support and edit history as v1 capabilities | These directly improve personal collection quality and manageability | Phase 4 requirement baseline |
| Finish with a public-readiness hardening and showcase phase | Public release quality requires explicit cleanup, security review, dependency hygiene, and presentation work | Captured as Phase 6 |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd-transition`):
1. Requirements invalidated? -> Move to Out of Scope with reason
2. Requirements validated? -> Move to Validated with phase reference
3. New requirements emerged? -> Add to Active
4. Decisions to log? -> Add to Key Decisions
5. "What This Is" still accurate? -> Update if drifted

**After each milestone** (via `/gsd-complete-milestone`):
1. Full review of all sections
2. Core Value check -> still the right priority?
3. Audit Out of Scope -> reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-05-24 after Phase 3 completion and Phase 4 transition readiness review*
