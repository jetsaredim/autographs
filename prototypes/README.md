# Image review layout sketch

Open [image-review-workbench.html](image-review-workbench.html) in a browser. It is a standalone layout sketch based on the live 0.7.0 admin screenshots. The artwork and JSON are dummy data; no requests or saves occur.

The dark preview area fills the left column to viewport height on desktop, with larger images centered in each half. The right rail uses smaller type, tighter spacing, inline slider values, one-row overlay toggles, shared coordinate headers, and floating help popups that do not expand the panels. The JSON panel uses a dark background with light text and stays collapsed until needed. On narrow screens, the controls move below the preview.

Review [the desktop preview](image-review-workbench-preview.png) for the initial layout, [the help popup preview](image-review-workbench-popup-preview.png), and [the JSON comparison preview](image-review-workbench-json-preview.png). The HTML also contains a few local interactions for help, grid overlays, and slider values so their spacing can be assessed. Help popups close on Escape, outside click, or scrolling. The adjustment JSON button opens a centered saved/draft comparison and highlights values changed by the mockup controls. Comparison and action buttons are visual placeholders.

After the layout is approved, adapt the production admin markup, CSS, and render logic, preserving its actual image adjustment behavior and accessibility labels.

## Item editor workbench

Open [item-editor-workbench.html](item-editor-workbench.html) to compare two compact item-form layouts inside the current admin header, session message, navigation tabs, and editor panel. Variant A keeps metadata in a tight left rail beside image records. Variant B uses two compact metadata columns above the image records. The Edit/Add toggle shows how the same spacing works for a new item; images use an empty state until that item is saved. The width toggle compares the current 1120px admin shell with the 1500px shell used by image review. The views can also be opened directly with `?layout=b`, `?mode=add`, or `?width=wide`.

Preview captures: [metadata rail at current width](item-editor-workbench-a.png), [metadata rail at wide width](item-editor-workbench-a-wide.png), [two-column form at current width](item-editor-workbench-b.png), and [add item](item-editor-workbench-add.png). Open the HTML to scroll and switch among the layouts, modes, and widths.

The adjusted primary image shows an unadjusted source and saved result side by side. A supporting image without saved adjustment JSON keeps a single preview. Artwork and values are dummy content, and no action saves or calls the server. The toggle buttons only switch the mockup layout. After choosing a direction, transfer its spacing and image presentation into the production admin editor without changing the item list or image review page.
