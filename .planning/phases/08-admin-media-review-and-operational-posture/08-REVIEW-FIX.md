---
phase: 08-admin-media-review-and-operational-posture
fixed_at: 2026-10-05T20:27:44Z
review_path: .planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md
iteration: 6
findings_in_scope: 1
fixed: 1
skipped: 0
status: all_fixed
---

# Phase 08: Code Review Fix Report

**Fixed at:** 2026-10-05T20:27:44Z
**Source review:** `.planning/phases/08-admin-media-review-and-operational-posture/08-REVIEW.md`
**Iteration:** 6 — approved Media Revision Authority Addendum

**Summary:**

- Findings in scope: 1
- Fixed: 1
- Skipped: 0
- Implementation commit: `68431b4`
- PR fix comment: https://github.com/jetsaredim/autographs/pull/263#issuecomment-6002367136

## Fixed Issues

### CR-01: Editor image mutations can preserve an old-source drag and save it onto replacement bytes

**Status:** Fixed — requires human verification
**Files modified:** `controller/src/catalog.rs`, `controller/src/oracle_catalog.rs`, `controller/src/routes.rs`, `controller/static-admin/admin.js`, `controller/tests/admin_workflow.rs`, `controller/tests/live_persistence_smoke.rs`, `controller/tests/media_cleanup.rs`, `controller/tests/static_admin.rs`, `controller/tests/static_admin_behavior.mjs`, `controller/tests/static_contract.rs`
**Commit:** `68431b4`
**Applied fix:** Implemented the independently approved Media Revision Authority Addendum as one server, repository, client, and test contract. Authenticated admin image responses expose only a domain-separated SHA-256 `mediaRevision`; review source/draft/assist/Save/Reset work validates that token before work and uses a common terminal authority check after private media work on both success and error paths. In-memory and Oracle adjustment updates atomically compare the server-private expected object key, typed conflicts produce the stable redacted 409 response, and replacement rollback restores the complete original image snapshot including adjustment. The admin client reconciles every returned item before assigning it, treats a missing reviewed image and revision mismatch identically, binds revision into session/request/output/gesture/mutation authority, and tears down stale review work without replaying old geometry.

## Invariant and Consumer Audit

| Invariant | Producers and mutation boundaries | Consumers and terminal paths | Evidence |
|---|---|---|---|
| One opaque revision identifies the private media occupying an image UUID slot. | Admin response mapper and review response derive SHA-256 from domain tag, image UUID, object key, checksum/ETag, content type, and byte size. Replacement changes the random object key; rollback restores the original snapshot. | Admin item cards, review session, source/draft URLs, assist, Save, Reset, adjusted-output callbacks, perspective gestures, and item reconciliation. | Byte-identical/same-metadata replacement changes the token; response redaction and public artifact deny-list keep private identity and `mediaRevision` out of public output. |
| Revision conflict outranks an obsolete media operation result. | Preview/source/draft and assist prevalidate, perform media work, then use one terminal reloader on success and failure. | Successful derivatives/proposals and read/render/assist failures return only if the revision remains current; otherwise stable 409 wins. | Blocking read test replaces metadata and deletes the old object while the read is in flight; terminal response is redacted 409 rather than provider 500. |
| Adjustment writes cannot race replacement after route validation. | Save/Reset retain the validated private object key; memory compares under the item lock and Oracle includes `object_key = :4` in the update predicate. Canonical no-op routes revalidate before returning. | Save, Reset, stale other-session mutations, missing resource, validation, and repository failures. | Typed `ImageAdjustmentUpdateError` distinguishes conflict, not-found, validation, and repository failure; route tests assert stale Save 409 and repository tests distinguish conflict from 404-class absence. |
| Replacement and rollback persist coherent snapshots. | `ImageReplacementInput.image.adjustment` is now authoritative. Normal replacement supplies `None`; rollback supplies the complete original `AutographImage`. Oracle binds serialized adjustment instead of forcing null. | Cleanup success, cleanup warning, warning-persistence failure, returned admin item, and later review token/baseline. | Cleanup-warning persistence failure starts with a non-identity adjustment and proves object key plus adjustment are restored together. |
| Every item-returning client path reconciles before assignment. | One `reconcileAdminItemResponse(item)` owns the only `state.currentItem` assignment. Save item, upload, primary, remove, replace, cleanup retry, Save/Reset, item load, editor render, and refresh paths route through it. | Active gesture, source projection, pending preview/assist/mutation, review state, editor assignment, and render. | DOM harness covers exact revision continuation, changed revision invalidation, missing reviewed-image invalidation, stable conflict teardown, and revision-bearing Save/Reset/draft/assist requests. |

## Direct Media and Serializer Audit

- The only direct private-media reads in `routes.rs` for admin adjustment work are the shared preview/source/draft path and assist path; both perform prevalidation and common terminal revalidation on every operation result.
- Review response performs a final revision check after public-comparison and pending-change lookups before emitting URLs or authority.
- Save and Reset route through the typed atomic repository boundary; their no-op branches perform a fresh terminal revision check.
- The only `state.currentItem =` assignment in `admin.js` is inside `reconcileAdminItemResponse`.
- `mediaRevision` is added only by authenticated admin response types. Public contract/publisher serializers remain unchanged, and generated static artifact tests explicitly deny `mediaRevision`.
- Oracle replacement SQL persists supplied adjustment JSON, and Oracle adjustment SQL compares the expected private object key.

## Verification

All gates ran in `/tmp/autographs-pr263-review` on `gsd/phase-08-admin-media-review-and-operational-posture` at implementation commit `68431b4` (the report itself remains uncommitted for the orchestrator).

- `node --check controller/static-admin/admin.js` — passed.
- `node controller/tests/static_admin_behavior.mjs` — passed.
- `cargo fmt --manifest-path controller/Cargo.toml --all -- --check` — passed.
- `cargo test --manifest-path controller/Cargo.toml --test admin_workflow --test media_cleanup` — passed; 52 tests.
- `cargo test --manifest-path controller/Cargo.toml --test static_admin` — passed; 18 tests.
- `cargo test --manifest-path controller/Cargo.toml --all-targets` — passed; 165 non-ignored tests, with 2 credential-gated live tests ignored.
- `cargo test --manifest-path controller/Cargo.toml --all-targets --all-features` — passed; 223 non-ignored tests, with 2 credential-gated live tests ignored.
- `cargo check --manifest-path controller/Cargo.toml --all-targets --features production-persistence` — passed.
- `cargo clippy --manifest-path controller/Cargo.toml --all-targets --all-features -- -D warnings` — passed.
- `git diff --check` — passed.

---

_Fixed: 2026-10-05T20:27:44Z_
_Fixer: the agent (gsd-code-fixer)_
_Iteration: 6_
