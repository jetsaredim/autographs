---
status: passed
quick_id: 261105-bnc
---

# Verification

| Requirement | Evidence |
|---|---|
| Visible border between title and help icon | `.review-legend-row::after` draws the missing line and `.review-help` remains at the right edge in `controller/static-admin/admin.css`. |
| Honor 4% target with available 5% photographed margins | Target-aware assessment in `controller/static-admin/admin.js`; regression in `controller/tests/static_admin_behavior.mjs` confirms `Adjustable` status and a 4% proposal. |
| No suggestion when margins match target | Regression confirms `Balanced` status and no proposal for 4% margins with a 4% target. |
| Preserve photographed-pixel guards | Existing invalid-pixel, existing-crop, and maximum-band branches remain unchanged. |

Focused admin behavior harness and all 20 `static_admin` tests pass. CSS rendering has not been visually checked in a live browser; the line is verified by source inspection.
