---
phase: 05
slug: static-runtime-migration-foundation
status: verified
threats_open: 0
asvs_level: 1
created: 2026-06-20
---

# Phase 05 Security Verification

This audit verifies only the declared Phase 5 threat model from `05-01-PLAN.md`
through `05-06-PLAN.md`. Implementation files were treated as read-only; this
file is the only artifact written by the audit.

## Trust Boundaries

| Boundary | Verification Focus |
|----------|--------------------|
| Oracle/Object Storage source to public artifacts | Public JSON, HTML, manifests, and media paths must exclude private storage coordinates, private image IDs, object keys, original filenames, and unpublished records. |
| Generated release to anonymous browser | Anonymous users receive generated static files and public-safe derivatives only. |
| Browser/admin shell to Rust controller | Cookie-authenticated browser mutations must use same-origin protections; privileged APIs require session or bearer auth. |
| Local operator to Rust controller | Operator bearer calls are explicit, non-ambient credentials and may bypass browser CSRF checks. |
| Controller config to health/errors | Health and error responses must not echo secrets, Oracle details, Object Storage coordinates, or submitted credentials. |
| Admin API to Oracle/private originals | Authenticated seed/update/upload operations must validate input, ownership, upload type, and size before persistence. |
| Private original metadata to public publish path | Generated derivative names and artifacts must be independent from private object keys, UUIDs, and original filenames. |
| Candidate release to current release | Candidate output must validate before atomic promotion to `current`; failed candidates remain diagnostic-only. |
| Public internet to Caddy | Public traffic reaches static files and admin entry points, not retired operator/catalog APIs or private originals. |
| Caddy to Rust controller | Only `/admin/api/*` is proxied to the controller over the private Podman network. |
| GitHub Actions to runtime | Workflows build/deploy code, images, and runtime configuration; catalog publishing stays inside the runtime/controller boundary. |
| Local candidate validation to public cutover | Candidate/static preview is bound to localhost before public cutover checks. |

## Threat Register

| Threat ID | Category | Component | Disposition | Status | Evidence |
|-----------|----------|-----------|-------------|--------|----------|
| T-05-01 | Information Disclosure | generated JSON/HTML/manifests | mitigate | CLOSED | Deny-list scan in `controller/src/publisher.rs:964`; denied terms in `controller/src/publisher.rs:978`; source-aware image ID, original filename, and object-key scan in `controller/src/publisher.rs:994`; regression coverage in `controller/tests/static_contract.rs:42` and `controller/tests/publisher.rs:84`. |
| T-05-02 | Tampering | public contract versioning | mitigate | CLOSED | `PUBLIC_SCHEMA_VERSION` in `controller/src/contracts.rs:3`; schema fields in catalog/facets/manifest DTOs in `controller/src/contracts.rs:7`, `controller/src/contracts.rs:128`, and `controller/src/contracts.rs:159`; item detail version set in `controller/src/publisher.rs:601`; test assertion in `controller/tests/static_contract.rs:61`. |
| T-05-03 | Denial of Service | oversized fixture artifacts | mitigate | CLOSED | 500-item fixture assertion in `controller/tests/static_contract.rs:12`; single/split/hybrid byte profile output in `controller/tests/static_contract.rs:15`; documented profile in `docs/static-artifact-contract.md:32`. |
| T-05-SC | Tampering | cargo dependencies | mitigate | CLOSED | Constrained direct dependencies in `controller/Cargo.toml:19`; optional production-persistence feature gating in `controller/Cargo.toml:6`; tracked lockfile present via `controller/Cargo.lock:1`; no project `build.rs` generator hook was found. |
| T-05-04 | Spoofing | admin login | mitigate | CLOSED | Single auth state sources in `controller/src/auth.rs:19`; password/hash verification in `controller/src/auth.rs:97`; session cookie flags in `controller/src/routes.rs:710`; login/cookie tests in `controller/tests/auth_and_health.rs:84`. |
| T-05-05 | Elevation of Privilege | `/admin/api/*` | mitigate | CLOSED | Admin routes are declared in `controller/src/routes.rs:196`; mutation guard in `controller/src/routes.rs:577`; publish status auth in `controller/src/routes.rs:570`; create/update/upload/publication/publish handlers call the guard at `controller/src/routes.rs:297`, `controller/src/routes.rs:332`, `controller/src/routes.rs:366`, `controller/src/routes.rs:470`, and `controller/src/routes.rs:536`. |
| T-05-06 | Information Disclosure | health/errors | mitigate | CLOSED | Admin health returns readiness booleans/provider labels only in `controller/src/routes.rs:221`; health redaction test denies Oracle/OCI/bucket/object/secret strings in `controller/tests/auth_and_health.rs:40`; public item response excludes object key, bucket, namespace, and original filename fields in `controller/src/routes.rs:594`. |
| T-05-07 | Denial of Service | login endpoint | mitigate | CLOSED | Failed-login threshold and lockout in `controller/src/auth.rs:15`; lockout state update in `controller/src/auth.rs:62`; deterministic test in `controller/src/auth.rs:126`. |
| T-05-08 | Cross-Site Request Forgery | cookie-authenticated admin mutations | mitigate | CLOSED | Cookie mutations require same-origin `Origin` or `Referer` unless using bearer auth in `controller/src/routes.rs:677`; cross-origin and same-origin tests in `controller/tests/auth_and_health.rs:110`. |
| T-05-08 | Tampering | seed/update API | mitigate | CLOSED | Publication status enum restricts allowed values in `controller/src/catalog.rs:10`; required field validation in memory and Oracle repositories in `controller/src/catalog.rs:110` and `controller/src/oracle_catalog.rs:400`; upload verifies item ownership before media write in `controller/src/routes.rs:373`; primary image demotion/order in `controller/src/catalog.rs:220`, `controller/src/oracle_catalog.rs:192`, and `controller/src/publisher.rs:629`; Oracle constraints in `controller/db/schema.sql:24` and `controller/db/schema.sql:58`. |
| T-05-09 | Information Disclosure | object key generation | mitigate | CLOSED | UUID-only object key builder in `controller/src/storage_keys.rs:3`; key tests deny extension, spaces, and supplied filename in `controller/src/storage_keys.rs:17`; upload stores original filename separately in `controller/src/routes.rs:422`; Oracle schema keeps `original_filename` private metadata in `controller/db/schema.sql:50`. |
| T-05-10 | Denial of Service | upload endpoint | mitigate | CLOSED | Router body cap in `controller/src/routes.rs:210`; upload content-type allow-list and 20 MiB limit run before media write in `controller/src/routes.rs:395`; image decoder validation in `controller/src/routes.rs:452`; derivative source cap in `controller/src/derivatives.rs:4`. |
| T-05-11 | Repudiation | publish seed actions | accept | CLOSED | Accepted risk logged below. Phase 5 records created/updated timestamps and publish job status in `controller/db/schema.sql:22`, `controller/db/schema.sql:66`; Phase 6 edit-history follow-up is tracked in `.planning/REQUIREMENTS.md:18`, `.planning/ROADMAP.md:117`, and `.planning/ROADMAP.md:124`. |
| T-05-12 | Information Disclosure | derivative files | mitigate | CLOSED | Derivatives are decoded/resized/re-encoded as WebP in `controller/src/derivatives.rs:44`; public paths are deterministic generated paths in `controller/src/publisher.rs:568`; private source identifiers are scanned after candidate build in `controller/src/publisher.rs:537`; tests assert private filename/image ID/object key absence in `controller/tests/publisher.rs:84`. |
| T-05-13 | Tampering | candidate release | mitigate | CLOSED | Candidate publish validates before promotion in `controller/src/publisher.rs:456`; validation checks required files, JSON parseability, derivative existence, manifest byte sizes, inventory, and privacy scan in `controller/src/publisher.rs:696`; atomic current pointer rename in `controller/src/publisher.rs:927`; tests reject missing derivatives/private terms in `controller/tests/publisher.rs:169`. |
| T-05-14 | Denial of Service | publish rebuild | mitigate | CLOSED | Phase 5 derivative variants are limited to thumbnail/detail in `controller/src/derivatives.rs:7` and `controller/src/publisher.rs:566`; full rebuild is an explicit operator endpoint in `controller/src/routes.rs:207`; static admin calls explicit incremental/full publish endpoints in `controller/static-admin/admin.js:6`. |
| T-05-15 | Repudiation | publish attempts | mitigate | CLOSED | `PublishStatus` records state, release ID, artifact count, byte size, timestamps, and error summary in `controller/src/publisher.rs:351`; success/failure status updates in `controller/src/publisher.rs:465`; redaction in `controller/src/publisher.rs:1237`; route/status tests in `controller/tests/publisher.rs:484`. |
| T-05-16 | Information Disclosure | static admin source | mitigate | CLOSED | Static admin source tests deny admin secret names, storage terms, Object Storage URLs, OCI keys, and browser storage in `controller/tests/static_admin.rs:6`; admin files contain same-origin endpoint constants only in `controller/static-admin/admin.js:1`. |
| T-05-17 | Cross-Site Request Forgery | admin API | mitigate | CLOSED | Static admin fetches use same-origin credentials in `controller/static-admin/admin.js:16`; session cookies use `HttpOnly` and `SameSite=Strict` in `controller/src/routes.rs:710`; controller-side Origin/Referer checks in `controller/src/routes.rs:677`; tests in `controller/tests/auth_and_health.rs:76`. |
| T-05-18 | Tampering | publish controls | mitigate | CLOSED | Admin shell publish/unpublish/rebuild calls are under `/admin/api/*` in `controller/static-admin/admin.js:5`; static admin test requires privileged calls to stay under `/admin/api/` in `controller/tests/static_admin.rs:22`; publish routes require auth guard in `controller/src/routes.rs:514`. |
| T-05-19 | Information Disclosure | Caddy static/media routes | mitigate | CLOSED | Caddy blocks `/api/operator/*` in `deploy/ansible/roles/autographs_deploy/files/Caddyfile:8`; public root serves only `/srv/autographs/static/current` in `deploy/ansible/roles/autographs_deploy/files/Caddyfile:25`; deploy requires a promoted static manifest before cutover in `deploy/ansible/roles/autographs_deploy/tasks/main.yml:221`; retired `/api/operator/catalog` and `/api/catalog` public checks returned 404 in `.planning/phases/05-static-runtime-migration-foundation/05-07-SUMMARY.md:64` and `.planning/phases/05-static-runtime-migration-foundation/05-07-SUMMARY.md:65`. |
| T-05-20 | Elevation of Privilege | `/admin/api/*` proxy | mitigate | CLOSED | Caddy proxies only `/admin/api/*` to the Rust controller in `deploy/ansible/roles/autographs_deploy/files/Caddyfile:15`; controller quadlet joins the private Podman network without host `PublishPort` in `deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2:11`; route test asserts no controller `PublishPort` in `controller/tests/caddy_static_routes.rs:45`. |
| T-05-21 | Tampering | deploy workflow | mitigate | CLOSED | CI runs controller tests/check/build only in `.github/workflows/ci.yml:75`; deploy builds/publishes the controller image in `.github/workflows/deploy.yml:92`; image bake targets `controller/Dockerfile` in `.github/docker-bake.hcl:8`; deploy runs Ansible with provider variables in `.github/workflows/deploy.yml:266`; `rg` found no `publish.*catalog` or `generate.*catalog` workflow step. |
| T-05-22 | Denial of Service | cutover | accept | CLOSED | Accepted risk logged below. Phase 5 closeout recorded live smoke proof, and deploy docs define continued live-smoke use plus roll-forward/full-rebuild recovery in `docs/deployment-runbook.md:171`; static runbook documents explicit full rebuild in `docs/static-runtime-runbook.md:75`; Phase 5 UAT marks public cutover and private-controller boundary checks passed in `.planning/phases/05-static-runtime-migration-foundation/05-UAT.md:22`. |

## Accepted Risks Log

| Threat ID | Risk | Rationale | Follow-Up / Evidence | Status |
|-----------|------|-----------|----------------------|--------|
| T-05-11 | Phase 5 lacks full edit-history repudiation controls for seed/admin changes. | This was an explicit Phase 5 scope decision: the static runtime foundation records minimal timestamps/status while avoiding a premature full admin workflow. | Edit history remains tracked as Phase 6 work in `.planning/REQUIREMENTS.md:18` and `.planning/ROADMAP.md:117`; Phase 5 schema still records item timestamps and publish job status in `controller/db/schema.sql:22` and `controller/db/schema.sql:66`. | Accepted through Phase 6 admin workflow. |
| T-05-22 | Static-runtime cutover may involve planned downtime instead of a complex rollback path. | The project intentionally uses roll-forward/full-rebuild recovery for a one-owner low-cost deployment, backed by candidate validation and live smoke proof. | Roll-forward recovery is documented in `docs/deployment-runbook.md:171`; live static proof and public retired-route checks are recorded in `.planning/phases/05-static-runtime-migration-foundation/05-07-SUMMARY.md:64` and `.planning/phases/05-static-runtime-migration-foundation/05-07-SUMMARY.md:66`. | Accepted for Phase 5 cutover model. |

## Unregistered Flags

None. `rg -n "^## Threat Flags|Threat Flags" .planning/phases/05-static-runtime-migration-foundation/*-SUMMARY.md` returned no matches.

## Security Audit Trail

| Date | Auditor | Action | Result |
|------|---------|--------|--------|
| 2026-06-20 | gsd-security-auditor | Loaded all required Phase 05 plans, summaries, UAT, implementation files, deployment templates, tests, and docs listed in the prompt. | Complete. |
| 2026-06-20 | gsd-security-auditor | Checked repo-local `.codex/skills/` and `.agents/skills/` directories. | No project-local skill indexes present. |
| 2026-06-20 | gsd-security-auditor | Extracted threat models from `05-01-PLAN.md` through `05-06-PLAN.md`. | 24 declared threat rows found; duplicate `T-05-08` preserved as two rows with different category/component values. |
| 2026-06-20 | gsd-security-auditor | Verified mitigations by direct source/test/doc grep and line evidence; no implementation files modified. | 22 mitigated rows closed, 2 accepted-risk rows documented, 0 open threats. |
| 2026-06-20 | gsd-security-auditor | Checked summary threat flags. | No `## Threat Flags` sections found; no unregistered flags recorded. |

## Sign-Off

Phase 05 security audit status: verified.

Threats closed: 24/24.

Threats open: 0.

ASVS level: 1.
