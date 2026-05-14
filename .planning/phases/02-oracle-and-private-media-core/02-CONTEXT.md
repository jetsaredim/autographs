# Phase 2: Oracle and Private Media Core - Context

**Gathered:** 2026-05-14
**Status:** Ready for planning

<domain>
## Phase Boundary

Phase 2 proves the durable metadata and private media backbone for the autograph collection. It should add Oracle-backed autograph records, private OCI Object Storage media objects, app-controlled media access, sample data, and verification seams that later gallery and admin phases can reuse.

This phase should not build the final public gallery UI, single-admin authentication path, edit-history UI, or AI metadata suggestion workflow. It may add thin route handlers, scripts, fixtures, and diagnostic/admin-adjacent endpoints only where needed to prove the data and media core without pulling Phase 3, Phase 4, or Phase 5 scope forward.

</domain>

<decisions>
## Implementation Decisions

### Data Model
- **D-02-01:** Oracle Autonomous Database Free remains the target metadata store unless implementation proves it cannot satisfy v1 within reasonable complexity.
- **D-02-02:** The v1 autograph record schema must cover title, signer, description, category, tags, object references, event/source, event/location, inscription text, certification company and ID, estimated year, and publication status.
- **D-02-03:** Multi-image support belongs in the core schema now: one designated primary image plus ordered supporting images for each autograph item.
- **D-02-04:** Edit history is deferred to Phase 4, but Phase 2 schema choices should avoid blocking it later.

### Media Privacy
- **D-02-05:** Autograph image objects must stay private in OCI Object Storage.
- **D-02-06:** The application is the access boundary for media reads; Phase 2 must not introduce public bucket URLs or browser-visible storage credentials.
- **D-02-07:** Media metadata in Oracle should be sufficient to retrieve objects by app-owned bucket/object identifiers without making object keys part of the public contract.

### Local and CI Verification
- **D-02-08:** Phase 2 must keep local verification practical for one developer. If live Oracle or Object Storage cannot be used locally, provide explicit local doubles, seed fixtures, or smoke commands that keep the contract testable.
- **D-02-09:** CI should continue to validate without requiring production secrets, while live deploy verification can use GitHub-provided OCI configuration.
- **D-02-10:** Sample data is a first-class deliverable because later gallery and admin work depends on realistic records and images.

### Infrastructure and Config
- **D-02-11:** Terraform remains the source of truth for OCI resources, including ADB and the private media bucket where feasible.
- **D-02-12:** New database, wallet, bucket, namespace, and media configuration must extend the committed environment/GitHub config contract without committing secrets.
- **D-02-13:** Least-privilege OCI access should continue the Phase 1 pattern: the runtime gets only the object/database access it needs, not broad tenancy permissions.

### the agent's Discretion
- Exact Oracle client library and migration tool, provided they work in the Next.js container and CI path.
- Exact local data-store double for fast verification, provided production code paths remain clearly separated from test/local adapters.
- Whether route handlers are grouped under public `/api/...` paths or internal diagnostic paths, provided Phase 2 does not expose unauthenticated mutation APIs in the deployed app.
- Exact object key structure, provided object keys are deterministic enough for operations and not exposed as public URLs.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Scope
- `.planning/PROJECT.md` — Project intent, OCI Free Tier bias, single-app constraint, and private-media requirement.
- `.planning/REQUIREMENTS.md` — Phase-driving requirements: `DATA-01`, `DATA-02`, `DATA-04`, `MEDIA-01`, `MEDIA-02`, and `MEDIA-03`.
- `.planning/ROADMAP.md` — Phase 2 boundary, success criteria, and downstream dependencies.
- `.planning/STATE.md` — Current focus, Phase 1 completion status, and active concerns.

### Phase 1 Foundation
- `.planning/phases/01-delivery-spine-and-oci-bootstrap/01-CONTEXT.md` — Decisions around OCI bootstrap, runtime shape, and delivery boundaries.
- `.planning/phases/01-delivery-spine-and-oci-bootstrap/01-04-SUMMARY.md` — Delivery spine, config contract, and GitHub Actions deployment behavior.
- `docs/configuration-contract.md` — Current environment, secret, and GitHub variable contract to extend for ADB/Object Storage.
- `docs/deployment-runbook.md` — Live deploy proof and operator workflow.

### Implementation Surfaces
- `app/` — Single Next.js full-stack application.
- `infra/terraform/` — OCI Terraform root and modules to extend for data resources.
- `.github/workflows/` — Validation and deployment workflows that must continue passing without leaking secrets.
- `deploy/compose/compose.prod.yaml` — Runtime container contract for environment variables and deployed app behavior.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Next.js App Router app exists under `app/` with proof-of-life `/` and `/health` routes.
- Terraform baseline exists under `infra/terraform/` with network, compute, IAM, and state-bucket modules.
- Deployment already publishes a container to GHCR and deploys it through the OCI VM runtime.
- The architecture page documents the intended GitHub to OCI, ADB, Object Storage, and app-mediated flow.

### Established Patterns
- Keep the project as one `Next.js` full-stack app rather than splitting services.
- Prefer committed configuration examples and docs, with real secrets in local environment or GitHub Secrets only.
- Validate with focused scripts before relying on live deploy.
- Use Terraform for OCI resources and runbooks for unavoidable operator setup.

### Integration Points
- Database and media configuration must be available to the app container in local, CI-safe, and deployed contexts.
- Terraform outputs should feed GitHub/deploy configuration where possible.
- App-mediated image delivery must work through the Caddy/Next runtime path, not by making Object Storage public.
- Phase 3 gallery work should be able to consume Phase 2 APIs/data functions without rewriting storage or schema decisions.

</code_context>

<specifics>
## Specific Ideas

- Use a migration directory such as `app/db/migrations/` or `app/src/db/migrations/` with a scriptable runner.
- Model `autograph_items`, `autograph_images`, and tag/category support explicitly rather than hiding everything in a JSON blob.
- Include publication status now so public gallery filtering later can rely on the same field.
- Store object metadata such as bucket, namespace, object key, content type, byte size, checksum/etag, sort order, and primary-image flag.
- Provide seed fixtures with a few representative records and placeholder/sample images suitable for local verification.
- Add a streaming media route that retrieves private objects server-side and returns cache-conscious responses without exposing object URLs.
- Keep mutation routes protected by a local/operator-only guard or test harness until the Phase 4 admin auth path exists.

</specifics>

<deferred>
## Deferred Ideas

- Public gallery pages, search UI, and item detail UI are Phase 3.
- Single-admin authentication, final admin create/edit forms, edit history, and orphan cleanup flows are Phase 4.
- OCR and AI metadata suggestions are Phase 5.
- Bulk import remains out of scope for v1.

</deferred>

---
*Phase: 02-oracle-and-private-media-core*
*Context gathered: 2026-05-14*
