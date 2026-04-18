# Autographs

## What This Is

Autographs is a production-lean autograph photo sharing website for collectors and browsers to explore signed memorabilia through a public, anonymous gallery. The first release pairs a single self-hosted `Next.js` application with private OCI Object Storage for images and Oracle Autonomous Database Free for metadata, while also establishing the OCI bootstrap, CI/CD, and operator guidance needed to run it from a nearly blank repository.

## Core Value

A collector can reliably browse and manage a high-quality autograph catalog where private images and useful metadata stay connected end to end.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Deliver a thin but real end-to-end OCI-hosted autograph gallery foundation with infrastructure, application scaffold, and deployment automation.
- [ ] Support anonymous public browsing with searchable autograph records and private image delivery mediated by the app.
- [ ] Support a single-admin upload workflow with AI-assisted metadata suggestions reviewed before save.
- [ ] Keep the system operable by one developer using OCI Always Free services wherever practical.

### Out of Scope

- Public user accounts and social features — v1 is intentionally anonymous and read-only for public users.
- Multiple admin accounts or role hierarchies — the product only needs one admin path for the initial release.
- Bulk import pipelines and multi-image-per-item workflows — these add complexity before the core upload/review loop is proven.
- Advanced search beyond metadata filters such as signer, category, and tags — richer discovery can wait until the base catalog is working.
- Edit history, versioning, and moderation systems — not core to validating the catalog and upload experience.
- Separate frontend and backend services — v1 intentionally uses one `Next.js` full-stack application.

## Context

- The repository is currently prompt-first: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` is the authoritative implementation brief, and `.planning/codebase/` documents that there is not yet a runnable product.
- The desired platform is Oracle Cloud Infrastructure with an Always Free bias, including OCI Object Storage for private images and Oracle Autonomous Database Free as the preferred metadata store.
- GitHub is the intended source of truth for delivery. Validation on pull requests and auto-deploy on merge to `main` are foundational platform requirements, not later enhancements.
- The app should stay simple enough for a solo developer to operate, with clear tenancy bootstrap guidance, least-privilege IAM, narrow network exposure, and explicit secret contracts.
- The prompt already narrows the product direction significantly: anonymous public browsing, one admin, containerized deployment, app-mediated image access, and AI-assisted metadata extraction with human review.

## Constraints

- **Tech stack**: Use a single `Next.js` full-stack application for v1 — keeps implementation and operations simpler than a split-service design.
- **Cloud**: Prefer OCI Always Free services wherever feasible — the product should be realistic for a fresh low-cost tenancy.
- **Database**: Prefer Oracle Autonomous Database Free — the prompt explicitly selects it unless implementation friction forces a justified fallback.
- **Storage**: Keep autograph images private in OCI Object Storage — access should be centralized through the app rather than direct public buckets.
- **Delivery**: Auto-deploy from GitHub Actions on merge to `main` — CI/CD is part of project bootstrap, not optional polish.
- **Operations**: One developer should be able to understand and run the system — avoid enterprise sprawl and multi-service complexity.
- **Scope**: v1 must stay narrow — no staging environment, no bulk import, no public accounts, and no advanced search platform.
- **Security**: Use least-privilege OCI access and explicit secret handling — routine deploy workflows should not rely on tenancy-wide admin power.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Use `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` as the canonical product brief | The repo has no implementation yet, and the prompt already captures concrete scope, architecture direction, and verification expectations | — Pending |
| Treat the project as greenfield despite existing planning artifacts | There is no runtime app, infra, or test code to preserve; the committed assets are planning inputs | — Pending |
| Start with GitHub-driven OCI bootstrap and deployment as first-class work | The prompt makes CI/CD and tenancy bootstrap foundational, so later phases should build on that instead of bolting it on | — Pending |
| Bias toward OCI Always Free-compatible primitives and a single `Next.js` app | This matches the product brief and keeps the first release operable for one developer | — Pending |

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
*Last updated: 2026-04-18 after initialization*
