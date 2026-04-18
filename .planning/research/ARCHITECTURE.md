# Architecture Patterns

**Domain:** Self-hosted autograph gallery on OCI
**Researched:** 2026-04-18

## Recommended Architecture

Build v1 as one self-hosted `Next.js` application running on a single OCI Compute instance, with Oracle Autonomous Database Free as the system of record for autograph metadata and private OCI Object Storage for original images. Keep every external boundary narrow: the browser talks only to the `Next.js` app, the app talks to the database and object storage, and GitHub Actions is the only automated deploy actor.

This is the right shape for the current repo because the project is greenfield, the product scope is intentionally narrow, and the operator model is one developer. The supplied platform evidence supports this choice: self-hosted Next.js can run as a single Node.js server or in Docker behind a reverse proxy such as `nginx`, and one `next start` process supports the full framework feature set. That means we do not need a split frontend/backend, separate asset proxy service, queue system, or Kubernetes layer to ship a credible first version.

Use app-mediated image delivery for v1. Object Storage stays private, the database stores stable object references plus image metadata, and all anonymous gallery reads flow through a server route or handler in the Next.js app. That preserves privacy and keeps authorization, caching rules, and auditability centralized in one codebase. It also avoids teaching the browser about bucket layout or OCI credentials.

### Deployment Topology

```text
Browser
  |
  v
nginx reverse proxy on OCI Compute
  |
  v
Single Next.js app container (`next start`)
  |
  +--> Oracle Autonomous Database Free
  |
  +--> OCI Object Storage (private bucket)
  |
  +--> Optional AI/OCR provider for admin-only metadata suggestions

GitHub Actions
  |
  +--> validates repo on PR
  +--> deploys infra/app on merge to main
```

### Component Boundaries

| Component | Responsibility | Communicates With |
|-----------|---------------|-------------------|
| `Next.js` web app | Public gallery pages, item detail pages, admin UI, auth checks, API handlers, image mediation | Browser, database, object storage, optional AI/OCR |
| Public gallery module | Anonymous search/filter/list/detail rendering | App services, database |
| Admin module | Single-admin login, upload, review, publish flow | App services, object storage, database, optional AI/OCR |
| Image delivery handler | Fetches/streams private images from Object Storage with cache headers and access rules | Object storage, browser |
| Metadata service layer | Validates inputs, maps DB rows to app view models, encapsulates queries | Database |
| Storage service layer | Uploads originals, generates object keys, reads blobs/streams, hides OCI SDK details | Object storage |
| Oracle Autonomous Database Free | Stores autograph records, searchable fields, admin data, object references | App only |
| OCI Object Storage private bucket | Stores uploaded autograph images and derived variants if added later | App only |
| OCI Compute host | Runs reverse proxy, app container, and deployment target | GitHub Actions, OCI services |
| GitHub Actions deploy pipeline | Lint/test/build, infra apply, image build/push, remote rollout | GitHub, OCI, compute host |

## Data Flow

### Public gallery read

1. Anonymous user requests gallery or detail page.
2. `Next.js` server queries Oracle Autonomous Database Free for metadata and filter results.
3. Page renders metadata plus app-owned image URLs.
4. Browser requests image from the app, not from Object Storage.
5. Image delivery handler fetches the object from the private bucket and streams it back with conservative cache headers.

### Admin upload and review

1. Admin authenticates through the single admin path.
2. Admin uploads one image and enters or reviews candidate metadata.
3. App stores the original image in the private bucket under a generated stable object key.
4. Optional OCR/AI step proposes fields such as signer, item type, tags, or inscription text.
5. Admin reviews and corrects all suggestions before save.
6. App writes the final metadata row to Oracle Autonomous Database Free, including the object key and searchable fields.

### Deployment flow

1. Pull request runs validation in GitHub Actions.
2. Merge to `main` triggers deploy workflow.
3. Workflow applies infrastructure changes that are safe to automate.
4. Workflow builds and publishes the application artifact, then rolls the OCI Compute host forward.
5. Host restarts the single app container behind `nginx`.

## Patterns to Follow

### Pattern 1: Monolithic Full-Stack Boundary
**What:** Keep UI, admin workflow, data access, auth checks, and image mediation in one `Next.js` repository and one runtime process.
**When:** Throughout v1.
**Why:** Lowest operational overhead, fewest moving parts, easiest local reasoning for a solo operator.

### Pattern 2: Service Adapters Behind Route Handlers
**What:** Route handlers should call internal modules such as `metadata-service`, `storage-service`, and `auth-service` instead of embedding OCI or SQL logic directly.
**When:** Any server action, API route, or server component that touches persistence or external services.
**Example:**
```typescript
export async function GET(_: Request, { params }: { params: { id: string } }) {
  const item = await metadataService.getPublishedAutograph(params.id);
  const image = await storageService.openImageStream(item.objectKey);
  return new Response(image.body, { headers: image.headers });
}
```

### Pattern 3: App-Owned Object Keys
**What:** Treat object keys as internal identifiers generated by the app, not user-provided filenames.
**When:** Every upload.
**Why:** Prevents collisions, reduces path-coupling, and lets metadata evolve without breaking storage layout.

### Pattern 4: Metadata-First Rendering
**What:** Gallery and detail pages should load metadata from the database first and reference images second.
**When:** Public browsing pages.
**Why:** Search, filters, and page content remain fast and structured even if image fetches are slower.

### Pattern 5: Manual Bootstrap, Automated Steady State
**What:** Keep break-glass identity, initial compartment/policy setup, and any home-region-sensitive database provisioning as documented operator steps; automate the rest via GitHub Actions and infrastructure code.
**When:** Initial tenancy setup and later deploys.
**Why:** OCI Always Free database provisioning has home-region constraints, and the highest-risk IAM setup should stay human-audited.

## Suggested Build Order

1. **Delivery spine first**: repo layout, env contract, container shape, GitHub Actions validation/deploy skeleton.
2. **OCI baseline**: network, compute host, bucket, deploy user/policies, and documented manual bootstrap steps.
3. **Database integration**: Oracle Autonomous Database Free connectivity, schema, migrations, and basic health query.
4. **Storage integration**: private uploads and app-mediated download endpoint.
5. **Single-admin auth**: one secure admin path, no public accounts.
6. **Public catalog slice**: seeded list/detail pages with signer/category/tag filtering.
7. **Admin upload flow**: image upload, metadata form, reviewed persistence.
8. **AI/OCR assist**: suggestions only, always human-reviewed before commit.
9. **Hardening**: caching, logging, backup/restore docs, runbooks, and rollout polish.

This order is important. The repo’s current risk is not feature ambiguity; it is the absence of an executable backbone. Shipping deployment, persistence, and storage seams early reduces the chance of discovering OCI or Oracle friction after UI work is already spread through the app.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Splitting v1 Into Separate Frontend and API Services
**What:** Running independent web and backend deployments for the first release.
**Why bad:** Doubles deployment and secret management, adds internal API design work too early, and fights the repo’s explicit single-app constraint.
**Instead:** Keep one `Next.js` app with internal service modules.

### Anti-Pattern 2: Direct Browser Access to Private Object Storage
**What:** Exposing storage URLs, bucket structure, or OCI signing logic directly to the client for standard reads.
**Why bad:** Leaks storage concerns into the frontend, complicates rotation and policy changes, and weakens the privacy boundary.
**Instead:** Route reads through the app and keep Object Storage private.

### Anti-Pattern 3: Treating Image Files as the Source of Truth
**What:** Letting filenames or bucket paths drive app identity and search behavior.
**Why bad:** Makes refactors painful and creates brittle coupling between UI semantics and object layout.
**Instead:** Use the database as the canonical metadata source and store only opaque object references there.

### Anti-Pattern 4: Giving GitHub Actions Broad Tenancy-Wide Power
**What:** Reusing administrator-style credentials for routine deploys.
**Why bad:** High blast radius and poor solo-operator recovery posture.
**Instead:** Use narrowly scoped deploy credentials and keep break-glass access outside the CI path.

### Anti-Pattern 5: Designing for Multi-Tenant or Multi-Admin Needs Now
**What:** Adding roles, organizations, ownership models, or moderation workflows to the base schema.
**Why bad:** Pulls the project away from the validated v1 scope and increases auth complexity before the gallery is proven.
**Instead:** Model exactly one admin identity and anonymous public readers.

### Anti-Pattern 6: Adding OCI Service Sprawl
**What:** Introducing extra managed services for queues, CDN layers, container orchestration, or search before the bottleneck is real.
**Why bad:** Increases cost and operational burden without improving the core autograph workflow.
**Instead:** Start with one compute host, one app, one database, and one private bucket.

## Scalability Considerations

| Concern | At 100 users | At 10K users | At 1M users |
|---------|--------------|--------------|-------------|
| Web serving | Single A1 Flex host is acceptable | Tune Node/process sizing and reverse-proxy caching | Likely needs multiple app instances and externalized session strategy |
| Image delivery | App-mediated reads are fine | Add thumbnail strategy and careful cache headers | Likely needs signed edge delivery or dedicated media tier |
| Database load | Autonomous Database Free is sufficient | Query/index tuning on signer/category/tags becomes important | May outgrow Free tier and require paid Oracle shape or alternative architecture |
| Deploy process | GitHub Actions to one host is simple | Add safer rollout scripting and health checks | Needs blue/green or multi-node rollout |
| Operations | Solo operator runbook is manageable | Monitoring and backup discipline become mandatory | Single-operator model becomes fragile |

For v1, optimize for the left column. The project brief is about proving a real product on a lean OCI footprint, not pre-solving internet-scale problems.

## Solo-Operator Boundary Rules

- Keep all user-facing behavior inside the Next.js repo.
- Keep cloud dependencies to OCI Compute, Object Storage, and Autonomous Database Free.
- Keep every secret contract explicit and environment-driven.
- Keep manual steps limited to the few OCI bootstrap actions that are security-sensitive or region-constrained.
- Keep recovery understandable: one host, one reverse proxy, one app process, one database, one bucket.

## Sources

- Project brief: `.planning/PROJECT.md`
- Implementation prompt: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`
- Current repo architecture analysis: `.planning/codebase/ARCHITECTURE.md`
- Current repo concerns: `.planning/codebase/CONCERNS.md`
- Orchestrator-supplied official platform evidence: Next.js self-hosting supports Node.js server or Docker deployment; reverse proxy such as `nginx` is recommended; a single `next start` process supports all Next.js features when self-hosted; OCI Always Free includes compute, object storage, and autonomous database resources, with Always Free database provisioning constrained to the home region.
