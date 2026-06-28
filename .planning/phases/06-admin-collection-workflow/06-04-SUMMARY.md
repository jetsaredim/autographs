---
phase: 06-admin-collection-workflow
plan: 04
subsystem: api
tags: [rust, axum, static-publisher, admin-status, ansible]

requires:
  - phase: 06-03
    provides: retryable media cleanup warnings and pending-change foundations
provides:
  - bounded static release retention for promoted releases and failed candidates
  - redacted `/admin/api/status` diagnostics for the admin hub
  - explicit incremental publish batching that clears pending changes after success
  - runtime and Ansible configuration for release retention counts
affects: [phase-06-admin-ui, publish-status, release-retention, operator-diagnostics]

tech-stack:
  added: []
  patterns:
    - publisher-owned count-based release pruning under the configured static root
    - dedicated redacted admin diagnostics DTOs instead of exposing internal provider/storage state
    - repository-owned successful publish boundary for pending-change calculations

key-files:
  created:
    - .planning/phases/06-admin-collection-workflow/06-04-SUMMARY.md
  modified:
    - .env.example
    - controller/src/catalog.rs
    - controller/src/config.rs
    - controller/src/oracle_catalog.rs
    - controller/src/publisher.rs
    - controller/src/routes.rs
    - controller/tests/admin_workflow.rs
    - controller/tests/publisher.rs
    - deploy/ansible/roles/autographs_deploy/defaults/main.yml
    - deploy/ansible/roles/autographs_deploy/templates/controller.env.j2

key-decisions:
  - "Release retention is count-based: promoted releases default to 5 including the active current target, and failed candidates default to 1."
  - "Admin status uses dedicated redacted DTOs for diagnostics rather than reusing internal Oracle, OCI, media, or filesystem structures."
  - "A successful explicit publish records the pending-change boundary; save, image, and publication mutation routes do not publish automatically."

patterns-established:
  - "LocalPublisher::with_retention_policy carries bounded release retention and exposes ReleaseRetentionStatus for diagnostics."
  - "CatalogRepository::record_successful_publish records the publication boundary used by pending_changes()."

requirements-completed: [ADMIN-04, ADMIN-05, MEDIA-04]

duration: 10min
completed: 2026-06-27
---

# Phase 06-04: Pending Status, Diagnostics, Publish Batching, and Retention Summary

**Redacted admin status with bounded static-release retention and explicit incremental publish boundaries**

## Performance

- **Duration:** 10 min
- **Started:** 2026-06-27T19:31:30Z
- **Completed:** 2026-06-27T19:41:48Z
- **Tasks:** 3
- **Files modified:** 10

## Accomplishments

- Added `ReleaseRetentionPolicy` and `ReleaseRetentionStatus`, with configurable promoted and failed-release retain counts.
- Pruned promoted releases after successful publish while preserving the active `current` target.
- Replaced single failed-candidate retention with count-based failed candidate retention.
- Added `GET /admin/api/status` with redacted provider modes, controller booleans, publish summary, pending changes, cleanup warning count, retention status, and safe guidance copy.
- Recorded successful explicit publish jobs so pending changes clear after incremental publish succeeds.
- Added `.env.example` and Ansible defaults/template entries for runtime retention settings.

## Task Commits

1. **Task 1: Add bounded release retention to the publisher**
   - `141e019` test(06-04): add failing release retention coverage
   - `e5ee404` feat(06-04): add bounded release retention
2. **Task 2: Add redacted admin status and diagnostics API**
   - `a905aff` test(06-04): add failing admin status coverage
   - `f766f0f` feat(06-04): add redacted admin status endpoint
3. **Task 3: Keep publish explicit and incremental by default**
   - `edec212` test(06-04): add failing publish batching coverage
   - `ffab23b` feat(06-04): record publish boundary for pending changes

## Files Created/Modified

- `.env.example` - Adds promoted and failed release retention defaults.
- `controller/src/catalog.rs` - Adds `record_successful_publish` and memory pending-change boundary tracking.
- `controller/src/config.rs` - Adds retention count config parsing with zero/invalid fallback to defaults.
- `controller/src/oracle_catalog.rs` - Persists succeeded publish jobs and computes pending changes after the latest successful publish.
- `controller/src/publisher.rs` - Adds retention policy/status, promoted release pruning, and count-based failed candidate retention.
- `controller/src/routes.rs` - Registers `/admin/api/status`, returns redacted diagnostics, and records publish boundaries after explicit publish success.
- `controller/tests/admin_workflow.rs` - Adds status and publish-batching coverage.
- `controller/tests/publisher.rs` - Adds release retention regression coverage.
- `deploy/ansible/roles/autographs_deploy/defaults/main.yml` - Adds retention defaults for deploy rendering.
- `deploy/ansible/roles/autographs_deploy/templates/controller.env.j2` - Renders retention env vars for the controller.

## Decisions Made

- Count-based retention was implemented directly in `LocalPublisher` because release directories are publisher-owned runtime filesystem state.
- The active `current` symlink target is always included in the retained promoted release set before pruning older releases.
- Pending-change clearing is tied to successful explicit publish responses, not save or image mutation routes.
- `/admin/api/status` intentionally reports provider labels and counts, not object keys, bucket names, namespaces, original filenames, Oracle connection values, or secrets.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- The plan's Ansible syntax command needed the repository Ansible config to resolve the local role path. The command passed with `ANSIBLE_CONFIG=deploy/ansible/ansible.cfg` plus the planned `/tmp` temp vars.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check` - passed
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow admin_status_reports_pending_publish_cleanup_and_retention -- --nocapture` - passed
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow publish_batches_saved_changes -- --nocapture` - passed
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow -- --nocapture` - passed, 9 tests
- `cargo test --manifest-path controller/Cargo.toml --test publisher retention -- --nocapture` - passed, 2 tests
- `cargo test --manifest-path controller/Cargo.toml --test publisher -- --nocapture` - passed, 10 tests
- `cargo test --manifest-path controller/Cargo.toml` - passed, full controller suite
- `cargo check --manifest-path controller/Cargo.toml --features production-persistence` - passed
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets -- -D warnings` - passed
- `ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ansible-playbook -i deploy/ansible/inventory/ci.ini deploy/ansible/playbooks/deploy.yml --syntax-check` - passed

## Known Stubs

None.

## Threat Flags

None - new admin diagnostics and publish filesystem behavior were already covered by the plan threat model and mitigations.

## User Setup Required

None - no external service configuration is required. Operators can override `AUTOGRAPHS_STATIC_PROMOTED_RELEASE_RETAIN_COUNT` and `AUTOGRAPHS_STATIC_FAILED_CANDIDATE_RETAIN_COUNT` if the defaults are not suitable.

## Next Phase Readiness

Plan 06-05 can render the polished static admin hub against `/admin/api/status`, use `Publish changes` for the normal incremental publish action, and surface retention/cleanup diagnostics without exposing private storage details.

## Self-Check: PASSED

- Key files exist, including this summary and all modified source/config/test files.
- Task commits exist: `141e019`, `e5ee404`, `a905aff`, `f766f0f`, `edec212`, and `ffab23b`.
- `controller/src/routes.rs` contains `.route("/admin/api/status", get(admin_status))`.
- `controller/src/publisher.rs` contains `pub struct ReleaseRetentionPolicy`.
- Touched-file stub scan found no `TODO`, `FIXME`, placeholder, or empty hardcoded UI-data stubs.
- Privacy-term scan only found denied strings in existing internal env reads and explicit regression assertions.

---
*Phase: 06-admin-collection-workflow*
*Completed: 2026-06-27*
