# Coding Conventions

**Analysis Date:** 2026-07-10

## Naming Patterns

**Planning Artifacts**
- Phase directories use zero-padded numeric prefixes plus kebab-case slugs.
- Phase plan and summary files use `{phase}-{plan}-PLAN.md` and
  `{phase}-{plan}-SUMMARY.md`.
- Codebase map docs use uppercase concern names in `.planning/codebase/`.

**Rust Controller**
- Rust modules under `controller/src/` use descriptive snake_case names.
- Integration tests live under `controller/tests/`.
- Static public/admin assets live under `controller/static-public/` and
  `controller/static-admin/`.
- Production persistence behavior is guarded through the
  `production-persistence` feature.

**Operations**
- Ansible roles use descriptive snake_case role names, for example
  `autographs_deploy`, `autographs_system_cleanup`, and `security_patching`.
- GitHub workflow names should describe the operator action: CI, deploy, image
  cleanup, weekly security scan, and apply security updates.

**Domain Language**
- Prefer established terms: autograph item, signer, category, tags, primary
  image, supporting images, publication status, static release, candidate
  release, generated derivative, private original, admin shell, publisher,
  controller, edit history, and security patching issue.
- For Phase 7 work, prefer the richer taxonomy terms: signer profile, signer
  credit, signer role, character, franchise, product line, set name, format,
  origin, language, loose tags, schema version 2 facets, taxonomy backfill, and
  legacy-field deprecation.

## Code Style

- Rust is the active implementation language for runtime behavior.
- Keep public static artifacts free of private storage identifiers and
  unpublished records.
- Keep persistence/media details in controller adapters and service modules, not
  scattered through route handlers or static assets.
- Use plain static HTML/CSS/JavaScript for the admin/public static surfaces
  unless a later phase intentionally changes that constraint.
- Phase 6 admin UI copy should be concise, operational, and private-safe:
  same-origin `/admin/api/*` calls, no browser storage for credentials, redacted
  diagnostics, field-level history, cautious cleanup warnings, explicit publish
  batching, and bounded retention status.
- Phase 7 admin taxonomy copy should keep Identity, Classification, and Details
  as the main editor sections. Signer suggestions, duplicate warnings, and
  merge repair should remain private admin workflow features, not public
  taxonomy screens.
- Keep Ansible playbooks thin and put reusable behavior in roles.

## Error Handling

- Public static output should fail closed during generation/validation rather
  than publish incomplete or privacy-leaking artifacts.
- Controller routes should avoid leaking internal OCI, Oracle, or filesystem
  details in public/admin responses.
- Admin diagnostics, publish status, edit history, and cleanup warnings should
  stay redacted: no bucket names, namespaces, direct Object Storage URLs, raw
  object keys, original private filenames, Oracle internals, or secrets.
- Security patching apply runs must refuse drifted package sets and remove stale
  approval labels on failure.

## Testing Habits

- Use Cargo checks for current runtime code:
  `cargo fmt`, `cargo test`, `cargo check --features production-persistence`,
  `cargo build --features production-persistence`, and `cargo clippy`.
- Keep static contract/privacy tests mandatory for public artifact changes.
- For Phase 7 public output changes, keep schema version 2 static contract
  tests and live static publish smoke taxonomy assertions aligned with public
  facets: signer, franchise, productLine, format, language, origin, role, and
  tag. Category is not a schema version 2 public facet.
- Use live smoke workflows/runbooks only when real Oracle/Object Storage
  credentials and tenancy state are required.
- Run Ansible syntax/lint checks for deployment, cleanup, and security patching
  changes.

## Documentation Habits

- Distinguish historical Next.js/Phase 4 evidence from current Rust/static
  implementation.
- Keep operator docs procedural and explicit about manual prerequisites,
  secret handling, approval labels, and live-smoke requirements.
- Update `.planning/codebase/*` after substantial codebase drift so future
  agents do not resurrect retired architecture.
- Keep local/CI verification distinct from operator-run live Oracle/Object
  Storage smoke evidence.
- Document migration/backfill steps as report review, PL/SQL review, optional
  SQL Developer application, deploy, full static publish, and verification.
  Keep legacy `signer`, `category`, and `autograph_item_tags` cleanup framed as
  a later deprecation path until live schema version 2 verification is complete.

## Current Guidance

- Phase 5 foundation is complete; do not rebuild finished 05-01 through 05-07
  work, and treat the Rust/static cutover and Next.js retirement as validated.
- Phase 6 plans 06-01 through 06-07 are complete: polished admin workflow,
  session-cookie collection-management auth, field-level history, media cleanup
  ergonomics, pending-change status, explicit publish controls, release
  retention, operator docs, and security review are current-state behavior.
- Phase 7 metadata taxonomy and public facets are implemented: reusable signer
  profiles, item signer credits, first-class taxonomy fields, schema version 2
  public facets, signer merge repair, taxonomy backfill artifacts, rollout docs,
  and security review.
- Phase 8 starts with optional franchise/product-line/set/non-default-language
  taxonomy media cues on top of the richer manual metadata model, then adds
  advisory AI-assisted ingest.
- Do not introduce public accounts, multi-admin roles, direct Object Storage
  URLs, or a split multi-service architecture for v1.

---

*Conventions refreshed: 2026-07-19 after Phase 8 taxonomy media prioritization*
