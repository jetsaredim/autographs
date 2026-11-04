---
quick: 261104-ptm
status: complete
implementation_commit: a2a1701
---

# Local image review layout sketch

Created a standalone HTML workbench mockup at `prototypes/image-review-workbench.html` and a desktop preview image. It uses dummy artwork and no production data or network calls.

The desktop stage fills the left work area and shows larger source/output previews. The right rail has compact comparison and overlay controls, inline slider values, a shared X/Y coordinate header, click-open explanations, and a high-contrast JSON panel. Narrow screens stack the rail below the preview.

Verified by rendering the page in headless Chrome at 1694 × 938 and 430 × 900, visually inspecting both images, and running `git diff --check`. Production admin files were not changed. The next step is user review of the layout before porting it to the application.
