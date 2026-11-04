# Image review layout sketch

Open [image-review-workbench.html](image-review-workbench.html) in a browser. It is a standalone layout sketch based on the live 0.7.0 admin screenshots. The artwork and JSON are dummy data; no requests or saves occur.

The dark preview area fills the left column to viewport height on desktop, with larger images centered in each half. The right rail uses smaller type, tighter spacing, inline slider values, one-row overlay toggles, shared coordinate headers, and click-open help for explanatory text. The JSON panel uses a dark background with light text and stays collapsed until needed. On narrow screens, the controls move below the preview.

Review [the desktop preview](image-review-workbench-preview.png) for the initial layout. The HTML also contains a few local interactions for help, grid overlays, and slider values so their spacing can be assessed. Comparison and action buttons are visual placeholders.

After the layout is approved, adapt the production admin markup, CSS, and render logic, preserving its actual image adjustment behavior and accessibility labels.
