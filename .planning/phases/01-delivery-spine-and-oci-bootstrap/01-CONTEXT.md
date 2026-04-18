# Phase 1: Delivery Spine and OCI Bootstrap - Context

**Gathered:** 2026-04-18
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 1 establishes the deployable application foundation, OCI bootstrap, Terraform state strategy, repository organization, configuration contract, and GitHub-driven validation/deploy path for the project. It should end with a real proof-of-life deployment through the containerized OCI runtime, but it should not pull Oracle catalog functionality or collection features from later phases forward.

</domain>

<decisions>
## Implementation Decisions

### OCI Bootstrap Ownership
- **D-01:** Manual OCI work should be kept to the absolute minimum required to bootstrap Terraform.
- **D-02:** The long-term operating model is that compartments, IAM, policies, and the rest of the OCI footprint are managed through Terraform.
- **D-03:** If any OCI resources must be created manually to get Terraform running, they should be imported into Terraform state so code becomes the source of truth afterward.

### Repository Organization
- **D-04:** Keep the project in a single repository rather than splitting infra and app/runtime into separate repos.
- **D-05:** Separate concerns by directories inside the repo, such as Terraform, app code, deploy/runtime assets, workflows, and docs.
- **D-06:** Only split repositories later if the infrastructure grows into a broader reusable OCI foundation beyond this project.

### Runtime Deployment Shape
- **D-07:** The Phase 1 runtime should be fully containerized on a single OCI VM.
- **D-08:** The host runtime shape is one `nginx` container proxying to one `Next.js` app container.
- **D-09:** OCI Object Storage and Oracle remain external managed services; `nginx` is only the incoming reverse proxy and does not access Object Storage directly.

### CI/CD Authentication and Secret Model
- **D-10:** Phase 1 should start with OCI API signing keys in GitHub Secrets for deployment authentication.
- **D-11:** Workflows, Terraform, and documentation must treat GitHub-to-OCI authentication as replaceable so the project can move to short-lived or federated auth later without redesigning the delivery spine.
- **D-12:** Sensitive credentials belong in GitHub Secrets, while non-sensitive deployment configuration should live in versioned repo-managed files and GitHub Variables rather than being hidden in secrets.

### Terraform State Strategy
- **D-13:** Terraform state should live in OCI Object Storage using the Terraform `oci` backend, not in GitHub Secrets.
- **D-14:** The remote state bucket should have versioning enabled.
- **D-15:** Local state is acceptable only as a short bootstrap step until the remote backend exists and state is migrated.

### Phase 1 Done Boundary
- **D-16:** Phase 1 should deliver more than documentation and bootstrap code alone; it must prove the deployment path end to end.
- **D-17:** Phase 1 is complete when the OCI baseline, remote Terraform state, repo organization, config contract, GitHub validation/deploy path, and the containerized runtime are in place, plus a minimal deployed `Next.js` proof-of-life app or health page is running through `nginx` on OCI.

### the agent's Discretion
- Exact directory names inside the single repo, as long as app, infra, deploy/runtime assets, workflows, and docs are clearly separated.
- Exact naming of GitHub secrets and variables, as long as the contract is explicit and consistent.
- Whether the app proof-of-life is a dedicated health route, a minimal landing page, or both.
- Exact container orchestration mechanism on the VM, as long as it preserves the locked two-container runtime shape.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Scope
- `.planning/PROJECT.md` — Project intent, non-negotiable constraints, and the personal-collection framing.
- `.planning/REQUIREMENTS.md` — Phase-driving requirements, especially `PLAT-01`, `PLAT-02`, and `PLAT-03`.
- `.planning/ROADMAP.md` — Phase boundary, success criteria, and ordering constraints for Phase 1.
- `.planning/STATE.md` — Current progress, known blockers, and current focus.

### Research Inputs
- `.planning/research/SUMMARY.md` — Phase ordering rationale and recommended delivery spine strategy.
- `.planning/research/STACK.md` — Recommended stack, OCI runtime shape, and state/backend direction.
- `.planning/research/ARCHITECTURE.md` — Recommended deployment topology and build order.
- `.planning/research/PITFALLS.md` — Risks around over-privileged CI/CD, bootstrap ambiguity, and early delivery shape mistakes.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- No runtime application code exists yet; this phase is expected to create the first real implementation assets.

### Established Patterns
- Planning artifacts are the current source of truth, especially the prompt-first repo structure and `.planning/` documents.
- The repo has already standardized on a single `Next.js` app, OCI Always Free bias, app-mediated private media, and GitHub-driven deployment.

### Integration Points
- New infrastructure code should align with the Phase 1 scope in `.planning/ROADMAP.md`.
- New deployment and bootstrap work should establish the seams later phases will use for Oracle, Object Storage, and app rollout.
- `AGENTS.md` and the GSD planning docs now define the workflow guardrails that future implementation should respect.

</code_context>

<specifics>
## Specific Ideas

- "Only the absolute minimum should be done manually - as much as possible should be done in code."
- "Anything that needs to be done manually initially should end up being managed via code."
- "Terraform [should] manage all the resources and anything done manually should be imported into the state file."
- "Keeping the stack containerized completely is probably the best."
- GitHub-to-OCI auth should start pragmatically with signing keys, but the design should preserve a later move to federated or short-lived auth.
- Terraform state should live in OCI Object Storage using the `oci` backend with bucket versioning.
- The project should stay in one repo, with infra, app, and deploy assets separated by directories rather than separate repositories.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---
*Phase: 01-delivery-spine-and-oci-bootstrap*
*Context gathered: 2026-04-18*
