# Technology Stack

**Project:** Autographs
**Researched:** 2026-04-18
**Scope:** v1 stack for a single self-hosted OCI deployment, not a generalized platform menu
**Overall confidence:** MEDIUM-HIGH

## Recommended Stack

This project should ship as one containerized `Next.js` full-stack application running on a single OCI Always Free compute instance, with private images in OCI Object Storage and autograph metadata in Oracle Autonomous Database Free.

That recommendation is driven by the prompt and project constraints, not by theoretical purity:
- v1 explicitly wants one deployable app, one developer operating it, anonymous public browsing, one admin path, and GitHub-driven deployment.
- The supplied Next.js docs support a single `next start` process for all Next.js features, which fits the “one app, one server” requirement cleanly.
- OCI Always Free gives just enough primitives to make this viable without introducing extra paid services or service sprawl.

### Core Framework

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Next.js | `16.x` family | Full-stack web app for public gallery, admin UI, API routes, and image mediation | The project brief already standardizes on one `Next.js` app, and official docs support self-hosting all features behind a single Node.js process. |
| React | `19.x` family | UI runtime used by Next.js App Router | Default pairing for modern Next.js; no reason to diverge for v1. |
| TypeScript | `5.x` family | Application language across app, server code, scripts, and config | Stronger contracts matter because this repo will cross UI, storage, database, and deployment boundaries quickly. |
| Node.js | `22 LTS` family | Runtime for local dev, CI, and production app container | Next.js docs require at least Node `20.9`; using Node `22 LTS` gives headroom while staying mainstream and well-supported. |

### Database

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Oracle Autonomous Database Free | Current Always Free offering | System of record for autograph metadata, admin auth state, and object references | The project brief explicitly prefers it, and Oracle Always Free keeps the cost profile aligned with the repo goals. |
| `node-oracledb` | `6.x` family, thin mode | Direct Oracle access from the Next.js server | Oracle’s thin mode avoids Oracle Client libraries, which materially reduces deploy complexity for a single-container app. |
| SQL-first migrations | Tool choice can be finalized during implementation | Schema evolution and repeatable bootstrap | This project needs explicit DDL control because Oracle compatibility matters more than ORM magic. |

### Infrastructure

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| OCI Compute Ampere A1 | Always Free shape | Host the reverse proxy and the `Next.js` container | Oracle docs explicitly include Ampere A1 in Always Free; it is the simplest durable host for a self-managed Node app. |
| OCI Object Storage | Always Free tier | Private storage for autograph images | The product requires private image storage with app-mediated access; Object Storage is the right primitive. |
| nginx | Stable current package from OCI host OS | Reverse proxy in front of Next.js | Next.js self-hosting docs recommend a reverse proxy such as nginx in front of the app server. |
| Docker | Current stable engine on the host | Container packaging and deployment boundary | The prompt requires a containerized deployment; Docker is the boring, operable choice on one VM. |
| Terraform | `1.9.x` or newer stable `1.x` family | Infrastructure as code for OCI baseline resources | The implementation brief already points to Terraform as the strong default, and it matches the GitHub-driven provisioning requirement. |
| GitHub Actions | Current hosted runners | CI validation and merge-to-main deployment orchestration | Required by the prompt and the simplest fit for this repo’s source-of-truth workflow. |

### Supporting Libraries

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `zod` | `4.x` family | Input validation for admin forms, API handlers, and env parsing | Use everywhere data crosses trust boundaries. |
| `next-safe-action` or equivalent thin action validation helper | Current compatible release | Typed server action ergonomics | Use only if server actions become the dominant write path; skip if simple route handlers stay clearer. |
| `sharp` | Current Next-compatible major | Image normalization, resizing, and thumbnail preparation | Use in the admin ingest path before storing final derivatives or preview assets. |
| `tesseract.js` or external OCR wrapper | Latest compatible major at implementation time | OCR as one leg of the hybrid metadata extraction path | Use for v1 only if local/hosted OCR performance is acceptable on A1; otherwise isolate OCR behind a replaceable service boundary. |
| OpenAI SDK or equivalent LLM client | Current stable major at implementation time | AI-assisted metadata suggestion during upload review | Use only on the admin workflow, never in the public request path. |
| `pino` | `9.x` family | Structured application logging | Use from the start so OCI-hosted debugging stays tractable for one operator. |
| `vitest` | `3.x` family | Fast unit/integration tests for application logic | Use for server utilities, validation, and metadata transforms. |
| Playwright | `1.5x` family | End-to-end checks for admin upload and public browsing | Use for a thin smoke suite that proves the vertical slice works. |

## Prescriptive Decisions

### Use `Next.js` self-hosted on Node, not serverless

Run one production build and one `next start` process behind nginx on the OCI instance. This matches the supplied Next.js official guidance and keeps deployment legible for a solo operator.

Do not design v1 around:
- Vercel-specific platform features
- OCI Functions
- a split frontend/backend service boundary
- multiple independently deployed app containers

Those patterns add operational branching without helping the first release.

### Use Oracle directly from the app server

Connect from the `Next.js` server to Oracle Autonomous Database Free using `node-oracledb` thin mode.

That means:
- no Oracle Client libraries in the container
- no separate API service just to speak to Oracle
- no “temporary SQLite in dev, Oracle later” drift in the real application layer

If a repository abstraction is added, it should exist to keep SQL organized and testable, not to hide Oracle behind a fake portability layer.

### Use app-mediated private image delivery

Store images in a private OCI Object Storage bucket and serve them through the application, with strict route-level control and caching behavior designed intentionally.

For v1, do not use:
- public buckets
- permanent object URLs
- CDN complexity before the private delivery path is proven

This aligns with the prompt and avoids leaking storage structure into the public surface area.

### Prefer one OCI VM over managed service sprawl

Use one Ampere A1 instance for v1 app hosting and deployment reception, plus managed storage/database primitives where OCI Always Free already gives them.

This is the right compromise:
- compute is simple enough to reason about
- Object Storage and Autonomous Database remove the highest-value persistence burden
- the stack still fits a new tenancy and one operator

## What Not To Use For v1

| Category | Do Not Use | Why Not |
|----------|------------|---------|
| Frontend/backend split | Separate React SPA plus API service | Violates the project’s explicit “single `Next.js` app” decision and doubles deployment complexity. |
| ORM-first stack | Prisma as the primary data layer | Oracle support and migration ergonomics are not the safe default here; SQL-first Oracle work is lower risk. |
| Search engine | Elasticsearch, OpenSearch, Meilisearch, Algolia | v1 search only needs metadata filters like signer, category, and tags; a dedicated search system is premature. |
| Auth platform | Multi-user auth SaaS or identity suite | The product needs exactly one admin path, not a generalized account system. |
| Blob storage alternative | Local disk as primary image storage | Breaks portability, backup posture, and the prompt’s OCI Object Storage requirement. |
| Deployment target | Kubernetes, Nomad, or OCI Container Engine | Massive operational overhead for a one-app Always Free deployment. |
| Reverse proxy omission | Exposing `next start` directly to the internet | Next.js docs recommend a reverse proxy such as nginx; skipping it weakens the production baseline. |
| Database fallback by default | PostgreSQL on the VM | Only use as a documented fallback if Oracle implementation friction proves truly blocking during delivery. The planning default should remain Oracle. |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| App framework | Next.js `16.x` | Remix / plain React SPA + API | The brief already chooses `Next.js`, and one integrated app is the lowest-friction route to ship. |
| Production runtime | Node `22 LTS` | Node `20 LTS` | Node `20.9+` is supported, but `22 LTS` is the better forward-leaning default unless a dependency blocks it. |
| Database access | `node-oracledb` thin mode + SQL | Java service or sidecar for Oracle access | Extra process and language complexity with no v1 benefit. |
| Infra host | OCI A1 Always Free VM | OCI App Runner-style abstraction / more managed hosting | The repository needs tenancy bootstrap, reverse proxy control, and low-cost durability from a blank tenancy. |
| Object access pattern | App-mediated private fetch | Signed direct object URLs | Signed URLs can come later if traffic patterns demand it; centralized mediation is simpler and safer for the first release. |
| Admin auth | Single credential path stored in Oracle-backed app config | Full IAM or user directory integration | Grossly oversized for one operator and one admin. |

## Version Guidance

Use these version families when planning the roadmap and initial implementation:

- `Node.js 22 LTS`
- `Next.js 16.x`
- `React 19.x`
- `TypeScript 5.x`
- `node-oracledb 6.x`
- `Terraform 1.9+` stable `1.x`
- `Vitest 3.x`
- `Playwright 1.5x`

Avoid exact patch pinning in planning docs until the scaffold is generated and lockfiles exist. At implementation time, pin exact versions in manifests and lockfiles for reproducibility.

## Installation Shape

Illustrative package set for initial scaffolding:

```bash
# Core app
npm install next react react-dom typescript zod node-oracledb pino sharp

# Testing and quality
npm install -D vitest @playwright/test @types/node eslint prettier
```

Additional packages for OCR and AI integration should be added only when that upload workflow is being built, not during the earliest bootstrap if they would slow the first vertical slice.

## OCI Fit Notes

The selected stack is realistic for the stated OCI constraints, with caveats the roadmap should respect:

- OCI Always Free includes Ampere A1 compute, Object Storage, and Autonomous Database Free options in the home region, which is enough for this v1 architecture.
- Always Free Object Storage limits are small enough that v1 must treat images as curated assets, not a bulk media archive.
- Autonomous Database Free is home-region-only, around 20 GB per database, and lacks long-term/manual backup restore features, so backup/export strategy should be explicit in roadmap planning.
- Because Oracle thin mode avoids client library installation, the app container can stay materially simpler than a thick-client Oracle deployment.

## Recommendation Summary

Build this as:

1. One `Next.js 16` app in TypeScript
2. Running on `Node 22 LTS`
3. Self-hosted on one OCI Ampere A1 Always Free instance
4. Fronted by nginx
5. Using OCI Object Storage for private images
6. Using Oracle Autonomous Database Free via `node-oracledb` thin mode
7. Provisioned with Terraform
8. Validated and deployed by GitHub Actions

That is the most direct stack that satisfies the product brief without inventing platform complexity the repository does not need yet.

## Sources

### Official / supplied evidence

- Next.js official docs, install docs, updated 2026-03-16: minimum supported Node.js version is `20.9`.
- Next.js official docs, self-hosting docs, updated 2026-02-27: self-hosting via Node.js server or Docker is supported, and a reverse proxy such as nginx is recommended in front of the app server.
- Next.js official docs, updated 2026-03-25: a single `next start` process supports all Next.js features.
- Oracle official docs: Always Free includes Ampere A1 compute, Object Storage, and Autonomous AI Database / Autonomous Database Free options in the tenancy home region.
- Oracle official docs: Always Free Object Storage includes 20 GB total and 50,000 API requests per month in Always Free-only state.
- Oracle official docs: Always Free Autonomous Database supports roughly 20 GB per database, is limited to the home region, and does not support long-term/manual backup restore features.
- Oracle official docs for `node-oracledb`: thin mode avoids requiring Oracle Client libraries for direct connections.

### Local project sources

- [PROJECT.md](/home/jgreenwa/dev/git/github.com/jetsaredim/autographs/.planning/PROJECT.md)
- [001-autograph-gallery-bootstrap-do.md](/home/jgreenwa/dev/git/github.com/jetsaredim/autographs/.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md)
- [codebase STACK.md](/home/jgreenwa/dev/git/github.com/jetsaredim/autographs/.planning/codebase/STACK.md)
- [codebase CONCERNS.md](/home/jgreenwa/dev/git/github.com/jetsaredim/autographs/.planning/codebase/CONCERNS.md)
