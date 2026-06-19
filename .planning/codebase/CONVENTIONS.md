# Coding Conventions

**Analysis Date:** 2026-06-19

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

## Code Style

- Rust is the active implementation language for runtime behavior.
- Keep public static artifacts free of private storage identifiers and
  unpublished records.
- Keep persistence/media details in controller adapters and service modules, not
  scattered through route handlers or static assets.
- Use plain static HTML/CSS/JavaScript for the minimal admin/public static
  surfaces unless a later phase intentionally changes that constraint.
- Keep Ansible playbooks thin and put reusable behavior in roles.

## Error Handling

- Public static output should fail closed during generation/validation rather
  than publish incomplete or privacy-leaking artifacts.
- Controller routes should avoid leaking internal OCI, Oracle, or filesystem
  details in public/admin responses.
- Security patching apply runs must refuse drifted package sets and remove stale
  approval labels on failure.

## Testing Habits

- Use Cargo checks for current runtime code:
  `cargo fmt`, `cargo test`, `cargo check --features production-persistence`,
  `cargo build --features production-persistence`, and `cargo clippy`.
- Keep static contract/privacy tests mandatory for public artifact changes.
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

## Current Guidance

- Phase 5 foundation is mostly implemented; do not rebuild finished 05-01
  through 05-06 work, but keep 05-07 live proof/cutover documentation pending.
- Phase 6 owns polished admin workflow, edit history, and media cleanup
  ergonomics after the 05-07 checkpoint passes.
- Phase 7 owns advisory AI-assisted ingest.
- Do not introduce public accounts, multi-admin roles, direct Object Storage
  URLs, or a split multi-service architecture for v1.

---

*Conventions refreshed: 2026-06-19 after Phase 5 static runtime implementation and PR 129 production security patching merge*
