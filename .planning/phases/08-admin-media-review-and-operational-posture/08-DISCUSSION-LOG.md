# Phase 8: Admin Media Review and Operational Posture - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md - this log preserves the alternatives considered.

**Date:** 2026-07-28
**Phase:** 8-Admin Media Review and Operational Posture
**Areas discussed:** Admin previews, security reports, posture pass, CDN sequencing, image adjustment

---

## Admin Previews

| Option | Description | Selected |
|--------|-------------|----------|
| Show thumbnails | Saved image tiles show small authenticated same-origin previews by default. | yes |
| Click to reveal | Tiles stay compact until the admin chooses to load an image preview. | |
| Preview panel | Selecting an image opens a larger preview panel while tiles remain list-like. | |

**User's choice:** Show thumbnails.

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, focused view | Add an authenticated larger inspect view. | yes |
| Thumbnail only | Keep Phase 8 preview work compact. | |
| Open image route | Open a same-origin preview route in a new tab. | |

**User's choice:** Yes, focused view.

| Option | Description | Selected |
|--------|-------------|----------|
| Private latest | Show current saved private image/adjustment state, with pending badges. | |
| Published current | Show what the public site currently serves until publish. | |
| Compare both | Compare private latest and public current when they differ. | yes |

**User's choice:** Compare both.

| Option | Description | Selected |
|--------|-------------|----------|
| Focused view | Tiles show pending state; focused view compares private latest and public current. | yes |
| Inline tiles | Each tile shows both states directly. | |
| Always side-by-side | Inspect view always shows both states. | |

**User's choice:** Focused view.

| Option | Description | Selected |
|--------|-------------|----------|
| Bounded review size | Generate capped private previews for inspection. | |
| Original size | Show private original at full resolution through the controller. | |
| Public detail size | Use roughly the public detail derivative size. | yes |

**User's choice:** Public detail size.

| Option | Description | Selected |
|--------|-------------|----------|
| Immediately after save | Refresh preview after upload/replace API success. | yes |
| After publish | Preview appears only after public derivatives are published. | |
| Local then saved | Show browser-local preview during upload, then replace with saved preview. | |

**User's choice:** Immediately after save.

| Option | Description | Selected |
|--------|-------------|----------|
| Redacted problem state | Show a non-secret error with retry/replace context. | yes |
| Silent placeholder | Show a neutral unavailable state. | |
| Detailed diagnostics | Show richer operator details in admin. | |

**User's choice:** Redacted problem state.

| Option | Description | Selected |
|--------|-------------|----------|
| Private-only state | Show private preview with unpublished/new badge. | yes |
| Empty public pane | Show a side-by-side public-unavailable pane. | |
| Hide compare | Offer comparison only after a public image exists. | |

**User's choice:** Private-only state for never-published images.

**Notes:** Admin previews should be part of normal item management. They remain
authenticated, same-origin, and redacted; they should not expose direct Object
Storage URLs, bucket/object identifiers, original filenames, image UUIDs, or
unpublished media publicly.

---

## Security Reports

| Option | Description | Selected |
|--------|-------------|----------|
| Concise + links | Show package specs/advisories and external links without VM detail loops. | |
| Minimal inventory | Only show package specs and advisory IDs. | |
| Full enriched report | Keep CVE-level detail but enrich off the live host path. | |
| Other | VM extracts the minimal CVE/errata list, then detail comes from Oracle OVAL or static errata pages. | yes |

**User's choice:** Other.

**Notes:** The VM should extract the minimum useful package/advisory/CVE/errata
inventory. Richer detail should come from Oracle Linux OVAL files or static
HTML pages on `linux.oracle.com`, not from slow per-advisory `dnf updateinfo
info` loops on the runtime host.

| Option | Description | Selected |
|--------|-------------|----------|
| OVAL first | Use Oracle OVAL as structured source; errata HTML as fallback/link target. | yes |
| Errata pages first | Scrape static Oracle errata pages first. | |
| Either is fine | Planner may choose whichever source proves reliable. | |

**User's choice:** OVAL first.

| Option | Description | Selected |
|--------|-------------|----------|
| Create issue anyway | Open/update issue with minimal inventory and note enrichment failure. | yes |
| Fail the scan | Do not open an issue unless enrichment succeeds. | |
| Retry then issue | Retry enrichment briefly, then create minimal issue. | |

**User's choice:** Create issue anyway.

| Option | Description | Selected |
|--------|-------------|----------|
| Package specs only | Hidden metadata stores exact package specs for drift checks. | yes |
| Packages + advisories | Hidden metadata also stores advisory IDs. | |
| Full report data | Hidden metadata stores report details too. | |

**User's choice:** Package specs only.

| Option | Description | Selected |
|--------|-------------|----------|
| Summary + links | Include severity, CVE IDs, package rows, and links; avoid long copied descriptions. | yes |
| CVE/package rows | Mostly tabular issue body. | |
| Rich summaries | Include generated summaries of advisory descriptions. | |

**User's choice:** Summary + links.

| Option | Description | Selected |
|--------|-------------|----------|
| Live scan proof | Prove scanner completes and creates/updates issue. | |
| Scan + dry-run apply | Also exercise approval validation and drift-check paths without installing packages where feasible. | yes |
| Full live apply | Require approved production update run before closing Phase 8. | |

**User's choice:** Scan + dry-run apply.

| Option | Description | Selected |
|--------|-------------|----------|
| Update same issue | Replace issue body/metadata and remove stale approval state. | yes |
| Close and recreate | Close stale issue and create a fresh one. | |
| Append history | Add comments describing changes between scans. | |

**User's choice:** Update same issue.

| Option | Description | Selected |
|--------|-------------|----------|
| Group by severity | Put Critical/Important first. | |
| Keep DNF order | Reflect host inventory order exactly. | |
| Group by package | Make package families easier to review. | |
| Other | Re-review during development; DNF order or alphabetical package/version may make more sense. | yes |

**User's choice:** Other.

**Notes:** Ordering should be decided after inspecting real scan output. Severity
grouping is not locked.

---

## Posture Pass And CDN Sequencing

| Option | Description | Selected |
|--------|-------------|----------|
| Findings register + fixes | Create concise findings list, fix low-risk issues, track larger follow-ups. | yes |
| Fix-only cleanup | Only record issues fixed in Phase 8. | |
| Full audit report | Produce a formal repo-wide report even for deferred items. | |

**User's choice:** Findings register + fixes.

| Option | Description | Selected |
|--------|-------------|----------|
| Low-risk only | Fix stale docs, naming text, validation gaps, and simple workflow consistency issues. | |
| Moderate cleanup | Also rename confusing config/Terraform variables when impact is small. | |
| Aggressive cleanup | Use Phase 8 to pay down broader structure and naming debt. | yes |

**User's choice:** Aggressive cleanup.

| Option | Description | Selected |
|--------|-------------|----------|
| Within planned surfaces | Allow broader cleanup in security, docs, Terraform naming, workflows, and validation. | |
| Repo-wide rename pass | Permit coordinated naming/config cleanup across the repo. | |
| Separate cleanup PRs | Split larger fixes into their own PRs before/alongside media work. | |
| Other | Separate PRs; do this before media work; add guardrails to enforce hygiene going forward. | yes |

**User's choice:** Other.

| Option | Description | Selected |
|--------|-------------|----------|
| CI checks | Add or tighten automated validation for docs/map drift, workflow syntax, naming, and privacy/static contracts. | yes |
| Review checklist | Use human review checklists/runbooks. | |
| CI + checklist | Combine automation and human checklists. | |

**User's choice:** CI checks.

| Option | Description | Selected |
|--------|-------------|----------|
| Revalidate only | Confirm deferral remains sound and avoid CDN changes in Phase 8. | |
| Prepare follow-up | Revalidate and create a follow-up plan if useful. | |
| Enable if small | Enable CDN only if low-risk and bounded. | |
| Other | Add CDN review/implementation as a first-class Phase 8 objective. | yes |

**User's choice:** Other.

| Option | Description | Selected |
|--------|-------------|----------|
| Amend Phase 8 | Update phase scope/objectives before treating CDN implementation as required. | yes |
| Defer to later | Keep Phase 8 to revalidation only. | |
| Allow if small | Keep current roadmap wording. | |

**User's choice:** Amend Phase 8.

| Option | Description | Selected |
|--------|-------------|----------|
| Contract before media | Do security/posture first, define CDN/cache rules before media, build media, then enable/verify CDN. | yes |
| CDN before media | Fully enable CDN before admin media work. | |
| Media before CDN | Build previews/adjustments first, then do CDN review and implementation. | |

**User's choice:** Contract before media.

**Notes:** The user raised that existing images will likely be updated during
the media editor work, which has caching implications. Final sequencing:
security repair, hygiene cleanup/CI guardrails, CDN/cache contract, media
preview/adjustment, then CDN enablement/verification after media cache behavior
is testable.

---

## Image Adjustment

| Option | Description | Selected |
|--------|-------------|----------|
| From image tile | Each tile has an Adjust action opening a focused editor. | |
| Dedicated review view | Use a separate image-review screen for focused adjustment work. | |
| Inline controls | Put crop/rotate/pan controls directly on each image tile. | |
| Other | Dedicated review view, with visible guideline overlays for rectangular shape/skew. | yes |

**User's choice:** Other.

**Notes:** Adjustment should live in a dedicated review view, with image tiles as
the natural entry point. The review view should include guide overlays showing
rectangular shape/alignment to help identify skew and misalignment.

| Option | Description | Selected |
|--------|-------------|----------|
| Toggleable grid | Optional rectangular/rule-of-thirds overlay. | |
| Always-on frame | Simple rectangular target frame visible by default. | |
| Multiple overlays | Selectable overlays such as grid, centerlines, and card-edge guides. | yes |

**User's choice:** Multiple overlays.

| Option | Description | Selected |
|--------|-------------|----------|
| Rotate/crop/pan/zoom | Store bounded rotation plus normalized crop, pan, and zoom metadata. | |
| Add deskew too | Include deskew/perspective correction if feasible. | yes |
| Crop only first | Start with crop/zoom/pan and defer rotation. | |

**User's choice:** Add deskew too.

| Option | Description | Selected |
|--------|-------------|----------|
| Feasibility-gated | Ship deskew only if low-risk implementation is found. | |
| Must ship | Treat deskew/perspective correction as required Phase 8 functionality. | yes |
| Visual guides only | Defer actual deskew correction controls. | |

**User's choice:** Must ship.

| Option | Description | Selected |
|--------|-------------|----------|
| Manual corners | Admin drags four corners/edge guides. | |
| Auto + tweak | System tries edge/skew detection, then admin can adjust manually. | yes |
| Angle slider | Correct rotational skew only. | |

**User's choice:** Auto + tweak.

| Option | Description | Selected |
|--------|-------------|----------|
| Manual handles | Fall back to draggable corners/edges when auto detection is uncertain. | yes |
| Skip deskew | Disable deskew for that image. | |
| Warn and save none | Prevent saving deskew metadata until detection works. | |

**User's choice:** Manual handles.

| Option | Description | Selected |
|--------|-------------|----------|
| Draft then save | Edits stay local until Save; Cancel abandons; Reset clears saved metadata. | yes |
| Autosave drafts | Persist draft adjustments as controls move. | |
| Save per control | Each control saves immediately. | |

**User's choice:** Draft then save.

| Option | Description | Selected |
|--------|-------------|----------|
| Pending until publish | Private previews update immediately; public changes wait for normal publish. | yes |
| Auto-publish image | Save triggers a publish for the item derivatives. | |
| Private only | Require a separate apply-to-public action before publish. | |

**User's choice:** Pending until publish.

| Option | Description | Selected |
|--------|-------------|----------|
| Toggle + split | Offer before/after toggle plus split comparison. | yes |
| Side-by-side | Always show original and adjusted preview side by side. | |
| After only | Show adjusted result by default. | |

**User's choice:** Toggle + split.

---

## the agent's Discretion

- Exact report row ordering after inspecting real security scan output, with DNF
  order or alphabetical package/version as preferred candidates.
- Exact UI mechanics for overlay selection and focused review layout, provided
  the dedicated review view, multiple overlays, auto-assisted deskew, manual
  fallback, draft-local save model, and before/after comparison are preserved.

## Deferred Ideas

- Taxonomy cue assets and public cue rendering remain Phase 9.
- OCR/AI image analysis or metadata suggestions remain Phase 10.
- Broad admin redesign outside the media review workflow remains out of scope.
- Batch image correction and content-aware enhancement are deferred beyond the
  required Phase 8 deskew/perspective correction.

---

*Phase: 8-Admin Media Review and Operational Posture*
*Discussion captured: 2026-07-28*
