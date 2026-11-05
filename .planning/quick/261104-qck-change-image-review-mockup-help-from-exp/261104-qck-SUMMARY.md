---
quick: 261104-qck
status: complete
implementation_commit: d10786b
---

# Floating help popups

Changed all mockup question-mark help controls to open an anchored popup outside the scrollable control rail. The popup leaves panel height unchanged and closes on Escape, outside click, scrolling, or resize. Added a desktop popup preview image.

Verified the open state through Chrome's rendered DOM and visually inspected desktop and 430 px wide screenshots. Ran `git diff --check`. Production admin files remain untouched.
