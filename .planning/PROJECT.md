# Autographs

## What This Is

Autographs is a production-lean personal autograph collection website where you can publish your own signed memorabilia for anonymous public browsing. The current implementation serves a generated static public catalog through Caddy and uses a Rust private controller for admin health, seed/publish operations, Oracle metadata access, private OCI Object Storage media access, generated derivatives, and static release publishing on OCI infrastructure managed through Terraform, GitHub Actions, Ansible, and Podman quadlets.

## Core Value

A collector can reliably browse and manage a high-quality autograph catalog where private images and useful metadata stay connected end to end.

## Requirements

### Validated

- [x] Phase 1 proved the OCI bootstrap and delivery spine: Terraform-managed baseline, explicit configuration contract, GitHub PR validation, GHCR image publishing, and live deploy from `main` to the OCI runtime.
- [x] Phase 2 proved Oracle-backed catalog persistence, private OCI media storage, app-mediated image delivery, and operator-safe verification seams.
- [x] Phase 3 delivered the anonymous public gallery MVP with searchable published records, detail pages, mediated image access, and privacy regression coverage.

### Active

- [x] Deliver a real end-to-end OCI-hosted personal autograph collection foundation with infrastructure, application scaffold, and deployment automation.
- [x] Support anonymous public browsing with searchable autograph records, generated public-safe image derivatives, and enough metadata to make the collection useful.
- [x] Prove a static public catalog and minimal private seed/publish path into Oracle/Object Storage before building the full admin workflow.
- [ ] Support a single-admin collection management workflow with multiple images per item and edit history from v1.
- [ ] Add advisory AI-assisted metadata suggestions after the manual admin workflow exists, without making cataloging depend on AI.
- [ ] Keep the system operable by one developer using OCI Always Free services wherever practical.
- [ ] Keep catalog content generation inside the OCI/runtime boundary so private image identifiers, Object Storage details, Oracle data, and image UUIDs do not flow through GitHub-hosted workflows.

### Out of Scope

- Public user accounts and social features — this is a personal collection site, not a platform for community participation.
- Multiple admin accounts or role hierarchies — the product only needs one admin path for the collection owner.
- Bulk import pipelines — these add complexity before the single-item workflow is proven.
- Advanced search beyond metadata filters such as signer, category, and tags — richer discovery can wait until the base catalog is working.
- Moderation systems — there is no public contribution model to moderate in v1.
- Separate public frontend/backend product surfaces — v1 uses generated static public artifacts plus one private Rust controller, not a public multi-service platform.

## Context

- The repository now contains a runnable OCI-hosted implementation with completed delivery-spine, data/media-core, public-gallery, public-hardening, and static-runtime migration phases; Phase 5 has delivered plans 05-01 through 05-07, live static publish proof, UAT, security review, and verification closeout.
- The deployed platform uses Oracle Cloud Infrastructure with an Always Free bias, including OCI Object Storage for private images and Oracle Autonomous Database Free for metadata.
- GitHub remains the source of truth for delivery, with pull-request validation, GHCR image publishing, and automated deployment on merge to `main`.
- Runtime deployment uses Podman quadlets managed through Ansible rather than compose-style orchestration.
- Public image access now uses generated public-safe derivatives in the static release instead of direct Object Storage URLs or retired app-mediated image streaming routes.
- Retired operator-only mutation routes remain blocked at the public Caddy edge; normal admin and publish operations use the Rust private controller under `/admin` and `/admin/api/*`.
- Phase 5 plans 05-01 through 05-07 delivered the static public runtime foundation, minimal private seed/publish path, Rust controller, generated derivatives, runtime cutover, live static publish proof, and closure evidence. Phase 6 now focuses on polished collection-management ergonomics on that foundation.
- The intended product remains a personal collection site rather than a reusable platform, so roadmap choices continue to prioritize collection quality, manageability, and presentation over multi-user extensibility.

## Constraints

- **Tech stack**: Use generated static public artifacts plus one Rust private controller for v1 — keeps implementation and operations simpler than a public split-service platform.
- **Cloud**: Prefer OCI Always Free services wherever feasible — the product should be realistic for a fresh low-cost tenancy.
- **Database**: Prefer Oracle Autonomous Database Free — the prompt explicitly selects it unless implementation friction forces a justified fallback.
- **Storage**: Keep autograph originals private in OCI Object Storage — public access should use generated safe derivatives in static releases, with private originals mediated by the Rust controller/publisher boundary rather than direct public buckets.
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
| Bias toward OCI Always Free-compatible primitives and a minimal runtime | This matches the product brief and keeps the first release operable for one developer | Validated through deployed runtime |
| Optimize for a personal collection rather than a general user platform | The site is meant to present and manage your own autograph collection, so features like multi-image support and edit history matter more than user systems or social capabilities | Ongoing guiding principle |
| Keep public image delivery mediated instead of exposing direct Object Storage URLs | Preserves private-media boundaries; originally validated through app-mediated image routes in Phases 2-3 and now implemented through generated static derivatives plus the private Rust controller/publisher boundary | Updated for Phase 5 static runtime |
| Use token-guarded operator endpoints only as a temporary verification seam | Phase 2 needed safe mutation verification before the admin workflow existed | Replaced and retired by the Phase 5 Rust private controller and `/admin/api/*` path |
| Manage runtime services with Podman quadlets through Ansible | Simplifies long-lived runtime operations compared to compose-style orchestration on the OCI VM | Adopted in runtime deployment |
| Move public-readiness hardening before admin and AI | The current gallery/deployment system can be made safe and presentable before adding larger private mutation and AI surfaces | Phase 4 focus as of 2026-05-25 |
| Insert Static Runtime Migration Foundation before admin CRUD | Smoke-test and runtime complexity suggest anonymous browsing may be better served as static generated output, while admin/publish work stays private and thin | Phase 5 as of 2026-05-26 |
| Use native OCI instance-principal signing for controller Object Storage access | The dev-node smoke proved OCI instance principal auth plus native Object Storage binary PUT/GET/DELETE against the private media bucket, avoiding S3 Customer Secret credentials, Python SDK drift, and long-lived runtime object credentials | Phase 5 controller media adapter direction as of 2026-06-14 |
| Retire the active Next.js runtime in favor of Caddy-served static output and the Rust controller | Phase 5 implementation proved generated public artifacts, private publishing, Caddy routing, and controller health/persistence paths | Validated through Phase 5 closeout as of 2026-06-20 |
| Close Phase 5 after live static publish proof and verification gates | The live static publish smoke, cleanup verification, UAT, security review, and verification artifacts now prove the static runtime foundation end to end | Phase 5 complete as of 2026-06-20 |
| Add guarded production security patching through GitHub Issues and Ansible | Routine OS security updates need reviewable operator approval, package-set drift refusal, and cleanup of failed approvals | Added after PR 129 merge |
| Treat multi-image support and edit history as v1 capabilities | These directly improve personal collection quality and manageability | Phase 6 requirement baseline |
| Add AI-assisted ingest after admin workflow | AI suggestions should enhance a proven manual admin flow rather than define it | Captured as Phase 7 |

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
*Last updated: 2026-06-20 after closing Phase 5 verification and reconciling planning context*
