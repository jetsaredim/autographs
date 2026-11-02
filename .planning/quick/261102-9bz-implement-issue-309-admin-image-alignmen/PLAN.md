---
quick: 261102-9bz
status: in_progress
mode: quick
issue: 309
branch: feat/issue-309-admin-image-workbench
must_haves:
  truths:
    - "An operator sees the next reliable alignment suggestion and can explicitly run bounded auto-alignment without persisting it."
    - "Only improving passes are retained; unreliable or non-improving results preserve the last good draft."
    - "An operator can assess real photographed border/background margins, preview/apply/dismiss/undo framing, and use a confirmed manual boundary after abstention."
    - "Grid and corner controls are adjustable and every analysis result is bound to current image, media revision, and draft authority."
    - "Existing private-original, authenticated no-store, revision-conflict, and explicit Save contracts remain intact."
  artifacts:
    - path: controller/image-processing/src/image_adjustments.rs
      provides: Shared analysis, alignment decision, and real-pixel framing logic
    - path: controller/src/routes.rs
      provides: Authenticated draft analysis and auto-align routes
    - path: controller/static-admin/admin.js
      provides: Reviewable suggestions and draft-only UI behavior
    - path: controller/static-admin/index.html
      provides: Alignment and framing controls
  key_links:
    - from: controller/static-admin/admin.js
      to: controller/src/routes.rs
      via: Revision-bound analysis and auto-align JSON requests
    - from: controller/src/routes.rs
      to: controller/image-processing/src/image_adjustments.rs
      via: Shared detector and adjustment logic
---

# Restore workbench alignment and framing in admin review

## Contract

- Source originals remain private and immutable. Analysis and draft previews require the admin session and media revision, return no-store, and redact provider errors.
- Automatic alignment operates on a caller-supplied draft. It analyzes rendered output, proposes perspective or rotation, accepts only improving passes, backs off failed steps, detects cycles, and stops after at most eight passes. It returns a proposed final draft and stop reason; it never persists.
- Framing measures the item boundary and photographed pixel support. Tentative boundaries are diagnostic only. An operator may explicitly confirm a valid manual boundary for framing assessment. Crop or recentering suggestions are previewed and applied to the local draft only; undo restores the previous draft.
- Grid spacing and offsets, source corners, output diagnostics, and background target are operator controls. Saving remains a separate action with existing revision checks.

## API contract

- `POST .../adjustment/analysis` accepts `{mediaRevision, adjustment, manualBoundary?}` and returns `{boundary:{status,corners,edges,message?}, nextSuggestion, framingSupport, framingBoundarySource}`. `nextSuggestion` is rotation or perspective from a reliable automatic rendered-output boundary. `framingSupport` reports real-pixel margins and constraints from a reliable automatic or explicitly confirmed valid manual boundary. Tentative boundaries are diagnostic only.
- `POST .../adjustment/auto-align` accepts `{mediaRevision, adjustment}` and returns `{adjustment, acceptedPasses, stopReason, boundary}`. It never persists. Both routes authorize before parsing, validate inputs, check media revision before and after private media work, redact provider errors, and send `Cache-Control: no-store` on success and error.
- The browser captures image session, media revision, draft revision, and request generation for each request. A stale result cannot change draft, suggestion, status, or preview. Edits invalidate prior proposals and framing confirmations.

<tasks>

<task type="auto">
  <name>Task 1: Port alignment and framing analysis into shared Rust and private routes</name>
  <files>controller/image-processing/src/image_adjustments.rs; controller/src/routes.rs; controller/tests/image_adjustments.rs; controller/tests/admin_workflow.rs</files>
  <action>Port the accepted workbench decision, scoring, backoff, cycle, and eight-pass stop logic. Add real-pixel framing support and manual-boundary validation. Wire the two API routes to caller-supplied drafts with current session and revision checks.</action>
  <verify>Run `cargo test --manifest-path controller/Cargo.toml --test image_adjustments` and `cargo test --manifest-path controller/Cargo.toml --test admin_workflow`; fail on missing stop-condition or route-contract assertions.</verify>
  <done>Analysis and auto-align return reviewable draft proposals without changing persistence.</done>
</task>

<task type="auto">
  <name>Task 2: Restore operator controls in the admin review UI</name>
  <files>controller/static-admin/index.html; controller/static-admin/admin.css; controller/static-admin/admin.js; controller/tests/static_admin.rs</files>
  <action>Show next suggestion and explicit Auto-align to tolerance action, output measurements, adjustable grid and numeric source corners. Add real-background margin assessment with preview/apply/dismiss/undo and confirmed manual-boundary fallback. Integrate with current draft, session, and revision authority.</action>
  <verify>Run `cargo test --manifest-path controller/Cargo.toml --test static_admin`; locally exercise every interaction row below in the actual admin UI and record pass/fail observations.</verify>
  <done>Every control is available in production admin and drafts remain unsaved until explicit Save.</done>
</task>

<task type="auto">
  <name>Task 3: Verify, publish PR, and review to convergence</name>
  <files>All changed implementation and test files; this quick plan and SUMMARY.md</files>
  <action>Run existing Cargo checks and focused local UI verification. Compare accepted private workbench examples before declaring issue complete. Commit initial implementation, push outside sandbox with approval, open a ready PR, and preserve review/coder findings and clean confirmations in PR comments.</action>
  <verify>Run `cargo fmt --manifest-path controller/Cargo.toml --check`, `cargo test --manifest-path controller/image-processing/Cargo.toml`, `cargo test --manifest-path controller/Cargo.toml`, `cargo check --manifest-path controller/Cargo.toml --features production-persistence`, and `cargo clippy --manifest-path controller/Cargo.toml --all-targets --features production-persistence -- -D warnings`; check PR CI and private-example acceptance separately.</verify>
  <done>Ready PR carries verified implementation and review trail.</done>
</task>

</tasks>

## Falsifiable interaction checks

| Scenario | Expected result | Where / when |
| --- | --- | --- |
| Confident suggestion displayed | No draft mutation before explicit Auto-align | Local admin UI before PR; static contract guard in existing Cargo CI |
| Auto-align finds improvement then no improvement | Only improving passes remain; stop reason shown; Save still separate | Rust fixtures in existing Cargo CI; local UI before PR |
| User edits while analysis is pending | Older response cannot replace current draft, suggestion, or status | Local UI before PR and after integration |
| Framing preview, dismiss, apply, undo | Preview is non-mutating; dismiss leaves draft; apply changes draft; undo restores exact prior draft | Local UI before PR; Rust framing geometry checks in existing Cargo CI |
| Tentative or unavailable boundary | No automatic control suggestion; manual corner/confirmed-boundary path remains usable | Rust fixtures in existing Cargo CI; accepted private corpus before PR and after deploy |
| Media changes mid-request | Conflict response; stale UI result ignored; review recovery path used | Existing admin route test suite and local UI before PR |

## Verification boundaries

- No Node runtime, new CI job, private image corpus in Git, or blanket coverage expansion.
- Existing Cargo suites cover Rust decisions and API behavior; operator review of the accepted private examples is required outside CI before closeout. If the external corpus is inaccessible, leave acceptance pending and state the blocker explicitly.
- Issue #310 owns the separate repository-wide coverage audit.
