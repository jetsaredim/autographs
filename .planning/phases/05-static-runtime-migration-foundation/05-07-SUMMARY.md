---
phase: 05-static-runtime-migration-foundation
plan: 07
subsystem: live-runtime-validation
tags: [rust, oracle, oci-object-storage, static-publish, caddy, podman, live-smoke]
requires:
  - phase: 05-06
    provides: deployed Rust/static runtime wiring
provides:
  - Credential-gated live static publish proof through the deployed Rust/static runtime
  - Phase 5 closure evidence for the static runtime migration foundation
  - OCI cleanup and operator diagnostics for live smoke residue
affects: [06-admin-workflow, 06-media-cleanup]
tech-stack:
  added: [python3-oci-cli]
  patterns: [commit-tagged-smoke-images, controller-operation-logging, bounded-oci-cleanup]
key-files:
  modified:
    - controller/src/main.rs
    - controller/src/oci_media.rs
    - controller/src/oracle_catalog.rs
    - controller/src/routes.rs
    - controller/tests/live_persistence_smoke.rs
    - controller/tests/live_static_publish_smoke.rs
    - controller/Dockerfile.smoke
    - controller/Dockerfile.static-smoke
    - deploy/ansible/roles/autographs_deploy/defaults/main.yml
    - deploy/ansible/roles/autographs_deploy/tasks/main.yml
    - deploy/ansible/roles/autographs_deploy/templates/app.env.j2
    - deploy/ansible/roles/autographs_deploy/templates/autographs-controller.container.j2
    - docs/configuration-contract.md
    - docs/deployment-runbook.md
    - docs/static-runtime-runbook.md
key-decisions:
  - "Tag live smoke and controller images with the source commit and print the embedded revision so live evidence maps back to repository content."
  - "Keep the live static smoke responsible for its seeded test residue, while Phase 6 owns normal controller/admin media deletion ergonomics."
  - "Install OCI CLI on the runtime VM as an operator diagnostic tool for instance-principal Object Storage checks."
requirements-completed: [STATIC-07]
duration: live validation and closure session
completed: 2026-06-20
---

# Phase 05 Plan 07: Live Static Publish Proof Summary

**The deployed Rust/static runtime now has recorded live proof from seed content through public static output and cleanup.**

## Accomplishments

- Added and exercised credential-gated live smoke coverage that creates a minimal catalog item through the private controller, uploads a private original to OCI Object Storage, publishes generated static output, verifies the generated item page through Caddy, drafts and republishes the item away, and cleans up Oracle/Object Storage residue.
- Added live diagnostics so smoke output includes the image revision, seeded item id, private object key, publish release id, and generated slug. This makes repeated same-tag image testing traceable to the exact repository commit.
- Fixed live-only issues found during proof: Oracle catalog update bind ordering, no-op primary-image demotion causing `ORA-12838`, missing wallet access under SELinux labeling, controller mutation error visibility, and OCI cleanup running after the async runtime had already shut down.
- Added bounded OCI request timeouts, retry-oriented cleanup diagnostics, live persistence list/cleanup helpers, and the `python3-oci-cli` runtime diagnostic package path for direct instance-principal Object Storage verification.

## Verification

- `cargo fmt --manifest-path controller/Cargo.toml --check`
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence live_static_publish_smoke -- --ignored --nocapture` locally skipped without live opt-in, proving the gate remains safe by default.
- `cargo test --manifest-path controller/Cargo.toml --features live-persistence live_persistence_smoke -- --ignored --nocapture` locally skipped without live opt-in, proving the cleanup/list helper gate remains safe by default.
- `cargo test --manifest-path controller/Cargo.toml --test seed_content -- --nocapture`
- `cargo test --manifest-path controller/Cargo.toml --test caddy_static_routes -- --nocapture`
- `ANSIBLE_LOCAL_TEMP=/tmp/ansible-local ANSIBLE_REMOTE_TEMP=/tmp/ansible-remote ANSIBLE_CONFIG=deploy/ansible/ansible.cfg ansible-playbook --syntax-check deploy/ansible/playbooks/deploy.yml`
- Public edge sanity check from the workstation after deployment:
  - `https://autographs.jetsaredim.net/collection/` returned `200`.
  - `https://autographs.jetsaredim.net/api/operator/catalog` returned `404`.
  - `https://autographs.jetsaredim.net/api/catalog` returned `404`.
- Live static smoke on the runtime VM passed with image revision `23b6289`, item id `ab448e01-f359-4f21-8731-f4ae2090456f`, object key `originals/ab448e01-f359-4f21-8731-f4ae2090456f/13de6a7a-aa1f-4a4f-8d6a-3c1b2b4a8d2b`, release id `2cc81313-0638-4de2-8143-1a613391519d`, and generated slug `live-static-smoke-936509b3268249ab97bfe11bc0c7fa64`.
- Live persistence list mode reported no remaining live smoke rows after cleanup.
- Operator console verification found no smoke objects remaining outside the expected `/autographs` path after the final run.

## Deviations

- The closure pass became a focused live-debugging session because the first end-to-end attempts exposed deployment and production-persistence details that local tests could not see. Those fixes are now captured in the controller, Ansible, smoke images, and operator docs instead of being left as manual tribal knowledge.
- The smoke test now deletes its own uploaded object directly through the same instance-principal media path used by the controller. Normal admin-owned object lifecycle and deletion ergonomics remain Phase 6 scope.

## Next Phase Readiness

- Phase 5 is complete. The current public runtime is Caddy-served static output generated by the Rust private controller path; the old public Next.js/catalog API runtime is no longer the active implementation target.
- Phase 6 can start formal planning for the polished single-admin collection workflow, including create/edit UX, multi-image maintenance, edit history, controller-owned deletion behavior, media cleanup ergonomics, and admin hardening.
- Keep the live smoke images and OCI CLI diagnostics available for Phase 6 regression checks whenever admin workflows begin mutating production Oracle/Object Storage content.

---
*Phase: 05-static-runtime-migration-foundation*
*Completed: 2026-06-20*
