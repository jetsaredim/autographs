---
quick_id: 261105-bnc
status: planned
---

# Fix admin media review panel borders and framing target

## Goal

The review control groups show a continuous top border between each title and help icon, and the background framing workflow offers a crop when measured margins exceed the requested target, including the reported 5% margins with a 4% target.

## Tasks

1. Update `controller/static-admin/admin.css` so the full-width legend retains its current title and help positions while a visible border segment connects them. Verify the rendering with focused CSS inspection and the admin static checks.
2. Update `controller/static-admin/admin.js` so the Balanced assessment uses the requested target within a small measurement tolerance. Keep real-pixel availability and invalid-pixel protections. Add a regression to `controller/tests/static_admin_behavior.mjs` covering a 4% target against balanced 5% margins and a no-op when margins already match the target. Verify with `node controller/tests/static_admin_behavior.mjs` and `cargo test --test static_admin` from `controller/`.

## Must haves

- Help controls remain aligned at the right edge and accessible.
- A 4% target offers a 4% crop when the real photographed band supports it.
- A crop is not suggested when margins are already close to the selected target.
- Existing crop, invalid-pixel, and available-band guards remain in force.
