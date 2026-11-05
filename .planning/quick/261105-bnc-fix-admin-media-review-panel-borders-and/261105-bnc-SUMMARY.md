---
status: complete
quick_id: 261105-bnc
---

# Summary

- Added a visible border segment between each full-width control legend title and its help icon.
- Made the framing assessment compare measured margins with the selected background target. A balanced 5% source now offers a 4% crop, while margins already at the selected target produce no crop suggestion.
- Added a browser-behavior harness regression for both cases.

Code commit: `13f5323`.

Verification: `node controller/tests/static_admin_behavior.mjs`, `cargo test --test static_admin` from `controller/`, and `git diff --check` passed.
