# Codebase Concerns

**Analysis Date:** 2026-04-18

## Tech Debt

**Repository is prompt-defined but still lacks a runnable product skeleton:**
- Issue: The repository's most complete asset is the detailed implementation spec in `.prompts/001-autograph-gallery-bootstrap-do/`. That is valuable because scope, architecture direction, verification expectations, and delivery targets are already captured, but the repo still lacks the infrastructure, application code, workflows, scripts, and docs that would turn that spec into an executable baseline.
- Files: `README.md`, `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`, `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`
- Impact: The prompt reduces product ambiguity, which is a real advantage, but there is still no deployable baseline, no local development entry point, and no foundation for incremental delivery. Every future task remains large and cross-cutting until the prompt is decomposed into committed implementation artifacts.
- Fix approach: Use the prompt as the authoritative product spec, then translate it into the minimum executable baseline first: `package.json`, app scaffold, `.github/workflows/`, infrastructure directory, operator docs, and safe `.env.example` placeholders. Keep the first phase focused on a thin end-to-end slice rather than attempting the whole prompt at once.

**Prompt success criteria describe delivered features that are not yet represented in the repo:**
- Issue: The prompt requires OCI infrastructure, Autonomous Database integration, private object storage, single-admin auth, OCR/AI-assisted upload, public gallery pages, and merge-to-main deployment, but none of the corresponding implementation artifacts exist.
- Files: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md:118`, `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md:164`
- Impact: The repo currently communicates a much more complete target state than the codebase supports, which creates planning risk and makes progress hard to assess objectively.
- Fix approach: Break the prompt into explicit implementation phases with acceptance checks and artifact lists. Land each capability behind committed code and documentation before treating the larger platform story as complete.

**Detailed prompt spec is an asset, but it can become a false proxy for delivery if not operationalized:**
- Issue: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` already captures a strong product and platform brief. That creates an opportunity for consistent implementation decisions, but it also creates a risk that prompt completeness is mistaken for repository completeness.
- Files: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`, `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`
- Impact: Contributors have unusually clear intent to work from, which should accelerate implementation. At the same time, status reporting can drift if prompt artifacts are treated as evidence that infrastructure, CI/CD, docs, and the app itself are already meaningfully underway.
- Fix approach: Treat `.prompts/001-autograph-gallery-bootstrap-do/` as the canonical specification input, then mirror it into code-adjacent artifacts with visible completion markers: roadmap phases, implementation checklists, committed directories, and verification commands tied to real files.

## Known Bugs

**No known runtime bugs are observable because no application or infrastructure implementation is present:**
- Symptoms: There is no executable app, build, deploy script, or Terraform project to run and inspect.
- Files: `README.md`, `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`
- Trigger: Attempting to start, test, or validate the product from the current repository state.
- Workaround: None inside the repo; the missing implementation must be created first.

## Security Considerations

**Secret-handling contract is specified but not yet encoded in repo-safe artifacts:**
- Risk: The prompt requires GitHub Secrets and OCI credentials, but there is no `.env.example`, secret inventory doc, workflow contract, or bootstrap script defining how secrets should be named and consumed.
- Files: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md:42`, `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md:131`
- Current mitigation: The repo does not currently store secret values.
- Recommendations: Add a committed secret contract early in `README.md` or `docs/`, plus `.env.example` with placeholders only. Define which values are GitHub-only, which are local-only, and which one-time OCI bootstrap steps must stay manual.

**High-privilege OCI automation is a design risk until IAM boundaries are concretely documented:**
- Risk: The prompt expects GitHub Actions to provision or update networking, compute, storage, and related resources in a brand-new tenancy, which can easily drift into over-privileged CI if policies are not narrowed first.
- Files: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md:46`, `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md:48`, `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md:100`
- Current mitigation: The prompt explicitly asks for least-privilege policies, but no actual policy code or guidance exists yet.
- Recommendations: Treat IAM/bootstrap documentation and policy-as-code as first-class deliverables before enabling automated deploys from `main`.

## Performance Bottlenecks

**No measured bottlenecks yet, but the planned app-mediated image delivery path is an immediate scale hotspot to design carefully:**
- Problem: The prompt prefers private object storage with app-mediated image delivery, which means the web app becomes part of every image read path.
- Files: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md:93`
- Cause: Image privacy is centralized in the application layer rather than delegated to signed direct storage access.
- Improvement path: Define cache behavior, streaming approach, and object fetch limits in the first implementation pass so private delivery does not accidentally become synchronous, memory-heavy proxying.

## Fragile Areas

**The project depends on a single large prompt artifact rather than executable project structure:**
- Files: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`
- Why fragile: One document currently carries product scope, architecture guidance, deployment expectations, verification requirements, and success criteria. That is a strong starting point, but every implementation effort still has to manually translate the spec into many separate artifacts, which increases drift risk.
- Safe modification: Preserve the prompt as the product specification and decision anchor, but move operational truth into tracked implementation artifacts: roadmap phases, bootstrap docs, workflow files, infra modules, and app scaffolding.
- Test coverage: Not applicable yet; there are no automated tests or validation targets in the repo.

**Repository onboarding is effectively undocumented:**
- Files: `README.md`
- Why fragile: `README.md` contains only the repository name, so a new operator cannot infer project purpose, status, prerequisites, or what is safe to run.
- Safe modification: Expand `README.md` only after a concrete scaffold exists, and keep it aligned with actual commands and directories committed to the repo.
- Test coverage: Not applicable yet.

## Scaling Limits

**Current capacity is zero because no runtime or infrastructure baseline exists:**
- Current capacity: No app routes, no database schema, no storage integration, no CI jobs, and no deployment target are committed.
- Limit: The project cannot serve even a single gallery page or admin upload from the present repository state.
- Scaling path: Establish a minimal vertical slice first, then benchmark the chosen OCI Always Free footprint against real image and metadata flows.

## Dependencies at Risk

**OCI Always Free feasibility is still an unvalidated assumption:**
- Risk: The prompt intentionally targets OCI Always Free resources, including A1 Flex compute and Autonomous Database Free, but the repo contains no proof that the chosen combination will work smoothly with the eventual Next.js, database, OCR, and AI-assisted workflow stack.
- Impact: Future implementation may hit service limits, client-library friction, or operational complexity late, after architecture choices have already spread through the codebase.
- Migration plan: Validate the critical path early with a thin proof of life: one deployed app route, one DB write/read, one private object fetch, and one documented fallback decision if Autonomous Database or OCI-specific tooling proves too heavy for v1.

## Missing Critical Features

**All user-facing and operator-facing implementation artifacts are missing:**
- Problem: The prompt calls for `infra/`, `.github/workflows/`, application source, DB schema/migrations, scripts, docs, `.env.example`, and a real quickstart, but the current repo has none of them.
- Blocks: There is no way to build, test, deploy, review, or operate the autograph gallery from this repository.

**No verification pipeline exists despite CI/CD being part of the stated bootstrap:**
- Problem: The prompt requires validation on pull requests and automatic deployment on merge to `main`, but there are no workflow files or validation commands to enforce that contract.
- Blocks: The project cannot prove correctness, catch regressions, or automate infra/app rollout.

## Test Coverage Gaps

**Entire codebase functionality is untested because the codebase has not been implemented yet:**
- What's not tested: Application runtime, database connectivity, storage access, OCR/AI metadata suggestions, admin authentication, public gallery rendering, Terraform validation, and GitHub Actions behavior.
- Files: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md:134`
- Risk: The prompt sets a high verification bar, but there is no executable target for any of those checks, so future delivery could claim completeness without evidence.
- Priority: High

**Prompt summary overstates readiness relative to the repository state:**
- What's not tested: The summary describes the prompt artifact clearly, but there is no repository-level verification connecting the planned architecture to real commands or files.
- Files: `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`
- Risk: Readers may confuse prompt completion with product completion, causing planning drift and premature confidence.
- Priority: Medium

---

*Concerns audit: 2026-04-18*
