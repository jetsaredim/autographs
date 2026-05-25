# Phase 4: Public Showcase and Hardening - Research

**Researched:** 2026-05-25
**Domain:** Public-readiness hardening for a self-hosted Next.js/OCI/GitHub Actions repository
**Confidence:** HIGH for codebase findings; MEDIUM for dependency-automation recommendation

## User Constraints

- Scope is the current public-gallery/deployment surface only, before admin and AI work. [VERIFIED: user prompt]
- Produce implementation-ready findings for planning in `.planning/phases/04-public-showcase-and-hardening/RESEARCH.md`. [VERIFIED: user prompt]
- Focus on security/current attack surface, secrets and repo hygiene, dependency automation options, README/badges/public metadata polish, stale-doc/planning artifact cleanup, CI/deploy/readiness checks, and executable plan splitting. [VERIFIED: user prompt]
- Do not design Phase 5 admin workflow or Phase 6 AI features except to note follow-up boundaries. [VERIFIED: user prompt]

## Summary

Phase 4 should be a hardening and presentation pass over an already implemented public gallery and OCI deployment path, not an app re-scope. The current system has strong privacy boundaries: public catalog routes return published-safe DTOs, image delivery is app-mediated, public-surface tests deny private storage leakage, Caddy blocks `/api/operator/*`, and the app container publishes port 3000 only on VM loopback. [VERIFIED: codebase grep]

The highest-value planning work is to close public-readiness gaps: add HTTP security headers, tighten operator/readiness edge cases, document a current attack-surface review, add dependency automation, replace the placeholder README, add badges/public metadata, fix stale Phase 4/AI wording in docs, and create a repeatable release/readiness checklist. [VERIFIED: codebase grep] [CITED: https://nextjs.org/docs/app/api-reference/config/next-config-js/headers] [CITED: https://docs.github.com/en/code-security/reference/supply-chain-security/supported-ecosystems-and-repositories]

**Primary recommendation:** Split Phase 4 into five plans: security hardening, dependency automation, README/public metadata, docs/planning cleanup, and readiness gates.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|--------------|----------------|-----------|
| Public security headers | Frontend Server / Next.js | Caddy | Next.js supports path-scoped response headers in `next.config`; Caddy remains edge routing/proxy. [CITED: https://nextjs.org/docs/app/api-reference/config/next-config-js/headers] |
| Operator-route exposure control | CDN / Edge proxy | API / Backend | Caddy currently returns 404 for `/api/operator/*`; API token checks remain defense-in-depth. [VERIFIED: deploy/ansible/roles/autographs_deploy/files/Caddyfile] |
| Public DTO privacy | API / Backend | Browser / Client | `public-view-models.ts` strips private media fields before route responses/pages render. [VERIFIED: app/src/catalog/public-view-models.ts] |
| Dependency automation | Repository / GitHub | CI | Updates affect npm/pnpm, GitHub Actions, Dockerfile, Terraform, and Ansible collection surfaces. [VERIFIED: codebase grep] |
| README/badges/public metadata | Repository docs | GitHub project settings | The current root README is only `# autographs`; public showcase context belongs at repo root. [VERIFIED: README.md] |
| Readiness checks | CI/CD | Runtime VM | PR CI, deploy health, manual data smoke, and image cleanup already exist and should be formalized into a gate. [VERIFIED: .github/workflows/*.yml] |

## Project Constraints (from AGENTS.md)

- Use the existing single full-stack Next.js app; do not split frontend/backend services. [VERIFIED: AGENTS.md]
- Prefer OCI Always Free primitives, Oracle Autonomous Database Free, and private OCI Object Storage. [VERIFIED: AGENTS.md]
- Keep images private and app-mediated; do not expose public Object Storage URLs. [VERIFIED: AGENTS.md]
- Auto-deploy from GitHub Actions on merge to `main`; CI/CD is core project scope. [VERIFIED: AGENTS.md]
- Use existing service/repository/media boundaries; keep route/page files thin. [VERIFIED: AGENTS.md]
- The app uses native CSS; Phase 4 should not introduce Tailwind, shadcn, gradients, or icon libraries for doc/hardening work. [VERIFIED: AGENTS.md]
- Keep public DTOs free of private storage identifiers. [VERIFIED: AGENTS.md]
- Temporary operator APIs are token-guarded and blocked at public Caddy; do not present them as the v1 admin UX. [VERIFIED: AGENTS.md]
- Use Node's built-in test runner through `node --import tsx --test src/**/*.test.ts`; keep privacy regression tests mandatory for public-surface changes. [VERIFIED: AGENTS.md]
- Do not introduce public accounts, multi-admin roles, direct Object Storage URLs, or split services. [VERIFIED: AGENTS.md]

## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SHIP-01 | Security and attack-vector review for current app, operator, media, secrets, infra, CI/CD, containers, repo settings | Attack-surface map and hardening findings below. [VERIFIED: .planning/REQUIREMENTS.md] |
| SHIP-02 | Dependency update automation for packages, actions, containers, Terraform, maintained surfaces | Dependabot recommended, Renovate noted as fallback if GHCR/Caddy/Ansible coverage needs regex managers. [CITED: GitHub/Renovate docs] |
| SHIP-03 | Root README explains goals, architecture, local dev, deployment, ops, human+AI collaboration | README is currently placeholder only. [VERIFIED: README.md] |
| SHIP-04 | Badges/public metadata reflect quality signals | CI/deploy/data-smoke/image-cleanup workflows exist and can supply badge targets. [VERIFIED: .github/workflows/*.yml] |
| SHIP-05 | Loose ends, stale docs, planning artifacts, warnings are triaged/fixed/tracked | Stale docs include Phase 4/admin wording and architecture diagram AI claims. [VERIFIED: docs/] |

## Standard Stack

### Current Core

| Surface | Current Version / Shape | Purpose | Planning Guidance |
|---------|--------------------------|---------|-------------------|
| Next.js | `16.2.4` in repo; npm current `16.2.6` | App Router public pages/API routes | Keep; add hardening in `next.config.ts` and existing routes. [VERIFIED: app/package.json] [VERIFIED: npm registry] |
| React | `19.2.5` in repo; npm current `19.2.6` | UI rendering | Keep; updates should be automated and validated. [VERIFIED: npm registry] |
| pnpm | `11.2.2` in repo; npm current `11.3.0` | Package manager via Corepack | Keep; configure dependency automation for root/app package files and lockfile. [VERIFIED: package.json] [VERIFIED: npm registry] |
| Terraform | workflow pins `1.15.2`; local `1.15.4`; constraint `< 1.16.0` | OCI infra | Keep pinned in workflows; automate provider and version review. [VERIFIED: .github/workflows/ci.yml] |
| GitHub Actions | `checkout@v6`, setup-node@v6, docker actions, Terraform, Ansible | CI/deploy/image cleanup | Keep least-privilege `permissions`; add automation and readiness docs. [VERIFIED: .github/workflows/*.yml] |
| Caddy | `caddy:2-alpine` from Ansible defaults | Public HTTP/S edge | Keep operator block and add header checks at app/edge as appropriate. [VERIFIED: deploy/ansible] |

### Dependency Automation Recommendation

Use Dependabot first for Phase 4 because it is native to GitHub and supports the repo's main ecosystems: npm/pnpm via `package-ecosystem: npm`, GitHub Actions, Docker, and Terraform. [CITED: https://docs.github.com/en/code-security/reference/supply-chain-security/supported-ecosystems-and-repositories]

Use Renovate only if planning needs better coverage for Ansible collections, Caddy image variables, or custom regex-managed values; Renovate documents managers for npm, dockerfile, github-actions, terraform, and ansible/ansible-galaxy. [CITED: https://docs.renovatebot.com/modules/manager/]

**Installation:** No runtime package installation is required for Phase 4 dependency automation; add `.github/dependabot.yml` or `renovate.json`. [ASSUMED]

## Package Legitimacy Audit

No new application/runtime npm package should be installed for this phase. [VERIFIED: research scope] Existing package versions were checked with `npm view`; no slopcheck gate is required unless the plan later introduces new packages. [VERIFIED: npm registry]

| Package | Registry | Current in repo | Current registry | Source Repo | Disposition |
|---------|----------|-----------------|------------------|-------------|-------------|
| `next` | npm | 16.2.4 | 16.2.6 | vercel/next.js | Existing; update via automation only |
| `react` | npm | 19.2.5 | 19.2.6 | facebook/react | Existing; update via automation only |
| `oci-sdk` | npm | ^2.131.2 | 2.132.0 | oracle/oci-typescript-sdk | Existing; update via automation only |
| `oracledb` | npm | ^6.10.0 | 6.10.0 | oracle/node-oracledb | Existing; keep |
| `typescript` | npm | 6.0.3 | 6.0.3 | microsoft/TypeScript | Existing; keep |

## Current Attack Surface

| Surface | Current State | Risk | Planning Action |
|---------|---------------|------|-----------------|
| Public pages `/`, `/collection`, `/collection/{id}`, `/architecture` | Anonymous, server-rendered/gallery UI. [VERIFIED: app/app] | Public content can leak stale/private metadata if DTO boundary regresses. | Preserve `public-surface.test.ts`; add README/docs checklist requiring DTO privacy gate. |
| Public catalog API `/api/catalog/*` | Published-only list/detail/image route; image response has `nosniff` and short cache. [VERIFIED: app/app/api/catalog] | Lack of explicit global security headers; possible unbounded filter inputs. | Add Next headers and lightweight input/query validation tests. |
| Data health `/health/data` | Non-live route returns config readiness details; live route requires operator bearer token. [VERIFIED: app/app/health/data/route.ts] | Public config error strings can reveal runtime configuration names/states. | Decide whether non-live data health should be public, reduced, or blocked by token in production. |
| Operator API `/api/operator/catalog/*` | Token-guarded in app and blocked by public Caddy. [VERIFIED: app/app/api/operator] [VERIFIED: Caddyfile] | Plain equality token check, no rate limit, upload size/content validation gaps; acceptable only because route is tunnel-only. | Keep blocked; add test/static check for Caddy operator block; document Phase 5 retirement. |
| Media storage | Private Object Storage behind `PrivateMediaStore`; public routes expose app IDs only. [VERIFIED: app/src/media] | MIME/content trust and cache headers matter for uploaded files. | Keep `X-Content-Type-Options`; consider allow-listing image content types on operator upload. |
| CI/CD | PR CI uses read-only contents; deploy uses packages write/actions write; image cleanup uses packages write. [VERIFIED: .github/workflows] | Secrets in deploy, long-lived OCI API key, third-party actions by tag. | Record least-privilege review; consider OIDC future as tracked follow-up, not Phase 4 blocker. |
| Runtime VM | Caddy public, app loopback, secrets/wallet mounted read-only. [VERIFIED: deploy/ansible] | VM-local ignored state/secrets must stay out of repo; Caddy security headers not explicit. | Add readiness checklist and header verification command. |
| Repository hygiene | `.gitignore` excludes tfstate, tfvars, overrides; ignored local `backend_override.tf`, `.terraform/`, `terraform.tfstate` exist. [VERIFIED: git status --ignored] | Public repo can confuse operators if ignored local files remain on machine; committed docs must not reference real secrets. | Add public-readiness hygiene task: `git status --ignored`, secret scan, README note that ignored local state is not committed. |

## Architecture Patterns

### System Architecture Diagram

```text
Anonymous browser
  -> Caddy HTTPS edge
    -> blocks /api/operator/*
    -> Next.js app container on 127.0.0.1:3000
      -> public pages and /api/catalog/*
        -> CatalogService
          -> Oracle repository for published metadata
          -> PrivateMediaStore for image bytes
            -> OCI Object Storage private bucket

Operator workstation
  -> SSH tunnel to VM loopback app port
    -> /api/operator/* with bearer token
      -> same CatalogService and PrivateMediaStore
```

### Recommended Project Structure

```text
.github/
├── dependabot.yml          # Phase 4 dependency automation if using Dependabot
├── workflows/              # Existing CI/deploy/readiness signals
README.md                   # Public showcase entry point
docs/
├── deployment-runbook.md   # Fix phase numbering and readiness gates
├── temporary-production-data-entry.md # Keep as temporary bridge until Phase 5
└── configuration-contract.md # Current secrets/config contract
app/
├── next.config.ts          # Security headers
├── app/api/catalog/        # Public route hardening only
└── src/gallery/*.test.ts   # Privacy/readiness regression tests
.planning/codebase/
└── *.md                    # Refresh after Phase 4 changes
```

### Pattern 1: Security Headers in Next Config

**What:** Add global public headers via `nextConfig.headers()`: `Strict-Transport-Security`, `X-Content-Type-Options`, `Referrer-Policy`, `Permissions-Policy`, and CSP with `frame-ancestors 'none'` or equivalent. [CITED: https://nextjs.org/docs/app/api-reference/config/next-config-js/headers]

**When to use:** Phase 4 current public routes; avoid admin-specific CSP work until Phase 5. [VERIFIED: roadmap boundary]

### Pattern 2: Preserve Edge and App Defense in Depth

**What:** Keep Caddy returning 404 for `/api/operator/*` while app routes also require `AUTOGRAPHS_OPERATOR_API_TOKEN`. [VERIFIED: Caddyfile] [VERIFIED: operator route files]

**When to use:** Until Phase 5 replaces/retire the temporary operator bridge. Do not add public UI affordances that call these endpoints. [VERIFIED: .planning/ROADMAP.md]

### Pattern 3: Readiness Gates as Scripts/Docs, Not New Architecture

**What:** Reuse existing commands and workflows: `pnpm lint`, `pnpm typecheck`, `pnpm test`, `pnpm build`, Terraform fmt/validate/plan, Ansible syntax/lint, deploy `/health`, manual Data Smoke. [VERIFIED: package.json] [VERIFIED: .github/workflows]

**When to use:** Phase 4 release/readiness checklist and CI improvements.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Dependency update PR generation | Custom update scripts | Dependabot first; Renovate if broader manager coverage is required | Native tooling already covers most repo surfaces. [CITED: GitHub/Renovate docs] |
| Secret detection | Ad hoc grep-only review | Add a documented secret-scan command/tool in plan; keep grep as supplementary | Secrets can appear in multiple formats; grep misses entropy-based leaks. [ASSUMED] |
| HTTP security header framework | Middleware from scratch | `next.config.ts` `headers()` and/or Caddy header directives | Existing stack supports response headers. [CITED: Next.js docs] |
| Admin authentication | Temporary operator token upgrades | Defer to Phase 5 | Phase 4 boundary explicitly excludes admin workflow design. [VERIFIED: user prompt] |
| AI/OCR metadata flow | Prompt/provider scaffolding | Defer to Phase 6 | Phase 4 boundary explicitly excludes AI features. [VERIFIED: user prompt] |

## Common Pitfalls

### Stale Phase Numbering

**What goes wrong:** Docs say Phase 4 replaces the temporary operator bridge with admin auth, but admin workflow is now Phase 5. [VERIFIED: docs/temporary-production-data-entry.md] [VERIFIED: docs/deployment-runbook.md]

**How to avoid:** Search docs/planning for `Phase 4 admin`, `Phase 4 auth`, `AI metadata processing`, and update only current-surface language.

### Treating `/health/data` as Harmless

**What goes wrong:** Public non-live data health currently returns detailed configuration-readiness errors. [VERIFIED: app/app/health/data/route.ts]

**How to avoid:** Plan a decision/fix: reduce production output, token-guard all data health, or document why it is intentionally exposed.

### Depending on Caddy Alone for Operator Safety

**What goes wrong:** Local/tunnel paths still reach operator routes. [VERIFIED: docs/temporary-production-data-entry.md]

**How to avoid:** Preserve bearer-token checks, add constant-time token comparison if desired, validate content types/sizes, and keep docs explicit that this is temporary.

### Public README Overpromising Future Features

**What goes wrong:** Public docs present admin/AI as implemented. `docs/architecture.drawio` already mentions AI metadata processing though no AI integration exists. [VERIFIED: docs/architecture.drawio] [VERIFIED: .planning/codebase/INTEGRATIONS.md]

**How to avoid:** README should separate implemented, planned, and intentionally out-of-scope sections.

## Code Examples

### Security Headers Skeleton

```typescript
// Source: https://nextjs.org/docs/app/api-reference/config/next-config-js/headers
const nextConfig: NextConfig = {
  reactStrictMode: true,
  output: "standalone",
  async headers() {
    return [
      {
        source: "/:path*",
        headers: [
          { key: "X-Content-Type-Options", value: "nosniff" },
          { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
          { key: "Permissions-Policy", value: "camera=(), microphone=(), geolocation=()" },
          { key: "Content-Security-Policy", value: "frame-ancestors 'none'; default-src 'self'" },
        ],
      },
    ];
  },
};
```

### Dependabot Skeleton

```yaml
# Source: https://docs.github.com/en/code-security/reference/supply-chain-security/supported-ecosystems-and-repositories
version: 2
updates:
  - package-ecosystem: npm
    directory: /
    schedule:
      interval: weekly
  - package-ecosystem: github-actions
    directory: /
    schedule:
      interval: weekly
  - package-ecosystem: docker
    directory: /app
    schedule:
      interval: weekly
  - package-ecosystem: terraform
    directory: /infra/terraform
    schedule:
      interval: weekly
  - package-ecosystem: terraform
    directory: /infra/terraform/tenancy
    schedule:
      interval: weekly
```

## Plan Split Recommendation

| Plan | Goal | Key Tasks | Verification |
|------|------|-----------|--------------|
| 04-01 Security and attack-surface hardening | Close current public/operator/media/readiness risks | Add headers; review `/health/data`; add operator/Caddy static regression; document attack-surface decisions | App tests, build, header curl check, Caddy route check |
| 04-02 Dependency automation and supply-chain hygiene | Configure update automation and review CI permissions | Add Dependabot or Renovate; group schedules; document update policy; review third-party actions and workflow permissions | Automation config validates; actionlint CI; docs updated |
| 04-03 README, badges, public metadata | Turn repo root into showcase entry point | Rewrite README; add badges for CI/deploy/data smoke/manual where appropriate; add architecture/current status; add human+AI collaboration story | Markdown review; links/badges resolve |
| 04-04 Stale docs and planning cleanup | Make docs coherent after phase reorder | Fix Phase 4/5 wording; remove AI-as-current diagram claims; update codebase maps; capture open follow-ups | `rg` stale-term audit; docs reviewed |
| 04-05 Readiness gate and final public-readiness audit | Prove repo is ready to make public or track exceptions | Run local validation, secret/repo hygiene scan, deploy/readiness checklist, open issue list if needed | Checklist complete; no untracked committed-risk files; manual smoke status documented |

## Validation Architecture

Nyquist validation is disabled in `.planning/config.json`, so no separate Nyquist section is required. [VERIFIED: .planning/config.json]

| Gate | Command / Check | Applies To |
|------|------------------|------------|
| App lint | `corepack pnpm --filter app lint` | README-linked quality badge and app hardening |
| Typecheck | `corepack pnpm --filter app typecheck` | Route/config changes |
| Unit/privacy tests | `corepack pnpm --filter app test` | Public DTO/media/operator boundary regressions |
| Build | `corepack pnpm --filter app build` | Next headers/config changes |
| Terraform | `terraform -chdir=infra/terraform fmt -check -recursive -list=true -diff && terraform -chdir=infra/terraform validate` | Infra/deploy doc changes where applicable |
| Ansible | `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check ...` | Deploy/Caddy changes |
| Public health | `curl --fail --silent https://$AUTOGRAPHS_DOMAIN/health` | Runtime readiness |
| Operator block | `curl -i https://$AUTOGRAPHS_DOMAIN/api/operator/catalog` expects 404 | Public edge safety |
| Data smoke | Manual `.github/workflows/data-smoke.yml` | Live Oracle/Object Storage proof |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|-------------|-----------|---------|----------|
| Node.js | App validation | yes | 22.22.2 | GitHub Actions setup-node uses Node 24 |
| pnpm/Corepack | App validation | yes | pnpm 11.2.2 | CI setup action |
| Terraform | Infra checks | yes | 1.15.4 local | GitHub Actions pins 1.15.2 |
| Ansible | Deploy syntax/lint | yes with temp override | core 2.19.0 | Set `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local` in sandbox |
| ansible-lint | Deploy lint | yes with temp override | 25.6.1+really25.2.1 | Same temp override |
| Docker | Image build | yes | 29.5.2 | GitHub Actions Buildx |
| actionlint | Local workflow lint | no | — | Existing CI uses `raven-actions/actionlint@v2` |

**Missing dependencies with no fallback:** None for planning.

**Missing dependencies with fallback:** `actionlint` is unavailable locally; CI covers it via a GitHub Action. [VERIFIED: local command] [VERIFIED: .github/workflows/ci.yml]

## Security Domain

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | yes, operator only | Existing bearer token; keep tunnel-only and defer real admin auth to Phase 5. [VERIFIED: operator routes] |
| V3 Session Management | no for current public surface | No public sessions; Phase 5 owns admin sessions. [VERIFIED: roadmap] |
| V4 Access Control | yes | Published-only repository/service reads; Caddy blocks operator routes. [VERIFIED: service/routes/Caddyfile] |
| V5 Input Validation | yes | Add query/upload/content-type constraints where public/operator routes currently trust input. [VERIFIED: route files] |
| V6 Cryptography | yes via platform secrets | Do not hand-roll; keep OCI keys/wallets in GitHub/VM secret stores. [VERIFIED: deploy/docs] |
| V14 Configuration | yes | Add public-readiness checklist for headers, secrets, CI permissions, ignored state. [VERIFIED: repo audit] |

## Open Questions (RESOLVED)

1. **RESOLVED:** Phase 4 should minimize the current public/security surface. The planner should treat `/health/data` public behavior as part of the hardening pass: reduce public detail, token-gate sensitive readiness output, or otherwise ensure anonymous callers cannot learn sensitive configuration state. This is a must-fix class issue if the current output would look unsafe or careless in a public review.
2. **RESOLVED:** Use Renovate, conservatively scoped, per D-04-09 through D-04-11. It should cover package, workflow, container, Terraform, Ansible collection, and pinned tooling/version surfaces where Renovate can reliably detect them. It is dependency automation, not a general security scanner.
3. **RESOLVED:** The public README should present the project as a portfolio-quality showcase with project constraints, current architecture, lifecycle/security posture, and the human+AI/GSD build story. It may mention deployment status and validation signals, but should avoid publishing a live hostname unless that is intentionally safe for the public repo at implementation time.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | No new runtime packages are needed for Phase 4. | Standard Stack | Planner may need package legitimacy gate if it chooses tooling that installs packages. |
| A2 | A documented secret-scan tool should supplement grep. | Don't Hand-Roll | Planner must choose an approved tool or keep this as manual checklist. |

## Sources

### Primary

- `AGENTS.md` - project constraints and workflow guardrails. [VERIFIED: codebase grep]
- `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`, `.planning/STATE.md`, `.planning/PROJECT.md` - Phase 4 scope and boundaries. [VERIFIED: codebase grep]
- `.planning/codebase/*.md` - current architecture, testing, integrations, concerns. [VERIFIED: codebase grep]
- `app/app/api/catalog/*`, `app/app/api/operator/catalog/*`, `app/src/catalog/*`, `app/src/gallery/public-surface.test.ts` - current route/privacy behavior. [VERIFIED: codebase grep]
- `.github/workflows/*.yml`, `.github/actions/setup-node/action.yml`, `.github/docker-bake.hcl` - CI/deploy surfaces. [VERIFIED: codebase grep]
- `deploy/ansible/roles/autographs_deploy/*`, `infra/terraform/*` - runtime/infra surfaces. [VERIFIED: codebase grep]
- npm registry via `npm view` for current package versions. [VERIFIED: npm registry]

### Official Docs

- Next.js `headers` config: https://nextjs.org/docs/app/api-reference/config/next-config-js/headers
- GitHub Dependabot supported ecosystems: https://docs.github.com/en/code-security/reference/supply-chain-security/supported-ecosystems-and-repositories
- GitHub `GITHUB_TOKEN` and permissions: https://docs.github.com/en/actions/concepts/security/github_token and https://docs.github.com/en/actions/using-jobs/assigning-permissions-to-jobs
- Renovate managers: https://docs.renovatebot.com/modules/manager/

## Metadata

**Confidence breakdown:**
- Security/current attack surface: HIGH - verified against route, service, workflow, Caddy, and Ansible files.
- Dependency automation: MEDIUM - official docs verify coverage, but final Dependabot vs Renovate choice depends on desired Caddy/Ansible custom update breadth.
- README/docs cleanup: HIGH - placeholder README and stale docs were verified directly.
- Plan split: HIGH - maps directly to SHIP-01 through SHIP-05.

**Research date:** 2026-05-25
**Valid until:** 2026-06-24 for codebase findings; 2026-06-01 for current package/version recommendations.
