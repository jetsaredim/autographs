---
quick: 261104-qmx
status: complete
implementation_commit: 279234d
---

# Saved and draft JSON comparison

Replaced the rail's inline JSON details with a centered dialog. It shows example saved adjustment values on the left and current mockup control values on the right. Changed field rows are highlighted on both sides, with a change count. The dialog stacks the two values panels on narrow screens and can be closed with the Close button, Escape, or a backdrop click.

Verified the open dialog in Chrome's rendered DOM, including the two initial changed fields. Rendered and visually inspected desktop and 430 px mobile previews. Ran `git diff --check`. Production admin files remain unchanged.
