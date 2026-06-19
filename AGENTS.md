<!-- GSD:project-start source:PROJECT.md -->
## Project

**Autographs**

Autographs is a production-lean personal autograph collection website where you can publish your own signed memorabilia for anonymous public browsing. The current implementation serves a generated static public catalog through Caddy and uses a Rust private controller for admin health, seed/publish operations, Oracle metadata access, private OCI Object Storage media access, generated derivatives, and static release publishing on OCI infrastructure managed through Terraform, GitHub Actions, Ansible, and Podman quadlets.

**Core Value:** A collector can reliably browse and manage a high-quality autograph catalog where private images and useful metadata stay connected end to end.

### Constraints

- **Tech stack**: Use generated static public artifacts plus one Rust private controller for v1 — keeps implementation and operations simpler than a public split-service platform.
- **Cloud**: Prefer OCI Always Free services wherever feasible — the product should be realistic for a fresh low-cost tenancy.
- **Database**: Prefer Oracle Autonomous Database Free — the prompt explicitly selects it unless implementation friction forces a justified fallback.
- **Storage**: Keep autograph originals private in OCI Object Storage — public access should use generated public-safe derivatives rather than direct public buckets.
- **Delivery**: Auto-deploy from GitHub Actions on merge to `main` — CI/CD is part of project bootstrap, not optional polish.
- **Operations**: One developer should be able to understand and run the system — avoid enterprise sprawl and multi-service complexity.
- **Scope**: v1 must stay narrow — no staging environment, no bulk import, no public accounts, and no advanced search platform, but multi-image items and edit history are in scope because they matter directly for managing a personal collection well.
- **Security**: Use least-privilege OCI access and explicit secret handling — routine deploy workflows should not rely on tenancy-wide admin power.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- Rust for the private controller, static publisher, persistence adapters, media handling, routes, and tests.
- Markdown for planning, operator runbooks, and repository documentation.
- HCL for Terraform infrastructure.
- YAML for GitHub Actions and Ansible deployment automation.
- HTML, CSS, and JavaScript for generated/static public and admin surfaces.
- Jinja templates for Ansible-managed runtime files and security patching issue/comment bodies.
## Runtime
- Public runtime: Caddy serves generated static releases.
- Private runtime: Rust controller container runs admin/API and publishing behavior behind Caddy/private routes.
- Containerized controller image published through GitHub/GHCR and deployed to OCI.
## Frameworks
- Rust controller and static publisher under `controller/`.
- Static public and admin assets under `controller/static-public/` and `controller/static-admin/`.
- Oracle-backed catalog service with generated public-safe static output.
- Terraform under `infra/terraform/` for OCI baseline resources and state guidance.
- GitHub Actions for PR validation, image build/publish, deployment, image cleanup, and production security patching workflows.
- Ansible under `deploy/ansible/` for VM runtime configuration.
- Podman quadlets for long-lived controller and Caddy containers on the runtime VM.
## Key Dependencies
- Oracle Autonomous Database Free for catalog metadata.
- OCI Object Storage for private autograph media.
- Local/mock media and catalog modes for local/CI paths without live OCI credentials.
- Caddy as the public HTTP(S) edge and static-file server in front of private controller routes.
- Podman as the container runtime on the OCI VM.
- GHCR as the container image registry.
## Configuration
- `.env.example` documents local/runtime variables.
- `.github/.env.github.example` documents GitHub Actions environment/secret expectations.
- `docs/configuration-contract.md` documents the committed configuration and secret contract.
- Ansible renders deployed controller/Caddy runtime files from `deploy/ansible/roles/autographs_deploy/`.
- Secrets such as Oracle DB password, wallet material, OCI private key, OCI identifiers, GHCR token, deploy SSH key, and operator/admin tokens must be supplied through GitHub/environment/operator secret stores.
## Platform Requirements
- Development uses Rust/Cargo for controller work plus Terraform and Ansible for infrastructure/runtime validation.
- Production targets OCI tenancy resources, Oracle Autonomous Database Free, private OCI Object Storage, and an OCI VM runtime capable of Podman, Caddy, the controller container, and configured swap.
## Project Maturity
- Phase 5 plans 05-01 through 05-06 are done, and the static runtime migration foundation is implemented in code/docs; 05-07 live static publish proof and closure summary remain pending before closing the phase.
- The repository is no longer planning-only; it contains Rust/controller, static public/admin, infrastructure, deployment, maintenance, testing, and operator documentation artifacts.
- Do not re-scaffold the retired Next.js app or infra. Phase 6 owns polished admin workflow after the Phase 5 05-07 proof/closure checkpoint; Phase 7 owns advisory AI-assisted ingest.
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Naming Patterns
- Phase directories use zero-padded numeric prefixes plus kebab-case slugs, for example `.planning/phases/03-public-gallery-mvp/`.
- Phase plan and summary files use `{phase}-{plan}-PLAN.md` and `{phase}-{plan}-SUMMARY.md`.
- Codebase map docs use uppercase concern names in `.planning/codebase/`.
- Rust modules under `controller/src/` use descriptive snake_case names.
- Integration tests live under `controller/tests/`.
- Static public/admin assets live under `controller/static-public/` and `controller/static-admin/`.
- Prefer established domain terms: autograph item, signer, category, tags, primary image, supporting images, publication status, static release, candidate release, generated derivative, private original, admin shell, publisher, controller, edit history, and security patching issue.
## Code Style
- Rust is the active implementation language for runtime behavior.
- Use plain static HTML/CSS/JavaScript for minimal admin/public static surfaces unless a later phase intentionally changes that constraint.
- Keep public static artifacts free of private storage identifiers and unpublished records.
- Keep persistence/media details in controller adapters and service modules, not scattered through route handlers or static assets.
- Keep Ansible playbooks thin and put reusable behavior in roles.
## Import Organization
- Keep Rust module organization explicit and close to the controller/publisher domain.
- Keep static assets decoupled from production secrets and private source identifiers.
## Error Handling
- Public static output should fail closed during generation/validation rather than publish incomplete or privacy-leaking artifacts.
- Controller routes should avoid leaking internal OCI, Oracle, or filesystem details.
- Security patching apply runs must refuse drifted package sets and remove stale approval labels on failure.
## Logging
- No project-specific logging abstraction exists yet.
- Runtime and deploy diagnostics currently rely on Rust controller, Podman, Caddy, GitHub Actions, and Ansible logs.
## Comments
- Add comments only where a non-obvious operational or domain boundary needs context.
- Prefer self-explanatory Rust names and existing controller/repository/media separation over explanatory comments for ordinary flow.
## Testing Habits
- Use Cargo checks for current runtime code: `cargo fmt`, `cargo test`, `cargo check --features production-persistence`, and `cargo clippy`.
- Keep static contract/privacy tests mandatory for public artifact changes.
- Run Ansible syntax/lint checks for deployment, cleanup, and security patching changes.
- Use live smoke workflows/runbooks only for real Oracle/Object Storage verification.
## Documentation Habits
- Distinguish current implementation from planned/future phases.
- Keep operator docs procedural and explicit about manual prerequisites, secret handling, approval labels, and live-smoke requirements.
- Update `.planning/codebase/*` after substantial codebase drift so future agents do not resurrect planning-only assumptions.
## Current Guidance
- Phase 5 foundation is mostly implemented; do not rebuild finished 05-01 through 05-06 work, and treat the Rust/static cutover and Next.js retirement as already implemented. Keep 05-07 live static publish proof and closure summary pending. Phase 6 should add the polished admin collection workflow, edit history, and media cleanup ergonomics on that foundation after the 05-07 checkpoint passes. Phase 7 should add advisory AI-assisted ingest.
- Do not introduce public accounts, multi-admin roles, direct Object Storage URLs, or a split frontend/backend service architecture for v1.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## Pattern Overview
- Autographs is a static-public, Rust-controller system, not a planning-only repository.
- The former active Next.js runtime has been retired from the repository; public behavior lives in generated static artifacts under `controller/static-public/`, and private operator/admin behavior lives in the Rust controller under `controller/`.
- Caddy serves the generated static release, blocks retired operator routes, and routes private `/admin` and `/admin/api/*` traffic to the controller/admin surfaces.
## Layers
- Static public layer: `controller/static-public/` contains generated public HTML, JSON, assets, templates, and public-safe media paths.
- Static admin shell: `controller/static-admin/` contains the minimal private admin seed/publish shell.
- Rust controller: `controller/src/` contains private admin/API routes, auth, catalog/media adapters, static publishing, derivative generation, and release promotion.
- Database layer: `controller/db/schema.sql` contains the Oracle schema used by the Rust controller.
- Infrastructure, delivery, and maintenance layer: `infra/terraform/`, `deploy/ansible/`, and `.github/workflows/` provide OCI infrastructure, runtime VM configuration, Podman quadlets, PR validation, image publishing, deploy, image cleanup, and production security patching.
- Planning and operator documentation: `.planning/`, `docs/`, and `.prompts/` hold GSD state, roadmap, phase artifacts, codebase intelligence, bootstrap/runbook docs, and original product prompt.
## Data Flow
- Anonymous visitors request the public site through Caddy and receive generated static HTML, public-safe JSON, assets, and generated media derivatives.
- Operators use the private admin shell and `/admin/api/*` controller routes for health, minimal seed, and publish operations.
- The controller reads and writes Oracle catalog metadata and private OCI Object Storage media.
- The publisher generates candidate static output inside the runtime/OCI boundary, validates privacy and completeness, then promotes the release.
- GitHub Actions validates changes, builds/publishes the controller image, deploys runtime changes, and runs production maintenance workflows.
## Key Abstractions
- Rust controller routes: private admin/API boundary.
- Static artifact contracts: public-safe gallery/detail/search/facet data and publish manifests.
- Publisher: candidate generation, validation, derivative creation, and release promotion.
- Oracle catalog adapter: metadata persistence for production.
- OCI media adapter: private original media access.
- Security patching role: scan, issue rendering, approval validation, patching, result reporting, and failure cleanup.
## Current Phase Boundary
- Phase 5 plans 05-01 through 05-06 are done, and the Rust/static foundation is present; 05-07 live static publish proof and closure summary remain pending before closing the phase.
- Phase 6 should build the polished admin collection workflow on the Rust/static foundation after the 05-07 checkpoint.
- Phase 7 should add advisory AI-assisted ingest.
- Do not re-scaffold the retired Next.js app or replace the delivery spine.
## Notable Absences
- Polished Phase 6 admin collection workflow is not implemented yet.
- Edit history persistence/rendering is not implemented yet.
- Full media replacement/orphan cleanup ergonomics are not implemented yet.
- AI-assisted metadata suggestions are not implemented yet.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

## Git Commit Branch Guardrails

- Never commit directly to `main` or `master`.
- If work starts on `main` or `master`, create or switch to a dedicated work branch before editing files that will be committed.
- Keep all commits for a task inside the current work branch. Merge back to `main` only through the project's normal PR/merge path.
- If a commit command would run on `main` or `master`, stop and report the current branch plus the branch that should contain the work.

## Connectivity and Publishing Failures

- If `git push`, `git fetch`, `gh`, SSH, DNS, or GitHub API calls fail because of local connectivity, local SSH configuration, credentials, network restrictions, or sandbox/network access, stop and tell the user immediately.
- Do not spend time inventing alternate push/fetch workarounds such as overriding `GIT_SSH_COMMAND`, switching remotes, bypassing SSH config, or retrying through a different protocol unless the user explicitly asks for that approach.
- Prefer a concise report with the failing command, the important error text, and the suggested user-side action. The user can often resolve host connectivity issues faster outside the Codex environment.
- After the user confirms the issue is fixed, retry the original straightforward command.

## Pull Request Review Findings

- When a review agent is deployed on a GitHub PR, every actionable finding from that agent must be written back to the PR as a GitHub comment.
- Prefer inline review comments when a finding maps to a specific changed line; otherwise post a single PR-level review/comment that groups the remaining findings by severity.
- Do not leave review-agent findings only in chat, local files, or agent summaries. The PR must contain the review feedback so it is visible during code review.
- If GitHub comment creation fails because of connectivity, credentials, permissions, or API errors, stop and report the failed command/API action plus the finding text that still needs to be posted.



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
