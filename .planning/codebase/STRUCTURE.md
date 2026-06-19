# Codebase Structure

**Analysis Date:** 2026-06-19

## Directory Layout

```text
autographs/
├── controller/                    # Rust private controller and static publisher
│   ├── db/                         # Oracle schema
│   ├── fixtures/                   # Static/publisher fixtures
│   ├── src/                        # Controller, catalog, media, publisher modules
│   ├── static-admin/               # Minimal private admin shell
│   ├── static-public/              # Generated/public static artifact templates/assets
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
- `controller/src/catalog.rs`: catalog domain behavior.
- `controller/src/oracle_catalog.rs`, `controller/src/oracle_schema.rs`:
  production persistence adapter and schema handling.
- `controller/src/media.rs`, `controller/src/oci_media.rs`: media abstraction
  and OCI Object Storage implementation.
- `controller/src/publisher.rs`, `controller/src/contracts.rs`,
  `controller/src/derivatives.rs`: static artifact generation, validation,
  derivative creation, and release behavior.

**Static Assets**
- `controller/static-public/`: public static release source/templates/assets.
- `controller/static-admin/`: minimal admin shell.
- `controller/fixtures/`: fixtures for static contract and publisher tests.

**Tests**
- `controller/tests/auth_and_health.rs`
- `controller/tests/caddy_static_routes.rs`
- `controller/tests/live_persistence_smoke.rs`
- `controller/tests/live_static_publish_smoke.rs`
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
- Extend `controller/static-admin/` and `controller/src/routes.rs` for polished
  admin UX/API behavior.
- Keep edit history, media cleanup, and publication controls in Rust/controller
  boundaries rather than resurrecting the retired Next.js app.
- Add persistence changes through `controller/db/schema.sql` and production
  adapter tests where needed.

**Public Static Output**
- Update `controller/static-public/`, `controller/src/contracts.rs`, and
  `controller/src/publisher.rs`.
- Preserve public-safe output: no private object keys, bucket names,
  namespaces, signed URLs, Oracle internals, image UUIDs, or unpublished data.

**Infrastructure and Operations**
- Use `infra/terraform/` for OCI resources and `deploy/ansible/` for VM/runtime
  behavior.
- Use `deploy/ansible/roles/security_patching/` for production OS security
  patching behavior and `docs/security-patching.md` for operator guidance.

## Current Layout Guidance

- Do not re-scaffold `app/`, pnpm workspace commands, or the retired Next.js runtime.
- Treat `.prompts/001-autograph-gallery-bootstrap-do/` as historical product intent.
- Treat Phase 5 static runtime/controller foundation as implemented.
- Treat Phase 6 as polished admin workflow and Phase 7 as advisory AI ingest.

---

*Structure analysis refreshed: 2026-06-19 after Phase 5 static runtime implementation and PR 129 production security patching merge*
