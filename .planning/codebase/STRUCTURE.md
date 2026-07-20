# Codebase Structure

**Analysis Date:** 2026-07-10

## Directory Layout

```text
autographs/
├── controller/                    # Rust private controller and static publisher
│   ├── db/                         # Oracle schema
│   ├── fixtures/                   # Static/publisher and taxonomy migration fixtures
│   ├── src/                        # Controller, catalog, media, publisher modules
│   ├── static-admin/               # Private admin workflow with Phase 7 taxonomy editor
│   ├── static-public/              # Generated/public schema v2 templates/assets/fixtures
│   └── tests/                      # Rust integration and contract tests
├── deploy/ansible/                 # OCI runtime VM configuration and maintenance roles
│   ├── playbooks/                  # deploy, cleanup, security scan/patch playbooks
│   └── roles/                      # deploy, cleanup, security_patching roles
├── docs/                           # Operator runbooks and public project docs
├── infra/terraform/                # OCI infrastructure as code
├── .github/workflows/              # CI, deploy, cleanup, security patch workflows
├── .planning/                      # GSD project state, roadmap, phases, codebase maps
├── .prompts/                       # Original implementation prompt artifacts
└── renovate.json                   # Conservative dependency automation policy
```

## Key File Locations

**Rust Controller and Publisher**
- `controller/src/main.rs`: controller entry point.
- `controller/src/routes.rs`: admin/API route wiring.
- `controller/src/auth.rs`: single-admin/private access foundation.
- `controller/src/catalog.rs`: catalog domain behavior, edit history,
  cleanup event, publication, pending-change model, signer profiles, item
  signer credits, taxonomy validation, signer suggestions, and merge repair.
- `controller/src/catalog_admin.rs`: admin workflow DTOs and private catalog
  service behavior.
- `controller/src/oracle_catalog.rs`, `controller/src/oracle_schema.rs`:
  production persistence adapter and schema handling, including signer profile
  tables, item signer credits, characters, franchises, and item taxonomy fields.
- `controller/src/media.rs`, `controller/src/oci_media.rs`: media abstraction
  and OCI Object Storage implementation.
- `controller/src/publisher.rs`, `controller/src/contracts.rs`,
  `controller/src/derivatives.rs`: static artifact generation, validation,
  derivative creation, schema version 2 public facets, and release behavior.
- `controller/src/taxonomy_migration.rs` and
  `controller/src/bin/taxonomy_backfill.rs`: Phase 7 mapping/report/PLSQL
  generation from legacy category/tag values.
- `controller/db/updates/07-01-taxonomy-schema.sql`,
  `controller/db/updates/07-02-taxonomy-backfill-review.sql`, and
  `controller/db/updates/07-03-taxonomy-backfill-apply.sql`: additive schema,
  review SQL, and operator-reviewable taxonomy backfill artifacts.

**Static Assets**
- `controller/static-public/`: public static release source/templates/assets,
  including checked-in schema version 2 collection/facet/detail fixtures.
- `controller/static-admin/`: private admin workflow for item search,
  create/edit, image primary/remove/replace, history, pending changes,
  diagnostics, publish controls, cleanup warnings, release retention status,
  Identity/Classification/Details taxonomy sections, signer rows, duplicate
  warnings, and `Merge signer` repair.
- `controller/fixtures/`: fixtures for static contract and publisher tests.
- `.planning/phases/07-metadata-taxonomy-and-public-facets/taxonomy-backfill-mapping.json`:
  temporary Phase 7 migration mapping artifact for repeatable review/backfill.

**Tests**
- `controller/tests/auth_and_health.rs`
- `controller/tests/admin_workflow.rs`
- `controller/tests/caddy_static_routes.rs`
- `controller/tests/live_persistence_smoke.rs`
- `controller/tests/live_static_publish_smoke.rs`
- `controller/tests/media_cleanup.rs`
- `controller/tests/publisher.rs`
- `controller/tests/seed_content.rs`
- `controller/tests/static_admin.rs`
- `controller/tests/static_contract.rs`

**Infrastructure and Runtime**
- `infra/terraform/`: OCI runtime, DNS, database, media bucket, and supporting resources.
- `infra/terraform/tenancy/`: tenancy-level bootstrap concerns.
- `deploy/ansible/roles/autographs_deploy/`: controller/Caddy runtime deployment.
- `deploy/ansible/roles/autographs_system_cleanup/`: runtime cleanup.
- `deploy/ansible/roles/security_patching/`: production security update scanner/apply workflow.

**Planning and Documentation**
- `.planning/PROJECT.md`, `.planning/ROADMAP.md`, `.planning/REQUIREMENTS.md`,
  `.planning/STATE.md`: high-level GSD truth.
- `.planning/phases/05-static-runtime-migration-foundation/`: Phase 5 plans and summaries.
- `.planning/codebase/*.md`: current codebase maps for future agents.
- `README.md`: public showcase and current architecture overview.
- `docs/`: operator-facing setup, deploy, DNS, Terraform, dependency-update,
  security, static runtime, controller, and production patching runbooks.

## Where to Add New Code

**Phase 6 Admin Workflow**
- Continue extending `controller/static-admin/`, `controller/src/routes.rs`,
  `controller/src/routes/admin_items.rs`, and `controller/src/catalog_admin.rs`
  for admin workflow refinements.
- Keep edit history, media cleanup, release retention, pending-change status,
  and publication controls in Rust/controller boundaries rather than
  resurrecting the retired Next.js app.
- Add persistence changes through `controller/db/schema.sql`,
  `controller/db/updates/`, and production adapter tests where needed.

**Phase 7 Metadata Taxonomy and Public Facets**
- Phase 7 is implemented across `controller/src/catalog.rs`,
  `controller/src/oracle_catalog.rs`, `controller/src/contracts.rs`,
  `controller/src/publisher.rs`, `controller/src/routes/admin_items.rs`,
  `controller/static-admin/`, `controller/static-public/`,
  `controller/tests/admin_workflow.rs`, `controller/tests/publisher.rs`,
  `controller/tests/static_contract.rs`, and
  `controller/tests/live_static_publish_smoke.rs`.
- Keep legacy `signer`, `category`, and `autograph_item_tags` cleanup as a
  later deprecation path until live rollout and full static publish verification
  are complete.

**Public Static Output**
- Update `controller/static-public/`, `controller/src/contracts.rs`, and
  `controller/src/publisher.rs`.
- Preserve public-safe output: no private object keys, bucket names,
  namespaces, signed URLs, Oracle internals, image UUIDs, or unpublished data.
- Public static artifacts now use schema version 2 and facets for signer,
  franchise, productLine, format, language, origin, role, and tag. Do not
  reintroduce category as a public facet.

**Infrastructure and Operations**
- Use `infra/terraform/` for OCI resources and `deploy/ansible/` for VM/runtime
  behavior.
- Use `deploy/ansible/roles/security_patching/` for production OS security
  patching behavior and `docs/security-patching.md` for operator guidance.

## Current Layout Guidance

- Do not re-scaffold `app/`, pnpm workspace commands, or the retired Next.js runtime.
- Treat `.prompts/001-autograph-gallery-bootstrap-do/` as historical product intent.
- Treat Phase 5 static runtime/controller foundation as complete, including
  05-07 live static publish proof and closure evidence.
- Treat Phase 6 plans 06-01 through 06-07 as implemented: polished admin
  workflow, edit history, media cleanup, pending changes, release retention,
  session-cookie collection-management auth, docs, and security review are
  current-state surfaces.
- Treat Phase 7 metadata taxonomy/public facets as implemented current-state
  surfaces.
- Treat Phase 8 as pending taxonomy media cue exploration first, then advisory
  AI-assisted ingest.

---

*Structure analysis refreshed: 2026-07-19 after Phase 8 taxonomy media prioritization*
