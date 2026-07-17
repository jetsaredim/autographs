---
status: resolved
trigger: "Merged/deployed admin UI still shows strange item-list misalignment after hard refresh."
created: 2026-07-16
updated: 2026-07-16
---

# Debug Session: Admin Item List Link Alignment

## Symptoms

- Expected behavior: admin item-list rows should read as cleanly aligned text cells on the left and icon/action controls on the right.
- Actual behavior: after the merged deployment and a hard refresh, rows still show strange visual misalignment.
- Error messages: none reported.
- Timeline: observed after PR #179 was merged and deployed.
- Reproduction: open the deployed admin item list after hard refresh.

## Current Focus

- hypothesis: signer link buttons stretch across the grid cell, making the subtle border appear as a full-width horizontal rule and creating apparent row misalignment.
- test: constrain inline signer link controls to their content width and rerun static admin checks.
- expecting: signer names retain a non-underlined clickable affordance without drawing full-cell horizontal lines.
- next_action: patch `.inline-link` sizing/alignment and verify.

## Evidence

- 2026-07-16: screenshot `2026-07-15_21-35.png` shows signer name controls producing full-width horizontal lines within the signer/taxonomy column.
- 2026-07-16: CSS uses `.signer-cell-content { display: grid; }`; grid children stretch by default.
- 2026-07-16: `.inline-link` is applied to button elements and does not override default stretch behavior with `justify-self`.

## Eliminated

- hypothesis: browser cache alone caused stale CSS.
  reason: user performed hard refresh and the screenshot matches current CSS behavior.

## Resolution

- root_cause: `.inline-link` was used on signer buttons inside a CSS grid. Grid children stretch by default, so the rounded border stretched across the signer cell and read as a full-width horizontal divider/misalignment.
- fix: constrained `.inline-link` to an inline-flex, content-width control with `justify-self: start`, left-aligned content, and `max-width: 100%`.
- verification: `node --check controller/static-admin/admin.js`; `cargo test --test static_admin`.
- files_changed: `controller/static-admin/admin.css`; `controller/tests/static_admin.rs`.
