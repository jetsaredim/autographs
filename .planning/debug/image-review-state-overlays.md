---
status: awaiting_human_verify
trigger: "Image review UI leaks state across items and renders overlays inconsistently"
created: 2026-10-12
updated: 2026-10-12T17:26:55-04:00
---

# Debug Session: Image Review State and Overlays

## Symptoms

- Expected behavior: each reviewed image initializes an isolated draft from that image's saved adjustments or clean defaults; overlay checkboxes match rendered overlays; overlay geometry is computed for the current image; edge detection visibly proposes valid corners, reports that no meaningful correction is needed, or clearly abstains.
- Actual behavior: overlay control state appears to carry from one image review to another when the next image has no saved adjustments. Grid can remain checked while only the dashed Edges rectangle is visible, and Edges can render while its checkbox is unchecked. The dashed geometry appears in the same preview-relative position across different images. Grid renders as a single central box rather than intersecting grid lines. Edge detection failed on a high-contrast card image and reported success on another image without a perceptible change or synchronized Edges checkbox.
- Error messages: no browser exception was reported. UI messages observed were "Auto correction could not find reliable edges. Adjust the corners manually." and "Detected edges applied. Review the corners before saving."
- Timeline: discovered during initial Phase 8 testing of the newly implemented private admin image-review UI; no evidence that these behaviors previously worked correctly.
- Reproduction: open one image review, change overlay state, navigate back through the admin UI, then open a different image with no saved adjustments. Observe inherited checkbox/render state and geometry. Click Detect edges and compare corner handles, adjusted output, overlay visibility, and status message.
- Evidence: local screenshots, one supplied diagnostic image, and 116 local
  production-like originals held outside the repository. These private artifacts
  were used only for local diagnosis and were not copied into Git.

## Current Focus

- bug_class: bohrbug
- hypothesis: confirmed — single-threshold confidence plus adaptive-threshold inflation allowed internal high-contrast content to masquerade as an outer card/slab boundary
- test: self-verification complete; production verification remains deferred until the existing PR is updated, deployed, and followed by an authenticated full static publish
- expecting: real admin Detect edges safely abstains on ambiguous light-border/slab images while retaining material confident corners for the supplied dark-card image
- next_action: hand the two-file uncommitted detector hardening to the orchestrator for commit/push; retain the already-agreed post-deployment human verification checkpoint

reasoning_checkpoint:
  hypothesis: "Border variability inflates the adaptive primary threshold, and using only fractions of that inflated value leaves every stability probe above a weak enclosing boundary, so the detector can still label an internal high-contrast component Confident."
  confirming_evidence:
    - "The remaining known corpus failure has adaptive threshold 234 and probes 175/117/78; all remain on the inner region even though the visible slab/card extends far beyond it."
    - "A sanitized variable-border fixture reproduces the mechanism and fails red as Confident with the current fractional probes."
    - "The known accurate dark-card diagnostic is stable from threshold 66 down through 22 with only 7.2% population growth, showing that lower probes need not reject its actual outer boundary."
  falsification_test: "The hypothesis is false if fixed thresholds down to 8 do not materially expand the remaining slab/noisy-border candidate, or if they materially expand and reject the known accurate dark-card diagnostic."
  fix_rationale: "Adding absolute low probes decouples confidence validation from a potentially inflated primary threshold; the detector still uses the adaptive threshold to select a candidate but abstains when weaker evidence reveals that candidate is not a stable outer boundary."
  blind_spots: "Very noisy real backgrounds may connect a valid subject to the frame at low thresholds and cause additional abstention; this is acceptable only under the stated safe-abstention preference and will be quantified across the full corpus."
  candidate_causes:
    - "code: stability thresholds are only relative to an adaptive value and therefore inherit its border-noise inflation"
    - "data: variable/noisy borders coexist with low-contrast clear slabs and pale card borders around much darker internal content"
    - "environment/config: the same failure reproduces in a synthetic PNG, ruling out JPEG decoder, EXIF, and runtime configuration as necessary causes"
  and_gate: "yes: the miss requires an inflated adaptive threshold plus a weak outer enclosure below all relative probes; fixed low probes remove the interaction while retaining the adaptive primary candidate"

## Evidence

- timestamp: 2026-10-12T00:00:00-04:00
  checked: `.planning/debug/knowledge-base.md`
  found: no debug knowledge base exists in this repository
  implication: there is no known-pattern candidate to prioritize; investigation proceeds from current implementation and common state-management patterns
- timestamp: 2026-10-12T00:00:00-04:00
  checked: repository search for image review, overlays, perspective corners, and detection messages
  found: `controller/static-admin/admin.js` owns review state initialization near lines 1959-1963, overlay rendering near 2277, perspective projection/handles near 2119-2698, assist handling near 2773, and overlay listeners near 3503; `admin.css` owns distinct grid/centerline/edges visuals; Rust assist logic and tests live in `controller/src/image_adjustments.rs` and `controller/tests/image_adjustments.rs`
  implication: the bug spans a small, traceable producer/consumer set rather than unrelated routes; common state-management and data-shape patterns are plausible candidates
- timestamp: 2026-10-12T00:00:00-04:00
  checked: complete review-state initialization and output rendering paths in `controller/static-admin/admin.js`
  found: `openImageReview` explicitly resets `state.reviewOverlays` to grid false, centerline false, edges true for every image; `renderReviewOutputPanel` creates overlay layers only from `state.reviewOverlays`; `syncReviewControls` synchronizes adjustment controls but not `[data-review-overlay]` checkboxes; checkbox DOM state changes only in the one-way `change` listener
  implication: rendered layers are correctly reset by image identity, but checkbox controls retain prior DOM state, directly explaining Grid checked with no grid layer and Edges unchecked with an edges layer
- timestamp: 2026-10-12T00:00:00-04:00
  checked: overlay rules in `controller/static-admin/admin.css`
  found: `.review-overlay-grid` is `inset: 33%` plus a rectangular border, so it is literally one central box; `.review-overlay-edges` is always `inset: 8%` plus a dashed border and receives no current-image or perspective-corner coordinates
  implication: the grid and repeated edge geometry are deterministic CSS behavior, not a projection/render timing artifact; the reported visuals are exact consequences of the implementation
- timestamp: 2026-10-12T00:00:00-04:00
  checked: `propose_image_adjustment` and its tests in `controller/src/image_adjustments.rs` and `controller/tests/image_adjustments.rs`
  found: assist thresholds grayscale pixels at 210, returns the axis-aligned bounding box of all bright pixels, and labels it confident solely from bright-pixel ratio and box dimensions; tests assert only status, corner count, and normalized range, not corner accuracy, skew detection, or material change
  implication: assist can abstain on high-contrast images whose subject is not predominantly bright and can claim confidence for a near/full-frame bright region without visible correction; existing tests cannot catch either false negative or false confidence
- timestamp: 2026-10-12T00:00:00-04:00
  checked: supplied screenshots and diagnostic WEBP
  found: screenshots exactly show the DOM/state inversion predicted by source (Grid checked, Edges unchecked, fixed dashed rectangle rendered); the diagnostic image is 914x1280 with a visually clear dark card on light background, but only 5.9051% of grayscale pixels meet luma >= 210 and those bright pixels span 914x1280
  implication: the current bright-ratio gate deterministically rejects a valid high-contrast card and its bright-pixel bounds cannot represent the card edges
- timestamp: 2026-10-12T00:00:00-04:00
  checked: existing behavior and image-adjustment tests
  found: `node controller/tests/static_admin_behavior.mjs` passes and `cargo test --manifest-path controller/Cargo.toml --test image_adjustments -- --nocapture` passes 7/7
  implication: SBFL is skipped because there is no failing test spectrum; coverage has an oracle gap rather than a localized existing failure
- timestamp: 2026-10-12T00:00:00-04:00
  checked: git history and blame for review state, overlay CSS, and assist code
  found: all affected review UI paths were introduced together in `921a397`; assist was introduced in `cdda67d`; symptoms were discovered during first testing and there is no known-good implementation to bisect against
  implication: failure class is deterministic Bohrbug; git bisect is not applicable beyond identifying the initial feature commits
- timestamp: 2026-10-12T00:00:00-04:00
  checked: newly authored regression assertions before production changes
  found: DOM harness fails because the second review retains `grid.checked === true` instead of the reset false state; detector tests fail because the skew fixture returns an axis-aligned top-left x=0.168 instead of 0.218 and the dark-card fixture returns full-frame corner (0,0) instead of the card corner near (0.214,0.168)
  implication: the tests reproduce both the dual-source-of-truth defect and both detector failure modes with specified/derived oracles
- timestamp: 2026-10-12T00:00:00-04:00
  checked: target regressions after implementation
  found: DOM behavior harness passes, all 10 image-adjustment tests pass, and the static overlay layout contract passes; new cases cover cross-image checkbox reset, edge segment reprojection after a corner move, rule-of-thirds CSS, accurate light-on-dark and dark-on-light quadrilaterals, no-correction status, and the correction/no-correction threshold neighbor
  implication: the implementation now addresses each confirmed producer rather than masking rendered symptoms
- timestamp: 2026-10-12T00:00:00-04:00
  checked: transient local diagnostic test against the supplied private WEBP
  found: the new detector returns `confident` with corners approximately (0.061,0.061), (0.925,0.056), (0.939,0.934), and (0.057,0.933), matching the visible card boundary and providing a material correction
  implication: the supplied false-negative image is fixed by the new background-contrast quadrilateral detector; the transient test file was deleted and the private artifact/path is not part of the intended code commit
- timestamp: 2026-10-12T00:00:00-04:00
  checked: adjacent static-admin contract suite during verification
  found: the first rule-of-thirds implementation used CSS gradients and failed the repository's explicit no-gradient contract; it was replaced with four ordinary line elements, after which all 19 static-admin tests passed
  implication: adjacent tests prevented an in-scope visual contract regression and the revised guide preserves the shared invariant without a policy exception
- timestamp: 2026-10-12T00:00:00-04:00
  checked: full adjacent verification after the revised fix
  found: full Cargo suite passes (185 passed, 2 live tests ignored by contract), production-persistence check passes, Clippy passes with only the pre-existing `oracle_wallet_dir` dead-code warning, Rust formatting passes, Node syntax/DOM behavior pass, and the 19-test static-admin suite passes
  implication: related controller, route, publisher, privacy, admin, and production-persistence behavior remains intact
- timestamp: 2026-10-12T12:04:03-04:00
  checked: fix-acceptance revert-and-reconfirm guardrail
  found: with only the three production files temporarily stashed and all regression tests retained, the DOM harness reproduced the inherited Grid checkbox and detector tests failed against the missing accurate/notNeeded behavior; after restoring the production files, the DOM harness and all 10 image-adjustment tests passed again
  implication: the accepted production changes, rather than unrelated environment state or weakened assertions, causally remove the reported defects
- timestamp: 2026-10-12T12:11:23-04:00
  checked: controller static-admin inclusion, Caddy serving path, container image build, and release/deploy documentation
  found: `publisher.rs` compile-time embeds `static-admin/index.html`, `admin.js`, and `admin.css` with `include_str!` and writes them into each candidate release; Caddy serves the promoted shared-volume path `/srv/autographs/static/current/admin`; the Dockerfile's additional `/opt/autographs/static-admin` copy has no runtime reader in controller source; normal deployment replaces the controller image but intentionally does not publish catalog/static content
  implication: copying JS/CSS into the running controller would not update the embedded publisher inputs or the Rust detector, and editing the promoted release would bypass the manifest/promotion contract; supported verification requires a rebuilt controller image followed by an authenticated full (or currently equivalent incremental) static publish, with full publish the safest explicit choice
- timestamp: 2026-10-12T12:23:01-04:00
  checked: human-verification checkpoint response and PR-preparation worktree inventory
  found: the user chose to open the PR first and defer production verification until that checkpoint; the fix remains exactly six code/test files with 499 insertions and 48 deletions, while the debug session remains `awaiting_human_verify`
  implication: hand off the verified local fix for PR preparation without deployment, production file copying, session archival, commit, or push; production confirmation remains the only unresolved acceptance step
- timestamp: 2026-10-12T13:43:34-04:00
  checked: exact branch detector against all 116 private local JPEG originals
  found: 114 results were labeled `Confident`, 2 were `Unavailable`, and none were `NotNeeded`; a conservative geometry screen identified 25 confident proposals that failed to approach at least one image side within 12%, with visual confirmation that representative proposals selected internal artwork/text instead of the outer card or slab
  implication: low outer-boundary contrast is not merely an intentional abstention case; the current detector produces systematic false confidence on real collection images and PR #276 must not merge without revised candidate/confidence behavior
- timestamp: 2026-10-12T13:43:34-04:00
  checked: representative full-resolution originals and returned normalized corners
  found: three representative raw-card images return central internal-content quadrilaterals; one graded-slab case excludes much of the clear slab and label; two sibling slab cases safely abstain
  implication: the largest background-contrast connected component is often the card's internal content when a pale border blends with the background, and current area/geometry checks are insufficient to justify `Confident`
- timestamp: 2026-10-12T13:46:48-04:00
  checked: complete detector and image-adjustment test implementation
  found: confidence is based only on the single component found at one adaptive threshold plus a broad 15%-98.5% quadrilateral-area gate; the detector never compares the candidate against lower contrast thresholds that could reveal a larger enclosing card or slab
  implication: a high-contrast internal rectangle and a genuine dark card are indistinguishable at the current threshold, but an enclosing low-contrast boundary can be exposed as ambiguity by testing candidate stability below that threshold
- timestamp: 2026-10-12T13:47:53-04:00
  checked: agent-authored synthetic low-contrast enclosure regressions and high-contrast boundary neighbor
  found: both low-contrast card/slab tests fail red because the detector returns `Confident` instead of `Unavailable`; the high-contrast enclosing card, existing dark-card, skew, identity, and all unrelated image-adjustment tests pass
  implication: the failure is isolated to missing confidence evidence for contrast-hierarchical inputs, and a blanket abstention for nested content would be incorrect because the high-contrast outer boundary remains detectable and accurate
- timestamp: 2026-10-12T13:48:50-04:00
  checked: focused detector suite after multi-threshold component-stability implementation
  found: all 13 image-adjustment tests pass; the two low-contrast enclosure cases now abstain, while accurate dark-card, skew, high-contrast enclosing-border, near-identity, and threshold-neighbor behavior remains unchanged
  implication: the minimal stability gate fixes the synthetic root-cause class without deleting confident detection behavior; production-like corpus evaluation is the next held-out signal
- timestamp: 2026-10-12T13:56:24-04:00
  checked: hardened detector against all 116 private local originals
  found: results changed from 114 confident / 2 unavailable / 25 geometric suspects to 63 confident / 53 unavailable / 1 geometric suspect; three representative internal-content proposals now abstain and the two already-safe slab siblings remain unavailable, but the remaining graded-slab case still confidently returns an internal region bounded roughly x=0.186..0.826 and y=0.286..0.879
  implication: the stability gate substantially reduces known false confidence but is incomplete for a graded slab whose visible outer boundary is not captured as material growth of the selected anchor-connected component; the remaining known failure must be represented before accepting the fix
- timestamp: 2026-10-12T13:57:26-04:00
  checked: exact threshold/component evolution for remaining known corpus failure and supplied accurate dark-card diagnostic
  found: the remaining graded-slab case uses adaptive threshold 234 and fractional probes 175/117/78, whose selected component grows in population but only from area 0.3431 to 0.3849; the accurate dark-card diagnostic uses threshold 66 and stays geometrically stable through 49/33/22 with only 7.2% population growth
  implication: the first fix's relative probes never reach low-contrast evidence when border variability inflates the primary threshold; fixed low probes can distinguish the slab case while the known accurate dark-card already remains stable down to threshold 22
- timestamp: 2026-10-12T13:57:58-04:00
  checked: sanitized noisy-border/low-contrast-enclosure regression before the second detector revision
  found: the focused test fails deterministically because the current detector returns `Confident` for the inner dark rectangle instead of `Unavailable`
  implication: the remaining corpus mechanism is reproduced without private data and provides a derived-oracle regression for the missing absolute stability floor
- timestamp: 2026-10-12T13:59:00-04:00
  checked: fixed low-threshold probes against focused tests, remaining slab, and supplied accurate dark-card diagnostic
  found: all 14 image-adjustment tests pass; the remaining graded-slab case now abstains after its component expands from area 0.3431/count 54,378 to area 0.9626/count 220,852 at threshold 32; the supplied diagnostic remains confident and accurate through threshold 8 because its component population grows only 24.91% despite low-threshold background noise
  implication: the fixed probe reaches evidence hidden by the inflated adaptive threshold and distinguishes the remaining known false-confidence case without regressing the known dark-card success; full-corpus counts remain the held-out acceptance signal
- timestamp: 2026-10-12T14:00:24-04:00
  checked: final detector against all 116 private local originals and worktree contents after evaluator cleanup
  found: final results are 19 confident / 97 unavailable / 0 not-needed, with zero confident proposals in the conservative geometric screen that previously contained 25 known failures; all six named representative slab/internal-content images abstain, the supplied accurate dark-card diagnostic remains confident, and only the production detector plus synthetic test file are modified
  implication: the fix meets the explicit safe-abstention preference and removes the known minimum false-confidence set without copying or retaining any private evaluation artifact
- timestamp: 2026-10-12T14:01:33-04:00
  checked: full adjacent Rust guardrails and mutation-tool availability
  found: 189 tests pass with 2 live contract tests ignored, production-persistence check passes, Clippy passes with only the pre-existing `oracle_wallet_dir` dead-code warning, formatting and diff checks pass; no cargo-mutants tool or configuration is present
  implication: adjacent behavior remains intact and mutation testing must be recorded as unavailable; revert-and-reconfirm is the final automated causal signal
- timestamp: 2026-10-12T17:26:55-04:00
  checked: fix-acceptance revert-and-reconfirm for detector confidence hardening
  found: with only the new production stability gate removed and all new regressions retained, exactly the three low-contrast enclosure tests failed as `Confident`; after reapplying the production gate, all 14 image-adjustment tests, formatting, and diff checks passed
  implication: the multi-threshold stability gate causally removes the false-confidence class and the regressions are not passing because of unrelated branch state or weakened assertions

## Eliminated

- hypothesis: stale async review or assist responses overwrite the next image's state
  evidence: image/session/revision guards are present and the deterministic mismatch occurs after `openImageReview` resets JS state; the surviving stale value is only the unsynchronized checkbox property
  timestamp: 2026-10-12T12:04:03-04:00
- hypothesis: saved backend adjustment metadata is leaking between image records
  evidence: each review initializes draft/saved adjustments from its own response, while the reproduced leak persists only in overlay DOM controls that are not part of adjustment persistence
  timestamp: 2026-10-12T12:04:03-04:00
- hypothesis: browser, configuration, or runtime differences cause the fixed guide geometry
  evidence: the exact central box and repeated dashed rectangle are defined by invariant `inset:33%` and `inset:8%` CSS and reproduce in screenshots without environment-dependent branches
  timestamp: 2026-10-12T12:04:03-04:00

## Resolution

- root_cause: "Overlay state was split between JS and persistent checkbox DOM without state-to-control synchronization; overlay visuals were hardcoded CSS rectangles unrelated to current image/corners; the original absolute-brightness assist rejected dark cards, while its replacement can falsely trust internal content because single-threshold component area does not establish a stable outer boundary when a low-contrast card/slab encloses higher-contrast content."
- fix: "Make review state authoritative for overlay controls; render rule-of-thirds, centerline, and corner-connected edge guides over the current contained source image; use background-contrast quadrilateral detection with explicit confident/notNeeded/unavailable outcomes, hardened by multi-threshold boundary-stability checks that abstain when a selected component materially expands into a weakly contrasted enclosure."
- verification:
    target_test: { result: pass, suites: ["node controller/tests/static_admin_behavior.mjs", "cargo test --manifest-path controller/Cargo.toml --test image_adjustments", "cargo test --manifest-path controller/Cargo.toml --test static_admin"] }
    mutation_check: { result: skipped, reason_if_skipped: "No Stryker or cargo-mutants configuration/tooling exists in the repository", mutant_killed: false }
    no_op_deletion: { result: pass, deletion_justified_by_rca: false }
    adjacent_tests: { result: pass, suites_run: ["full controller Cargo suite: 189 passed, 2 contract-ignored", "cargo check --features production-persistence", "cargo clippy --all-targets", "cargo fmt --check", "node --check", "git diff --check"] }
    revert_and_reconfirm: { result: pass, bug_returned_on_revert: true, fixed_on_reapply: true }
    private_diagnostic: { result: pass, detail: "supplied WEBP retained its material confident quadrilateral; 116-image corpus changed from 114 confident / 2 unavailable / 25 geometric suspects to 19 confident / 97 unavailable / 0 geometric suspects; transient evaluator removed" }
    human_verification: { result: deferred, checkpoint: "after PR/release deployment and authenticated full static publish" }
    guardrail_verdict: accepted
- files_changed: "controller/static-admin/admin.js, controller/static-admin/admin.css, controller/src/image_adjustments.rs, controller/tests/static_admin_behavior.mjs, controller/tests/static_admin.rs, controller/tests/image_adjustments.rs"
- oracle_type: "specified for state/overlay behavior; derived from known synthetic quadrilaterals for detection accuracy"
