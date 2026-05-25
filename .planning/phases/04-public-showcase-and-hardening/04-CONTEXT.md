# Phase 4: Public Showcase and Hardening - Context

**Gathered:** 2026-05-25
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 4 makes the current public-gallery/deployment repository safe, coherent, and credible enough to show publicly before Phase 5 admin workflow and Phase 6 AI-assisted ingest are added.

This phase covers the current public surface, repository presentation, dependency automation, security/readiness checks, public-facing documentation, and planning/codebase doc organization. It does not design or implement real admin workflows, admin authentication, edit-history UI, or AI/OCR ingest features.

</domain>

<decisions>
## Implementation Decisions

### Public Story and README Positioning
- **D-04-01:** The root README should be a polished public showcase that also explains the project constraints and the human+AI build process.
- **D-04-02:** The README should explicitly mention GSD as part of the development process, enough to show how planning, phase work, review, and lifecycle thinking shaped the project.
- **D-04-03:** The public story should avoid overpromising Phase 5 admin or Phase 6 AI features. Current, planned, and out-of-scope capabilities must be clearly separated.

### Hardening Depth and Completion Bar
- **D-04-04:** Phase 4 should combine concrete surface minimization with a strict public-readiness gate.
- **D-04-05:** Phase 4 must fix issues that would look careless to a hiring manager or technical lead, including stale README claims, broken badges, confusing docs, missing lifecycle notes, missing dependency automation, untriaged warnings, unsafe public health detail, secret exposure risk, operator-route exposure, or private media leakage risk.
- **D-04-06:** Tracked exceptions are allowed only when the fix clearly expands into Phase 5 admin or Phase 6 AI scope. Exceptions must be documented as follow-up issues/tasks with rationale.
- **D-04-07:** The security surface should be minimized for the current system: public routes, health/readiness output, Caddy operator blocking, CI/CD permissions, Terraform/Ansible/deploy secrets, Object Storage privacy, and repository hygiene all count.
- **D-04-08:** Phase 4 should include proactive scanning and issue filing, not just passive documentation. The goal is to show full lifecycle ownership: find risks, fix what belongs now, and track what belongs later.

### Dependency Automation
- **D-04-09:** Use Renovate for dependency automation rather than Dependabot or a policy-only approach.
- **D-04-10:** Renovate is preferred because the project wants a stronger lifecycle story and broader/flexible coverage across npm/pnpm, GitHub Actions, Docker/container images, Terraform, Ansible-related dependencies, and custom version surfaces.
- **D-04-11:** Renovate configuration should be understandable and conservative: avoid noisy churn, group related updates where useful, and document how dependency PRs should be reviewed.

### Readiness Standard
- **D-04-12:** "Ready to make public" means obvious issues are resolved, security surface is minimized, proactive scanning exists, and remaining exceptions are deliberately filed/tracked.
- **D-04-13:** This phase is being reviewed as portfolio-quality work for a potential hiring manager or technical lead. Plans should demonstrate lifecycle thinking across code, infrastructure, CI/CD, dependencies, docs, security, and operations.
- **D-04-14:** The final readiness output should make it clear what was checked, what was fixed, what remains intentionally deferred, and why.

### Documentation and Repository Organization
- **D-04-15:** Documentation cleanup should organize docs across the repo, not merely patch individual stale phase numbers.
- **D-04-16:** The repo structure should look sane based on current content: root README, docs, planning artifacts, workflows, app, infra, and deploy surfaces should each have clear purpose and navigation.
- **D-04-17:** Phase 4 should refresh or reconcile docs, diagrams, runbooks, codebase maps, and planning notes as needed so future readers and agents do not see contradictory project states.

### Carried Forward From Earlier Phases
- **D-04-18:** Public image delivery must remain app-mediated through `/api/catalog/{itemId}/images/{imageId}`.
- **D-04-19:** Public pages and APIs must not expose direct Object Storage URLs, object keys, bucket names, namespaces, storage credentials, or browser-visible storage identifiers.
- **D-04-20:** Temporary operator routes remain token-guarded, reachable only through the documented tunnel/procedure, and blocked at the public Caddy edge until Phase 5 replaces the bridge.
- **D-04-21:** Public gallery behavior remains anonymous and published-only.

### the agent's Discretion
- Exact README section order, provided it balances public showcase, technical architecture, project constraints, and human+AI/GSD build story.
- Exact Renovate grouping and schedule, provided it is conservative, clear, and covers the repo's maintained dependency surfaces.
- Exact secret-scan/readiness tooling, provided the plan uses a credible scanner or documented equivalent and does not rely only on ad hoc grep.
- Exact issue/task tracking format for deferred exceptions, provided exceptions are explicit, actionable, and tied to the right future phase.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope and Requirements
- `.planning/ROADMAP.md` — Defines Phase 4 goal, success criteria, dependency on Phase 3, and boundary before admin/AI.
- `.planning/REQUIREMENTS.md` — Defines `SHIP-01` through `SHIP-05` and traceability.
- `.planning/STATE.md` — Records current phase position, recent decisions, and review follow-up state.
- `.planning/PROJECT.md` — Defines project value, constraints, active requirements, and key decisions.
- `.planning/phases/04-public-showcase-and-hardening/RESEARCH.md` — Phase 4 research, attack-surface map, dependency automation recommendation, and plan split recommendation.

### Current Codebase Maps
- `.planning/codebase/ARCHITECTURE.md` — Current app/infra architecture and phase boundary.
- `.planning/codebase/CONCERNS.md` — Current risks, security considerations, fragile areas, and near-term recommendations.
- `.planning/codebase/CONVENTIONS.md` — Current coding, testing, and documentation conventions.
- `.planning/codebase/INTEGRATIONS.md` — OCI, Oracle, Object Storage, GitHub Actions, and operator-route integration map.
- `.planning/codebase/STACK.md` — Current stack and maturity state.
- `.planning/codebase/STRUCTURE.md` — Repository structure and where future work belongs.
- `.planning/codebase/TESTING.md` — Current validation contract and pending coverage buckets.

### Prior Phase Decisions
- `.planning/phases/03-public-gallery-mvp/03-CONTEXT.md` — Public gallery, public media privacy, temporary operator bridge, and public UX decisions.
- `.planning/phases/03-public-gallery-mvp/03-05-SUMMARY.md` — Phase 3 final public-gallery gates and temporary production data-entry documentation.

### Current Implementation and Operations
- `README.md` — Current root README to replace with the public showcase entry point.
- `AGENTS.md` — Project instructions and generated codebase context consumed by future agents.
- `.github/workflows/` — CI, deploy, data smoke, and image cleanup workflows.
- `docs/` — Operator docs, deployment runbooks, configuration contract, architecture diagram, and temporary data-entry docs.
- `app/next.config.ts` — Next.js configuration surface for security headers if changed.
- `app/app/api/catalog/` — Public catalog and image API routes.
- `app/app/health/` — Runtime and data health routes.
- `app/src/gallery/public-surface.test.ts` — Public-surface privacy regression tests.
- `deploy/ansible/roles/autographs_deploy/files/Caddyfile` — Public edge routing and operator-route blocking.
- `infra/terraform/` — OCI infrastructure and Terraform surfaces.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `app/src/gallery/public-surface.test.ts`: Existing privacy regression tests can be extended to protect public-readiness and operator-surface assumptions.
- `deploy/ansible/roles/autographs_deploy/files/Caddyfile`: Current public edge already blocks `/api/operator` and `/api/operator/*`; Phase 4 should preserve and test/document this.
- `.github/workflows/ci.yml`: Existing CI already runs app, Dockerfile, Terraform, Ansible, image, and workflow checks; Phase 4 can use this as badge/readiness evidence.
- `docs/configuration-contract.md`, `docs/deployment-runbook.md`, and `docs/temporary-production-data-entry.md`: Current operator docs provide source material for README and lifecycle documentation.

### Established Patterns
- Planning docs should distinguish current implementation from planned/future phases.
- Public DTOs and public pages should stay free of private storage identifiers.
- Operator mutation routes are a temporary bridge, not public UX.
- The repo favors explicit operator guidance, secret handling boundaries, and validation checklists over implicit tribal knowledge.

### Integration Points
- README and docs changes should link into existing runbooks rather than duplicating all operational detail.
- Dependency automation should live in repository configuration, likely `renovate.json` or equivalent.
- Security headers and public-health behavior, if changed, should be verified through app tests/build and documented readiness checks.
- Public-readiness exceptions should be captured in a committed issue/task list or an equivalent documented tracking surface.

</code_context>

<specifics>
## Specific Ideas

- The README should make a hiring manager or technical lead see the whole lifecycle: product constraints, architecture, CI/CD, cloud deployment, privacy boundaries, security posture, dependency hygiene, operations, and human+AI/GSD workflow.
- Renovate should be presented as part of lifecycle maturity, not as a random bot config.
- Phase 4 should produce a clear readiness story: what was scanned, what was fixed, what remains, and why deferred items belong to Phase 5 or Phase 6.
- Docs should be organized so the root README is the public entry point and deeper `docs/` pages remain operator/runbook references.

</specifics>

<deferred>
## Deferred Ideas

- Real single-admin authentication, collection management UI, edit history UX, operator bridge retirement implementation, and admin-specific security review remain Phase 5.
- AI/OCR providers, prompts, privacy boundaries, and AI-specific eval/security documentation remain Phase 6.

</deferred>

---

*Phase: 4-Public Showcase and Hardening*
*Context gathered: 2026-05-25*
