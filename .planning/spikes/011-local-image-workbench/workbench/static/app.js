const $ = (selector) => document.querySelector(selector);
const identity = () => ({
  rotationDegrees: 0,
  zoom: 1,
  panX: 0,
  panY: 0,
  crop: null,
  perspective: null,
});
const fullCorners = () => [
  { x: 0, y: 0 }, { x: 1, y: 0 }, { x: 1, y: 1 }, { x: 0, y: 1 },
];
const cornerNames = ["Top left", "Top right", "Bottom right", "Bottom left"];
const edgeNames = ["Top", "Right", "Bottom", "Left"];
const cornerInsets = [
  [["Left inset", "left"], ["Top inset", "top"]],
  [["Right inset", "right"], ["Top inset", "top"]],
  [["Right inset", "right"], ["Bottom inset", "bottom"]],
  [["Left inset", "left"], ["Bottom inset", "bottom"]],
];

let images = [];
let selectedId = 0;
const capability = new URLSearchParams(location.search).get("cap") || "";
let adjustment = identity();
let renderTimer = null;
let renderGeneration = 0;
let previewUrl = null;
let dragIndex = null;
let measurementCorners = defaultMeasurementCorners();
let measurementTouched = new Set();
let measurementDragIndex = null;
let measurementOrigin = "automatic";
let detectedBoundary = { status: "unavailable", corners: [], edges: [], framingSupport: null };
let alignmentSuggestion = null;
let alignmentApplying = false;
let alignmentSuppression = null;
let framingProposal = null;
let framingPreviewVisible = false;
let framingDismissalSignature = null;
let framingUndoAdjustment = null;
let manualFramingSupport = null;
let manualFramingSignature = null;
let manualFramingReassessAfterRender = false;

function defaultMeasurementCorners() {
  return [
    { x: .05, y: .03 }, { x: .95, y: .03 }, { x: .95, y: .97 }, { x: .05, y: .97 },
  ];
}

function corners() {
  return adjustment.perspective?.corners || fullCorners();
}

function canonicalAdjustment() {
  const next = structuredClone(adjustment);
  const values = corners();
  const isFull = values.every((point, index) =>
    Math.abs(point.x - fullCorners()[index].x) < 1e-9 &&
    Math.abs(point.y - fullCorners()[index].y) < 1e-9
  );
  next.perspective = isFull ? null : { corners: values };
  return next;
}

function manualFramingKey() {
  return JSON.stringify({
    image: selectedId,
    adjustment: canonicalAdjustment(),
    corners: measurementCorners,
  });
}

function invalidateManualFramingSupport() {
  manualFramingSupport = null;
  manualFramingSignature = null;
}

function log(message) {
  const item = document.createElement("li");
  item.textContent = `${new Date().toLocaleTimeString()} — ${message}`;
  $("#events").prepend(item);
}

function status(message, kind = "normal") {
  const node = $("#status");
  node.textContent = message;
  node.style.borderLeftColor = kind === "error" ? "var(--red)" : kind === "success" ? "var(--green)" : "var(--cyan)";
}

async function api(path, options = {}) {
  const headers = new Headers(options.headers || {});
  if (path.startsWith("/api/")) headers.set("x-workbench-capability", capability);
  const response = await fetch(path, { cache: "no-store", ...options, headers });
  if (!response.ok) {
    throw new Error((await response.text()) || `HTTP ${response.status}`);
  }
  return response;
}

async function start() {
  try {
    $("#instance-address").textContent = location.host;
    images = await (await api("/api/images")).json();
    const select = $("#image-select");
    for (const image of images) {
      const option = document.createElement("option");
      option.value = image.id;
      option.textContent = `${image.name}${image.hasFixture ? " • fixture" : ""}`;
      select.append(option);
    }
    buildCornerInputs();
    bindControls();
    const requestedId = Number(new URLSearchParams(location.search).get("image"));
    await selectImage(Number.isInteger(requestedId) && images[requestedId] ? requestedId : 0);
  } catch (error) {
    status(error.message, "error");
  }
}

async function selectImage(id) {
  selectedId = Number(id);
  history.replaceState(null, "", `?cap=${encodeURIComponent(capability)}&image=${selectedId}`);
  detectedBoundary = { status: "unavailable", corners: [], edges: [], framingSupport: null };
  framingProposal = null;
  framingPreviewVisible = false;
  framingDismissalSignature = null;
  framingUndoAdjustment = null;
  invalidateManualFramingSupport();
  manualFramingReassessAfterRender = false;
  loadMeasurement();
  $("#image-select").value = selectedId;
  const sourceImage = $("#source-image");
  sourceImage.classList.add("is-loading");
  $("#source-loading").hidden = false;
  sourceImage.src = `/api/images/${selectedId}/source?cap=${encodeURIComponent(capability)}&fresh=${Date.now()}`;
  adjustment = identity();
  await loadFixture(false);
  syncAll();
  log(`Selected ${images[selectedId].name}`);
}

async function loadFixture(announce = true) {
  try {
    const response = await api(`/api/images/${selectedId}/fixture`);
    if (response.status === 204) {
      adjustment = identity();
      if (announce) status("No saved fixture; reset to identity adjustment.");
      return;
    }
    if (!response.ok) throw new Error(await response.text());
    adjustment = await response.json();
    if (announce) {
      status("Fixture reloaded from the external workspace.", "success");
      log("Reloaded fixture");
    }
  } catch (error) {
    status(error.message, "error");
  }
}

function buildCornerInputs() {
  const host = $("#corner-inputs");
  cornerNames.forEach((name, index) => {
    const card = document.createElement("div");
    card.className = "corner-card";
    const controls = cornerInsets[index].map(([label, edge]) =>
      `<label>${label}<input type="number" min="0" max="1" step="0.001" data-corner="${index}" data-inset="${edge}"></label>`
    ).join("");
    card.innerHTML = `<strong>${index + 1}. ${name}</strong><div class="xy">${controls}</div>`;
    host.append(card);
  });
}

function cornerInsetValue(point, edge) {
  return edge === "left" ? point.x
    : edge === "right" ? 1 - point.x
      : edge === "top" ? point.y
        : 1 - point.y;
}

function cornerInsetSummary(point, index) {
  return cornerInsets[index]
    .map(([label, edge]) => `${label} ${cornerInsetValue(point, edge).toFixed(4)}`)
    .join(", ");
}

function bindControls() {
  $("#image-select").addEventListener("change", (event) => selectImage(event.target.value));
  $("#previous").addEventListener("click", () => selectImage((selectedId - 1 + images.length) % images.length));
  $("#next").addEventListener("click", () => selectImage((selectedId + 1) % images.length));
  $("#detect").addEventListener("click", detect);
  $("#save").addEventListener("click", saveFixture);
  $("#reload").addEventListener("click", async () => { await loadFixture(); syncAll(); });
  $("#reset").addEventListener("click", () => { adjustment = identity(); syncAll(); log("Reset draft"); });
  $("#apply-json").addEventListener("click", applyJson);
  for (const [selector, property] of [["#rotation", "rotationDegrees"], ["#zoom", "zoom"], ["#pan-x", "panX"], ["#pan-y", "panY"]]) {
    $(selector).addEventListener("input", (event) => {
      adjustment[property] = Number(event.target.value);
      draftChanged();
    });
  }
  $("#corner-inputs").addEventListener("input", (event) => {
    if (!event.target.dataset.corner) return;
    const values = corners().map((point) => ({ ...point }));
    const index = Number(event.target.dataset.corner);
    const inset = clamp(Number(event.target.value));
    const edge = event.target.dataset.inset;
    if (edge === "left") values[index].x = inset;
    if (edge === "right") values[index].x = 1 - inset;
    if (edge === "top") values[index].y = inset;
    if (edge === "bottom") values[index].y = 1 - inset;
    adjustment.perspective = { corners: values };
    draftChanged();
  });
  for (const selector of ["#grid-spacing", "#grid-x", "#grid-y"]) {
    $(selector).addEventListener("input", syncGrid);
  }
  $("#framing-target").addEventListener("input", syncOutputGeometry);
  $("#show-grid").addEventListener("change", syncGrid);
  $("#show-center").addEventListener("change", syncGrid);
  $("#show-measurement").addEventListener("change", syncGrid);
  $("#reset-measurement").addEventListener("click", resetMeasurement);
  $("#source-image").addEventListener("load", () => {
    $("#source-image").classList.remove("is-loading");
    $("#source-loading").hidden = true;
    positionOverlays();
    syncGeometry();
    status("Full source loaded. Adjust the draft or run edge detection.");
  });
  $("#source-image").addEventListener("error", () => {
    $("#source-loading").hidden = true;
    status("The source image could not be loaded.", "error");
  });
  $("#preview-image").addEventListener("load", () => {
    positionOverlays();
    syncMeasurementOverlay();
    syncOutputGeometry();
  });
  window.addEventListener("resize", positionOverlays);
  const overlay = $("#source-overlay");
  overlay.addEventListener("pointermove", dragCorner);
  overlay.addEventListener("pointerup", endDrag);
  overlay.addEventListener("pointercancel", endDrag);
  const measurementOverlay = $("#measurement-overlay");
  measurementOverlay.addEventListener("pointermove", dragMeasurementCorner);
  measurementOverlay.addEventListener("pointerup", endMeasurementDrag);
  measurementOverlay.addEventListener("pointercancel", endMeasurementDrag);
}

function syncDraftControls() {
  $("#rotation").value = adjustment.rotationDegrees;
  $("#zoom").value = adjustment.zoom;
  $("#pan-x").value = adjustment.panX;
  $("#pan-y").value = adjustment.panY;
  corners().forEach((point, index) => {
    for (const [, edge] of cornerInsets[index]) {
      const value = cornerInsetValue(point, edge);
      $(`[data-corner="${index}"][data-inset="${edge}"]`).value = value.toFixed(4);
    }
  });
  $("#json").value = JSON.stringify(canonicalAdjustment(), null, 2);
}

function syncAll() {
  alignmentSuppression = null;
  framingProposal = null;
  framingPreviewVisible = false;
  framingDismissalSignature = null;
  framingUndoAdjustment = null;
  invalidateManualFramingSupport();
  manualFramingReassessAfterRender = false;
  syncDraftControls();
  syncSourceOverlay();
  syncMeasurementOverlay();
  syncGeometry();
  syncOutputGeometry();
  syncGrid();
  scheduleRender(0);
}

function draftChanged({ preserveFramingUndo = false, reassessManualFraming = false } = {}) {
  alignmentSuppression = null;
  framingProposal = null;
  framingPreviewVisible = false;
  syncFramingOverlay();
  invalidateManualFramingSupport();
  manualFramingReassessAfterRender = reassessManualFraming;
  if (!preserveFramingUndo) framingUndoAdjustment = null;
  syncDraftControls();
  syncSourceOverlay();
  syncGeometry();
  scheduleRender();
}

function syncSourceOverlay() {
  const values = corners();
  $("#source-polygon").setAttribute("points", values.map((p) => `${p.x * 1000},${p.y * 1000}`).join(" "));
  const group = $("#corner-handles");
  if (group.children.length !== 4) {
    group.replaceChildren();
    cornerNames.forEach((name, index) => {
      const handle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
      handle.setAttribute("r", 16);
      handle.setAttribute("tabindex", "0");
      handle.setAttribute("role", "slider");
      handle.setAttribute("aria-label", name);
      handle.dataset.index = index;
      handle.addEventListener("pointerdown", beginDrag);
      handle.addEventListener("keydown", nudgeCorner);
      group.append(handle);
    });
  }
  values.forEach((point, index) => {
    const handle = group.children[index];
    handle.setAttribute("cx", point.x * 1000);
    handle.setAttribute("cy", point.y * 1000);
  });
  positionOverlays();
}

function beginDrag(event) {
  dragIndex = Number(event.currentTarget.dataset.index);
  event.currentTarget.setPointerCapture(event.pointerId);
  event.preventDefault();
}

function dragCorner(event) {
  if (dragIndex === null || !(event.buttons & 1)) return;
  const rect = $("#source-overlay").getBoundingClientRect();
  setCorner(dragIndex, (event.clientX - rect.left) / rect.width, (event.clientY - rect.top) / rect.height);
}

function endDrag() {
  if (dragIndex !== null) log(`Adjusted ${cornerNames[dragIndex]} corner`);
  dragIndex = null;
}

function nudgeCorner(event) {
  const directions = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };
  const direction = directions[event.key];
  if (!direction) return;
  event.preventDefault();
  const index = Number(event.currentTarget.dataset.index);
  const step = event.shiftKey ? .01 : .001;
  const point = corners()[index];
  setCorner(index, point.x + direction[0] * step, point.y + direction[1] * step);
  requestAnimationFrame(() => $(`#corner-handles circle:nth-child(${index + 1})`)?.focus());
}

function setCorner(index, x, y) {
  const values = corners().map((point) => ({ ...point }));
  values[index] = { x: clamp(x), y: clamp(y) };
  adjustment.perspective = { corners: values };
  draftChanged();
}

function measurementStorageKey() {
  return `autographs-output-measurement:${images[selectedId]?.name || selectedId}`;
}

function loadMeasurement() {
  measurementCorners = defaultMeasurementCorners();
  measurementTouched = new Set();
  measurementOrigin = "automatic";
  try {
    const saved = JSON.parse(localStorage.getItem(measurementStorageKey()) || "null");
    if (saved?.origin === "manual" && saved?.corners?.length === 4) {
      measurementCorners = saved.corners;
      measurementTouched = new Set(saved.touched || []);
      measurementOrigin = "manual";
    }
  } catch (_error) {
    // A malformed browser-only measurement never affects the saved adjustment.
  }
}

function saveMeasurement() {
  if (measurementOrigin !== "manual") return;
  try {
    localStorage.setItem(measurementStorageKey(), JSON.stringify({
      corners: measurementCorners,
      touched: [...measurementTouched],
      origin: measurementOrigin,
    }));
  } catch (_error) {
    // Measurement persistence is optional and never blocks image adjustment.
  }
}

function resetMeasurement() {
  try {
    localStorage.removeItem(measurementStorageKey());
  } catch (_error) {
    // Continue with an in-memory reset when browser storage is unavailable.
  }
  measurementCorners = defaultMeasurementCorners();
  measurementTouched = new Set();
  measurementOrigin = "automatic";
  invalidateManualFramingSupport();
  syncOutputGeometry();
  log("Returned output measurement to automatic boundary detection");
}

function syncMeasurementOverlay() {
  const overlay = $("#measurement-overlay");
  overlay.classList.toggle("tentative", measurementOrigin === "automatic" && detectedBoundary.status === "tentative");
  const edgeGroup = $("#measurement-edges");
  const handleGroup = $("#measurement-handles");
  if (edgeGroup.children.length !== 4) {
    edgeGroup.replaceChildren();
    handleGroup.replaceChildren();
    cornerNames.forEach((name, index) => {
      const edge = document.createElementNS("http://www.w3.org/2000/svg", "line");
      edgeGroup.append(edge);
      const handle = document.createElementNS("http://www.w3.org/2000/svg", "circle");
      handle.setAttribute("r", 16);
      handle.setAttribute("tabindex", "0");
      handle.setAttribute("role", "slider");
      handle.setAttribute("aria-label", `Output ${name}`);
      handle.dataset.index = index;
      handle.addEventListener("pointerdown", beginMeasurementDrag);
      handle.addEventListener("keydown", nudgeMeasurementCorner);
      handleGroup.append(handle);
    });
  }
  const deviations = edgeAxisDeviations(measurementCorners, imageAspect("#preview-image"));
  measurementCorners.forEach((point, index) => {
    const next = measurementCorners[(index + 1) % 4];
    const edge = edgeGroup.children[index];
    edge.setAttribute("x1", point.x * 1000);
    edge.setAttribute("y1", point.y * 1000);
    edge.setAttribute("x2", next.x * 1000);
    edge.setAttribute("y2", next.y * 1000);
    edge.setAttribute(
      "class",
      measurementTouched.size < 4 ? "" : deviations[index] <= .5 ? "good" : deviations[index] <= 1.5 ? "warn" : "bad"
    );
    const handle = handleGroup.children[index];
    handle.setAttribute("cx", point.x * 1000);
    handle.setAttribute("cy", point.y * 1000);
  });
  positionOverlays();
}

function beginMeasurementDrag(event) {
  measurementDragIndex = Number(event.currentTarget.dataset.index);
  event.currentTarget.setPointerCapture(event.pointerId);
  event.preventDefault();
}

function dragMeasurementCorner(event) {
  if (measurementDragIndex === null || !(event.buttons & 1)) return;
  const rect = $("#measurement-overlay").getBoundingClientRect();
  setMeasurementCorner(
    measurementDragIndex,
    (event.clientX - rect.left) / rect.width,
    (event.clientY - rect.top) / rect.height
  );
}

function endMeasurementDrag() {
  if (measurementDragIndex !== null) log(`Measured output ${cornerNames[measurementDragIndex]} corner`);
  measurementDragIndex = null;
}

function nudgeMeasurementCorner(event) {
  const directions = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };
  const direction = directions[event.key];
  if (!direction) return;
  event.preventDefault();
  const index = Number(event.currentTarget.dataset.index);
  const step = event.shiftKey ? .01 : .001;
  const point = measurementCorners[index];
  setMeasurementCorner(index, point.x + direction[0] * step, point.y + direction[1] * step);
  requestAnimationFrame(() => $(`#measurement-handles circle:nth-child(${index + 1})`)?.focus());
}

function setMeasurementCorner(index, x, y) {
  measurementCorners[index] = { x: clamp(x), y: clamp(y) };
  measurementTouched.add(index);
  measurementOrigin = "manual";
  invalidateManualFramingSupport();
  saveMeasurement();
  syncMeasurementOverlay();
  syncOutputGeometry();
}

function clamp(value) { return Math.max(0, Math.min(1, value)); }

function positionOverlays() {
  positionOverlay($("#source-frame"), $("#source-image"), $("#source-overlay"));
  positionOverlay($("#preview-frame"), $("#preview-image"), $("#preview-grid"));
  positionOverlay($("#preview-frame"), $("#preview-image"), $("#preview-center"));
  positionOverlay($("#preview-frame"), $("#preview-image"), $("#measurement-overlay"));
  positionOverlay($("#preview-frame"), $("#preview-image"), $("#framing-overlay"));
}

function positionOverlay(frame, image, overlay) {
  const frameRect = frame.getBoundingClientRect();
  const sourceWidth = Number(image.naturalWidth);
  const sourceHeight = Number(image.naturalHeight);
  if (!frameRect.width || !frameRect.height || !sourceWidth || !sourceHeight) return;
  const scale = Math.min(frameRect.width / sourceWidth, frameRect.height / sourceHeight);
  const width = sourceWidth * scale;
  const height = sourceHeight * scale;
  Object.assign(overlay.style, {
    left: `${(frameRect.width - width) / 2}px`, top: `${(frameRect.height - height) / 2}px`,
    width: `${width}px`, height: `${height}px`, right: "auto", bottom: "auto",
  });
}

function scheduleRender(delay = 140) {
  clearTimeout(renderTimer);
  renderTimer = setTimeout(renderPreview, delay);
}

function publishPreviewResult(result) {
  if (previewUrl) URL.revokeObjectURL(previewUrl);
  previewUrl = URL.createObjectURL(result.blob);
  detectedBoundary = result.boundary;
  $("#preview-image").src = previewUrl;
  syncOutputGeometry();
  if (manualFramingReassessAfterRender) {
    manualFramingReassessAfterRender = false;
    void assessManualFraming();
  }
}

async function renderPreview({ publish = true } = {}) {
  const generation = ++renderGeneration;
  if (publish) {
    $("#rendering").hidden = false;
    $("#output-geometry").innerHTML = '<p class="analysis-pending">Analyzing rendered boundary…</p>';
  }
  try {
    const response = await api(`/api/images/${selectedId}/render`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify(canonicalAdjustment()),
    });
    const boundaryStatus = response.headers.get("x-workbench-boundary-status") || "error";
    let boundaryCorners = [];
    let boundaryEdges = [];
    let framingSupport = null;
    try {
      boundaryCorners = JSON.parse(response.headers.get("x-workbench-boundary-corners") || "[]");
      boundaryEdges = JSON.parse(response.headers.get("x-workbench-boundary-edges") || "[]");
      framingSupport = JSON.parse(response.headers.get("x-workbench-framing-support") || "null");
    } catch (_error) {
      boundaryCorners = [];
      boundaryEdges = [];
      framingSupport = null;
    }
    const blob = await response.blob();
    if (generation !== renderGeneration) return false;
    const result = {
      blob,
      boundary: { status: boundaryStatus, corners: boundaryCorners, edges: boundaryEdges, framingSupport },
    };
    detectedBoundary = result.boundary;
    if (publish) {
      publishPreviewResult(result);
    } else {
      alignmentSuggestion = nextAlignmentSuggestion(
        result.boundary.corners,
        imageAspect("#preview-image"),
        result.boundary.edges,
      );
    }
    return result;
  } catch (error) {
    if (generation === renderGeneration) status(error.message, "error");
    return false;
  } finally {
    if (publish && generation === renderGeneration) $("#rendering").hidden = true;
  }
}

async function detect() {
  status("Running the production edge detector…");
  try {
    const proposal = await (await api(`/api/images/${selectedId}/detect`, { method: "POST" })).json();
    if (proposal.status === "confident" && proposal.corners?.length === 4) {
      adjustment.perspective = { corners: proposal.corners };
      syncAll();
      status("Confident proposal applied to the draft. Review before saving.", "success");
    } else {
      status(proposal.message || `Detector result: ${proposal.status}`);
    }
    log(`Detector: ${proposal.status}`);
  } catch (error) { status(error.message, "error"); }
}

async function saveFixture() {
  try {
    adjustment = await (await api(`/api/images/${selectedId}/fixture`, {
      method: "PUT", headers: { "content-type": "application/json" },
      body: JSON.stringify(canonicalAdjustment()),
    })).json();
    images[selectedId].hasFixture = true;
    $("#image-select").options[selectedId].textContent = `${images[selectedId].name} • fixture`;
    syncAll();
    status("Fixture saved outside Git. Original image was not modified.", "success");
    log("Saved fixture");
  } catch (error) { status(error.message, "error"); }
}

function applyJson() {
  try {
    const parsed = JSON.parse($("#json").value);
    adjustment = parsed;
    syncAll();
    status("JSON applied to the draft.", "success");
    log("Applied JSON draft");
  } catch (error) { status(`Invalid JSON: ${error.message}`, "error"); }
}

function syncGrid() {
  const spacing = Number($("#grid-spacing").value);
  const x = Number($("#grid-x").value);
  const y = Number($("#grid-y").value);
  const grid = $("#preview-grid");
  grid.hidden = !$("#show-grid").checked;
  grid.style.backgroundSize = `${spacing}px ${spacing}px`;
  grid.style.backgroundPosition = `${x}px ${y}px`;
  $("#preview-center").hidden = !$("#show-center").checked;
  $("#measurement-overlay").toggleAttribute(
    "hidden",
    !$("#show-measurement").checked,
  );
  $("#grid-spacing-value").value = `${spacing} px`;
  $("#grid-x-value").value = `${x} px`;
  $("#grid-y-value").value = `${y} px`;
}

function syncGeometry() {
  const metrics = geometryMetrics(corners(), imageAspect("#source-image"));
  metrics.push(["Rotation applied after perspective", adjustment.rotationDegrees, "°"]);
  renderGeometryMetrics($("#source-geometry"), metrics, false);
}

function imageAspect(selector) {
  const image = $(selector);
  return image?.naturalWidth && image?.naturalHeight ? image.naturalWidth / image.naturalHeight : 1;
}

function edgeAxisOffsets(points, aspectRatio = 1) {
  return points.map((point, index) => {
    const next = points[(index + 1) % points.length];
    const angle = Math.atan2(next.y - point.y, (next.x - point.x) * aspectRatio) * 180 / Math.PI;
    const target = index % 2 === 0
      ? Math.round(angle / 180) * 180
      : Math.round((angle - 90) / 180) * 180 + 90;
    return angle - target;
  });
}

function edgeAxisDeviations(points, aspectRatio = 1) {
  return edgeAxisOffsets(points, aspectRatio).map(Math.abs);
}

function geometryMetrics(points, aspectRatio = 1) {
  const scaledPoints = points.map((point) => ({ x: point.x * aspectRatio, y: point.y }));
  const edges = scaledPoints.map((point, index) => {
    const next = scaledPoints[(index + 1) % scaledPoints.length];
    return { x: next.x - point.x, y: next.y - point.y };
  });
  const deviations = edgeAxisDeviations(points, aspectRatio);
  const angles = scaledPoints.map((_, index) => {
    const before = edges[(index + 3) % 4];
    const after = edges[index];
    const dot = (-before.x * after.x) + (-before.y * after.y);
    const lengths = Math.hypot(before.x, before.y) * Math.hypot(after.x, after.y);
    return Math.acos(Math.max(-1, Math.min(1, dot / lengths))) * 180 / Math.PI;
  });
  const cornerDeviations = angles.map((angle) => Math.abs(90 - angle));
  const worstCornerIndex = cornerDeviations.reduce(
    (worst, value, index) => value > cornerDeviations[worst] ? index : worst,
    0,
  );
  const widthTop = Math.hypot(edges[0].x, edges[0].y);
  const widthBottom = Math.hypot(edges[2].x, edges[2].y);
  const heightRight = Math.hypot(edges[1].x, edges[1].y);
  const heightLeft = Math.hypot(edges[3].x, edges[3].y);
  return [
    ["Worst edge axis deviation", Math.max(...deviations), "°", 0.5, 1.5],
    [`Worst corner from square (${cornerNames[worstCornerIndex]})`, cornerDeviations[worstCornerIndex], "°", 0.5, 1.5],
    ["Top / bottom length delta", Math.abs(widthTop - widthBottom) / ((widthTop + widthBottom) / 2) * 100, "%", 0.5, 2],
    ["Left / right length delta", Math.abs(heightLeft - heightRight) / ((heightLeft + heightRight) / 2) * 100, "%", 0.5, 2],
  ];
}

function framingSplit(first, second) {
  const total = first + second;
  if (total < .01) return { first: 50, second: 50, deviation: 0, measurable: false };
  const firstShare = first / total * 100;
  return {
    first: firstShare,
    second: 100 - firstShare,
    deviation: Math.abs(firstShare - 50),
    measurable: true,
  };
}

function previewRelativeFraming(points) {
  const margins = {
    top: ((points[0].y + points[1].y) / 2) * 100,
    right: (1 - ((points[1].x + points[2].x) / 2)) * 100,
    bottom: (1 - ((points[2].y + points[3].y) / 2)) * 100,
    left: ((points[3].x + points[0].x) / 2) * 100,
  };
  return {
    margins,
    vertical: framingSplit(margins.top, margins.bottom),
    horizontal: framingSplit(margins.left, margins.right),
  };
}

function sourceSpaceFraming(support) {
  if (!support || support.itemShortEdgePixels <= 0) return null;
  const scale = 100 / support.itemShortEdgePixels;
  const margins = Object.fromEntries(
    Object.entries(support.marginsPixels).map(([side, pixels]) => [side, pixels * scale])
  );
  return {
    margins,
    vertical: framingSplit(margins.top, margins.bottom),
    horizontal: framingSplit(margins.left, margins.right),
    maximum: support.maxSymmetricMarginPixels * scale,
    itemShortEdgePixels: support.itemShortEdgePixels,
    itemIntersectsInvalidPixels: support.itemIntersectsInvalidPixels,
    limitedByExistingCrop: support.limitedByExistingCrop,
  };
}

function framingBalanceClass(deviation) {
  return deviation <= 5 ? "good" : deviation <= 10 ? "warn" : "bad";
}

function framingSplitLabel(firstName, secondName, split) {
  if (!split.measurable) return `${firstName} / ${secondName} allocation unavailable`;
  return `${firstName} / ${secondName} allocation (${split.first.toFixed(0)} / ${split.second.toFixed(0)})`;
}

function framingAssessment(framing, target) {
  const minimumUsefulBand = 3;
  const margins = Object.values(framing.margins);
  const withinTrialRange = margins.every((value) => value >= minimumUsefulBand && value <= 6);
  const centered = framing.vertical.deviation <= 5 && framing.horizontal.deviation <= 5;
  if (framing.itemIntersectsInvalidPixels) {
    return {
      status: "Retake recommended",
      className: "bad",
      message: "The detected item intersects pixels created outside the photographed source. Cropping cannot reconstruct a complete real-background band.",
    };
  }
  if (withinTrialRange && centered) {
    return {
      status: "Balanced",
      className: "good",
      message: "Framing is within the current 3–6% trial range and opposite sides are balanced. No crop is suggested.",
    };
  }
  if (framing.maximum + .05 >= target) {
    return {
      status: "Adjustable",
      className: "warn",
      message: `A ${target.toFixed(1)}% symmetric background band is available using real pixels. A later preview can crop excess background without inventing any.`,
    };
  }
  if (framing.limitedByExistingCrop) {
    return {
      status: "Constrained",
      className: "warn",
      message: `The current crop, zoom, or pan leaves at most a ${framing.maximum.toFixed(1)}% symmetric real-pixel band. Reset those controls before deciding whether the original needs a retake.`,
    };
  }
  const noiseGuard = Math.max(1, framing.itemShortEdgePixels * .005) /
    framing.itemShortEdgePixels * 100;
  if (framing.maximum > noiseGuard) {
    const materiallyOffCenter = framing.vertical.deviation > 5 || framing.horizontal.deviation > 5;
    return {
      status: "Constrained",
      className: "warn",
      message: framing.maximum < minimumUsefulBand
        ? materiallyOffCenter
          ? `The ${target.toFixed(1)}% target is not available, and the best symmetric real-pixel band is only ${framing.maximum.toFixed(1)}%. The existing thin border can still be recentered by trimming only the excess background.`
          : `The ${target.toFixed(1)}% target is not available, but the ${framing.maximum.toFixed(1)}% symmetric real-pixel band is already centered. Preserve the available background.`
        : `The ${target.toFixed(1)}% target is not available. The best symmetric band supported by real pixels is ${framing.maximum.toFixed(1)}%.`,
    };
  }
  return {
    status: "Retake recommended",
    className: "bad",
    message: "At least one side has effectively no photographed background. Cropping cannot add real background on that side.",
  };
}

function suggestedFramingCrop(points, framing, bandPercent) {
  const image = $("#preview-image");
  if (!image.naturalWidth || !image.naturalHeight || points.length !== 4) return null;
  const xScale = Math.max(1, image.naturalWidth - 1);
  const yScale = Math.max(1, image.naturalHeight - 1);
  const bandPixels = framing.itemShortEdgePixels * bandPercent / 100;
  const left = clamp(Math.min(...points.map((point) => point.x)) - bandPixels / xScale);
  const right = clamp(Math.max(...points.map((point) => point.x)) + bandPixels / xScale);
  const top = clamp(Math.min(...points.map((point) => point.y)) - bandPixels / yScale);
  const bottom = clamp(Math.max(...points.map((point) => point.y)) + bandPixels / yScale);
  if (right - left < .01 || bottom - top < .01) return null;
  const crop = { left, top, right, bottom };
  const signature = JSON.stringify({
    image: selectedId,
    adjustment: canonicalAdjustment(),
    boundary: points,
    bandPercent: Number(bandPercent.toFixed(3)),
    crop,
  });
  return { crop, bandPercent, signature };
}

function syncFramingOverlay() {
  const overlay = $("#framing-overlay");
  const proposal = framingProposal;
  const visible = framingPreviewVisible && proposal &&
    framingDismissalSignature !== proposal.signature;
  overlay.toggleAttribute("hidden", !visible);
  if (!visible) return;
  const left = proposal.crop.left * 1000;
  const top = proposal.crop.top * 1000;
  const right = proposal.crop.right * 1000;
  const bottom = proposal.crop.bottom * 1000;
  const width = right - left;
  const height = bottom - top;
  $("#framing-dim").setAttribute(
    "d",
    `M0 0H1000V1000H0Z M${left} ${top}H${right}V${bottom}H${left}Z`,
  );
  const crop = $("#framing-crop");
  crop.setAttribute("x", left);
  crop.setAttribute("y", top);
  crop.setAttribute("width", width);
  crop.setAttribute("height", height);
  positionOverlays();
}

function appendFramingButton(host, label, className, action) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = className;
  button.textContent = label;
  button.addEventListener("click", action);
  host.append(button);
  return button;
}

async function assessManualFraming() {
  if (measurementOrigin !== "manual" || measurementTouched.size < 4) return;
  const signature = manualFramingKey();
  const button = $("#assess-manual-framing");
  if (button) {
    button.disabled = true;
    button.textContent = "Assessing manual boundary…";
  }
  try {
    const response = await api(`/api/images/${selectedId}/framing`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        adjustment: canonicalAdjustment(),
        corners: measurementCorners,
      }),
    });
    if (signature !== manualFramingKey()) return;
    manualFramingSupport = await response.json();
    manualFramingSignature = signature;
    syncOutputGeometry();
    status("Background framing assessed from the confirmed manual boundary.", "success");
    log("Confirmed manual output boundary for background framing");
  } catch (error) {
    status(error.message, "error");
    syncOutputGeometry();
  }
}

function applySuggestedFraming() {
  if (!framingProposal) return;
  const preserveManualBoundary = measurementOrigin === "manual" &&
    manualFramingSignature === manualFramingKey();
  framingUndoAdjustment = structuredClone(adjustment);
  const crop = { ...framingProposal.crop };
  adjustment.crop = crop;
  adjustment.zoom = 1;
  adjustment.panX = 0;
  adjustment.panY = 0;
  const appliedBand = framingProposal.bandPercent;
  framingProposal = null;
  framingPreviewVisible = false;
  framingDismissalSignature = null;
  if (preserveManualBoundary) {
    const width = crop.right - crop.left;
    const height = crop.bottom - crop.top;
    measurementCorners = measurementCorners.map((point) => ({
      x: clamp((point.x - crop.left) / width),
      y: clamp((point.y - crop.top) / height),
    }));
    saveMeasurement();
  }
  syncFramingOverlay();
  log(`Applied ${appliedBand.toFixed(1)}% framing crop to the unsaved draft`);
  status("Suggested framing applied to the draft. Review the rerender before saving.", "success");
  draftChanged({
    preserveFramingUndo: true,
    reassessManualFraming: preserveManualBoundary,
  });
}

function undoSuggestedFraming() {
  if (!framingUndoAdjustment) return;
  adjustment = framingUndoAdjustment;
  framingUndoAdjustment = null;
  framingProposal = null;
  framingPreviewVisible = false;
  framingDismissalSignature = null;
  syncFramingOverlay();
  log("Undid the framing crop");
  status("Framing crop undone; the prior draft was restored.", "success");
  draftChanged({ preserveFramingUndo: true });
}

function syncFramingActions(assessment, framing, target, points) {
  const host = $("#framing-actions");
  host.replaceChildren();
  framingProposal = null;
  if (framingUndoAdjustment) {
    appendFramingButton(host, "Undo framing", "quiet", undoSuggestedFraming);
  }
  const onePixelPercent = 100 / framing.itemShortEdgePixels;
  const proposalBand = assessment.status === "Adjustable"
    ? target
    : assessment.status === "Constrained" && !framing.limitedByExistingCrop &&
        (framing.vertical.deviation > 5 || framing.horizontal.deviation > 5)
      ? Math.max(0, framing.maximum - onePixelPercent)
      : 0;
  if (proposalBand <= .5 || framing.itemIntersectsInvalidPixels) {
    framingPreviewVisible = false;
    syncFramingOverlay();
    return;
  }
  const proposal = suggestedFramingCrop(points, framing, proposalBand);
  if (!proposal || proposal.signature === framingDismissalSignature) {
    framingPreviewVisible = false;
    syncFramingOverlay();
    return;
  }
  framingProposal = proposal;
  const isRecentering = assessment.status === "Constrained";
  const previewLabel = framingPreviewVisible
    ? isRecentering ? "Hide recenter preview" : "Hide crop preview"
    : isRecentering ? "Preview recentering" : "Preview crop";
  appendFramingButton(host, previewLabel, "quiet", () => {
    framingPreviewVisible = !framingPreviewVisible;
    syncFramingOverlay();
    syncFramingActions(assessment, framing, target, points);
  });
  appendFramingButton(
    host,
    isRecentering ? "Apply recentering" : "Apply to draft",
    "accent",
    applySuggestedFraming,
  );
  appendFramingButton(host, "Dismiss", "quiet", () => {
    framingDismissalSignature = proposal.signature;
    framingPreviewVisible = false;
    framingProposal = null;
    syncFramingOverlay();
    syncFramingActions(assessment, framing, target, points);
  });
  syncFramingOverlay();
}

function syncFramingGeometry(points, reliabilityMessage = "") {
  const host = $("#framing-geometry");
  const summary = $("#framing-summary");
  const actions = $("#framing-actions");
  host.replaceChildren();
  actions.replaceChildren();
  summary.className = "framing-summary";
  if (points.length !== 4) {
    framingProposal = null;
    framingPreviewVisible = false;
    syncFramingOverlay();
    host.innerHTML = '<p class="analysis-pending">No item boundary is available for background measurement.</p>';
    summary.textContent = "";
    return;
  }

  const automaticSupport = measurementOrigin === "automatic"
    ? sourceSpaceFraming(detectedBoundary.framingSupport)
    : null;
  const confirmedManualSupport = measurementOrigin === "manual" &&
    manualFramingSignature === manualFramingKey()
    ? sourceSpaceFraming(manualFramingSupport)
    : null;
  const sourceSupport = automaticSupport || confirmedManualSupport;
  const framing = sourceSupport || previewRelativeFraming(points);
  const metricSuffix = sourceSupport ? "% of item short edge" : "% of preview";
  const marginLabel = sourceSupport ? "real-pixel margin" : "preview margin";
  renderGeometryMetrics(host, [
    [`Top ${marginLabel}`, framing.margins.top, metricSuffix, null, null, false, false, 1],
    [`Right ${marginLabel}`, framing.margins.right, metricSuffix, null, null, false, false, 1],
    [`Bottom ${marginLabel}`, framing.margins.bottom, metricSuffix, null, null, false, false, 1],
    [`Left ${marginLabel}`, framing.margins.left, metricSuffix, null, null, false, false, 1],
  ], false);

  if (sourceSupport) {
    renderGeometryMetric(
      host,
      "Maximum symmetric real-pixel band",
      sourceSupport.maximum,
      "% of item short edge",
      "info",
      1,
    );
  }

  for (const [label, split] of [
    [framingSplitLabel("Top", "bottom", framing.vertical), framing.vertical],
    [framingSplitLabel("Left", "right", framing.horizontal), framing.horizontal],
  ]) {
    const row = document.createElement("div");
    row.className = `metric ${split.measurable ? framingBalanceClass(split.deviation) : "info"}`;
    row.innerHTML = `<span>${label}</span><strong>${split.measurable ? `${split.deviation.toFixed(1)} pp off center` : "—"}</strong>`;
    host.append(row);
  }

  if (reliabilityMessage) {
    framingProposal = null;
    framingPreviewVisible = false;
    syncFramingOverlay();
    summary.classList.add("warn");
    summary.textContent = `${reliabilityMessage} The values remain visible, but this boundary must not drive a framing suggestion.`;
    return;
  }

  if (!sourceSupport) {
    framingProposal = null;
    framingPreviewVisible = false;
    syncFramingOverlay();
    summary.classList.add("warn");
    summary.textContent = "Manual boundaries show preview-relative balance until you confirm that the four handles mark the item corners. Confirmation measures real photographed-pixel support without changing the image adjustment.";
    if (framingUndoAdjustment) {
      appendFramingButton(actions, "Undo framing", "quiet", undoSuggestedFraming);
    }
    const button = appendFramingButton(
      actions,
      "Assess using manual boundary",
      "quiet",
      assessManualFraming,
    );
    button.id = "assess-manual-framing";
    return;
  }

  if (!framing.vertical.measurable || !framing.horizontal.measurable) {
    framingProposal = null;
    framingPreviewVisible = false;
    syncFramingOverlay();
    summary.classList.add("bad");
    summary.textContent = "At least one axis has no measurable background on either side of the detected item. Balance cannot be assessed on that axis.";
    return;
  }
  const target = Math.max(1, Math.min(12, Number($("#framing-target").value) || 4));
  const assessment = framingAssessment(framing, target);
  summary.classList.add(assessment.className);
  const boundarySource = confirmedManualSupport ? " Confirmed manual boundary." : "";
  summary.innerHTML = `<strong>${assessment.status}</strong> — ${assessment.message}${boundarySource} This assessment does not change crop, zoom, or pan.`;
  syncFramingActions(assessment, framing, target, points);
}

function renderGeometryMetric(host, label, value, unit, className = "info", precision = 1) {
  const row = document.createElement("div");
  row.className = `metric ${className}`;
  row.innerHTML = `<span>${label}</span><strong>${value.toFixed(precision)} ${unit}</strong>`;
  host.append(row);
}

function outputGeometryMetrics(points, aspectRatio = 1, fittedEdges = []) {
  const offsets = fittedEdges.length === 4
    ? fittedEdges.map((edge) => edge.axisDeviationDegrees)
    : edgeAxisOffsets(points, aspectRatio);
  const base = geometryMetrics(points, aspectRatio);
  const metrics = [
    ["Top edge vs horizontal", offsets[0], "°", 0.5, 1.5, true],
    ["Right edge vs vertical", offsets[1], "°", 0.5, 1.5, true],
    ["Bottom edge vs horizontal", offsets[2], "°", 0.5, 1.5, true],
    ["Left edge vs vertical", offsets[3], "°", 0.5, 1.5, true],
    ["Horizontal-edge parallelism", Math.abs(offsets[0] - offsets[2]), "°", 0.5, 1.5],
    ["Vertical-edge parallelism", Math.abs(offsets[1] - offsets[3]), "°", 0.5, 1.5],
    base[1],
  ];
  if (fittedEdges.length === 4) {
    const worstStraightnessIndex = fittedEdges.reduce(
      (worst, edge, index) => edge.straightnessRms > fittedEdges[worst].straightnessRms ? index : worst,
      0,
    );
    const lowestCoverageIndex = fittedEdges.reduce(
      (lowest, edge, index) => edge.coverage < fittedEdges[lowest].coverage ? index : lowest,
      0,
    );
    metrics.push(
      [`Worst edge straightness residual (${edgeNames[worstStraightnessIndex]})`, fittedEdges[worstStraightnessIndex].straightnessRms * 100, "%", 0.10, 0.25, false, false, 3],
      ["Worst center bow deviation", Math.max(...fittedEdges.map((edge) => edge.bowDeviation)) * 100, "%", 0.10, 0.30, false, false, 3],
      [`Lowest edge-fit coverage (${edgeNames[lowestCoverageIndex]})`, fittedEdges[lowestCoverageIndex].coverage * 100, "%", 70, 40, false, true, 0],
    );
  }
  return metrics;
}

function fittedEdgesAreStable(fittedEdges) {
  return fittedEdges.length === 4 &&
    Math.min(...fittedEdges.map((edge) => edge.coverage)) >= .7 &&
    Math.max(...fittedEdges.map((edge) => edge.straightnessRms)) <= .004;
}

function rotatePoints(points, degrees) {
  const radians = degrees * Math.PI / 180;
  const cosine = Math.cos(radians);
  const sine = Math.sin(radians);
  const center = {
    x: points.reduce((sum, point) => sum + point.x, 0) / points.length,
    y: points.reduce((sum, point) => sum + point.y, 0) / points.length,
  };
  return points.map((point) => {
    const x = point.x - center.x;
    const y = point.y - center.y;
    return {
      x: center.x + cosine * x - sine * y,
      y: center.y + sine * x + cosine * y,
    };
  });
}

function suggestedPerspectiveCorners(points, aspectRatio, commonTilt) {
  const scaled = points.map((point) => ({ x: point.x * aspectRatio, y: point.y }));
  const untilted = rotatePoints(scaled, -commonTilt);
  const left = (untilted[0].x + untilted[3].x) / 2;
  const right = (untilted[1].x + untilted[2].x) / 2;
  const top = (untilted[0].y + untilted[1].y) / 2;
  const bottom = (untilted[2].y + untilted[3].y) / 2;
  const squared = [
    { x: left, y: top }, { x: right, y: top },
    { x: right, y: bottom }, { x: left, y: bottom },
  ];
  const desired = rotatePoints(squared, commonTilt);
  const source = corners();
  const sourceWidth = (
    Math.abs(source[1].x - source[0].x) + Math.abs(source[2].x - source[3].x)
  ) / 2;
  const sourceHeight = (
    Math.abs(source[3].y - source[0].y) + Math.abs(source[2].y - source[1].y)
  ) / 2;
  const cautiousStep = .75;
  return source.map((point, index) => ({
    x: clamp(point.x - ((desired[index].x - scaled[index].x) / aspectRatio) * sourceWidth * cautiousStep),
    y: clamp(point.y - (desired[index].y - scaled[index].y) * sourceHeight * cautiousStep),
  }));
}

function nextAlignmentSuggestion(points, aspectRatio, fittedEdges) {
  if (detectedBoundary.status !== "reliable" || points.length !== 4) {
    return null;
  }
  const hasFineEdges = fittedEdgesAreStable(fittedEdges);

  const offsets = hasFineEdges
    ? fittedEdges.map((edge) => edge.axisDeviationDegrees)
    : edgeAxisOffsets(points, aspectRatio);
  const commonTilt = offsets.reduce((sum, value) => sum + value, 0) / offsets.length;
  const parallelism = Math.max(
    Math.abs(offsets[0] - offsets[2]),
    Math.abs(offsets[1] - offsets[3]),
  );
  const worstAxis = Math.max(...offsets.map(Math.abs));
  const rotationThreshold = hasFineEdges ? .15 : .25;
  const parallelismThreshold = hasFineEdges ? .15 : .5;
  const axisThreshold = hasFineEdges ? .25 : .5;
  if (parallelism >= parallelismThreshold) {
    return {
      kind: "perspective",
      measurement: hasFineEdges ? "fine" : "coarse",
      corners: suggestedPerspectiveCorners(points, aspectRatio, commonTilt),
    };
  }
  if (Math.abs(commonTilt) >= rotationThreshold) {
    const boundedRotation = Math.max(-15, Math.min(15, adjustment.rotationDegrees - commonTilt));
    return {
      kind: "rotation",
      measurement: hasFineEdges ? "fine" : "coarse",
      rotationDegrees: Math.round(boundedRotation * 20) / 20,
    };
  }
  if (parallelism >= parallelismThreshold || worstAxis >= axisThreshold) {
    return {
      kind: "perspective",
      measurement: hasFineEdges ? "fine" : "coarse",
      corners: suggestedPerspectiveCorners(points, aspectRatio, commonTilt),
    };
  }
  return null;
}

function applyAlignmentSuggestion(suggestion) {
  if (suggestion.kind === "rotation") {
    adjustment.rotationDegrees = suggestion.rotationDegrees;
  } else {
    adjustment.perspective = {
      corners: suggestion.corners.map((point) => ({ ...point })),
    };
  }
}

function alignmentScore(boundary, aspectRatio, kind, measurement = "coarse") {
  if (measurement === "fine" && !fittedEdgesAreStable(boundary.edges)) {
    return Number.POSITIVE_INFINITY;
  }
  const offsets = measurement === "fine"
    ? boundary.edges.map((edge) => edge.axisDeviationDegrees)
    : boundary.corners.length === 4
      ? edgeAxisOffsets(boundary.corners, aspectRatio)
      : [];
  if (offsets.length !== 4) return Number.POSITIVE_INFINITY;
  const commonTilt = offsets.reduce((sum, value) => sum + value, 0) / offsets.length;
  if (kind === "rotation") {
    return Math.sqrt(offsets.reduce((sum, value) => sum + value * value, 0) / offsets.length);
  }
  const worstParallelism = Math.max(
    Math.abs(offsets[0] - offsets[2]),
    Math.abs(offsets[1] - offsets[3]),
  );
  const worstResidualTilt = Math.max(...offsets.map((value) => Math.abs(value - commonTilt)));
  return worstParallelism + worstResidualTilt;
}

async function applySuggestedAlignment() {
  if (!alignmentSuggestion || alignmentApplying) return;
  alignmentApplying = true;
  alignmentSuppression = null;
  framingProposal = null;
  framingPreviewVisible = false;
  framingUndoAdjustment = null;
  syncFramingOverlay();
  clearTimeout(renderTimer);
  $("#rendering").hidden = false;
  syncOutputGeometry();
  const seen = new Set([JSON.stringify(canonicalAdjustment())]);
  let acceptedPasses = 0;
  let stopReason = "iteration limit reached";
  let suppressFinalSuggestion = true;
  let finalResult = null;
  let lastAcceptedAdjustment = structuredClone(adjustment);
  const aspectRatio = imageAspect("#preview-image");

  try {
    for (let pass = 1; pass <= 8; pass += 1) {
      const suggestion = alignmentSuggestion;
      if (!suggestion) {
        stopReason = "alignment is within tolerance";
        suppressFinalSuggestion = false;
        break;
      }
      const previousAdjustment = structuredClone(adjustment);
      const previousScore = alignmentScore(detectedBoundary, aspectRatio, suggestion.kind, suggestion.measurement);
      applyAlignmentSuggestion(suggestion);
      const signature = JSON.stringify(canonicalAdjustment());
      if (seen.has(signature)) {
        adjustment = previousAdjustment;
        stopReason = "a repeated adjustment was avoided";
        break;
      }
      seen.add(signature);
      status(`Auto-aligning preview… pass ${pass} of 8`);
      alignmentSuggestion = null;
      let rendered = await renderPreview({ publish: false });
      if (!rendered || detectedBoundary.status !== "reliable") {
        adjustment = previousAdjustment;
        finalResult = await renderPreview({ publish: false });
        stopReason = "boundary detection became unreliable; retained the last reliable result";
        break;
      }
      let nextScore = alignmentScore(detectedBoundary, aspectRatio, suggestion.kind, suggestion.measurement);
      if (nextScore >= previousScore - .005) {
        let backedOff = false;
        const fractions = suggestion.kind === "rotation" ? [.5, 1 / 3, .25] : [.75, .5, 1 / 3, .25, .125];
        if (suggestion.kind === "rotation" || suggestion.kind === "perspective") {
          const fullRotationDelta = suggestion.kind === "rotation"
            ? suggestion.rotationDegrees - previousAdjustment.rotationDegrees
            : 0;
          const previousCorners = previousAdjustment.perspective?.corners || fullCorners();
          for (const fraction of fractions) {
            adjustment = structuredClone(previousAdjustment);
            if (suggestion.kind === "rotation") {
              adjustment.rotationDegrees = Math.round(
                (previousAdjustment.rotationDegrees + fullRotationDelta * fraction) * 20
              ) / 20;
            } else {
              adjustment.perspective = {
                corners: previousCorners.map((point, index) => ({
                  x: point.x + (suggestion.corners[index].x - point.x) * fraction,
                  y: point.y + (suggestion.corners[index].y - point.y) * fraction,
                })),
              };
            }
            const backedOffSignature = JSON.stringify(canonicalAdjustment());
            if (seen.has(backedOffSignature)) continue;
            seen.add(backedOffSignature);
            const candidate = await renderPreview({ publish: false });
            if (!candidate || detectedBoundary.status !== "reliable") continue;
            const candidateScore = alignmentScore(detectedBoundary, aspectRatio, suggestion.kind, suggestion.measurement);
            if (candidateScore < previousScore - .005) {
              rendered = candidate;
              nextScore = candidateScore;
              backedOff = true;
              log(`Backed off ${suggestion.kind} to ${(fraction * 100).toFixed(0)}% of the proposed step`);
              break;
            }
          }
        }
        if (!backedOff) {
          adjustment = previousAdjustment;
          finalResult = await renderPreview({ publish: false });
          stopReason = "the next pass worsened alignment; retained the better result";
          break;
        }
      }
      acceptedPasses += 1;
      log(`Accepted ${suggestion.measurement} ${suggestion.kind} pass: ${previousScore.toFixed(2)}° → ${nextScore.toFixed(2)}°`);
      lastAcceptedAdjustment = structuredClone(adjustment);
      finalResult = rendered;
      if (!alignmentSuggestion) {
        stopReason = "alignment is within tolerance";
        suppressFinalSuggestion = false;
        break;
      }
      if (alignmentSuggestion.kind === suggestion.kind && previousScore - nextScore < .01) {
        stopReason = "further improvement is below measurement precision";
        break;
      }
    }
    syncDraftControls();
    syncSourceOverlay();
    syncGeometry();
    if (suppressFinalSuggestion) {
      alignmentSuppression = {
        adjustment: JSON.stringify(canonicalAdjustment()),
        reason: stopReason,
      };
      alignmentSuggestion = null;
    }
    log(`Auto-alignment completed after ${acceptedPasses} improving pass${acceptedPasses === 1 ? "" : "es"}`);
    status(`Auto-alignment complete: ${stopReason}.`, "success");
  } catch (error) {
    adjustment = lastAcceptedAdjustment;
    status(`Auto-alignment stopped: ${error.message}`, "error");
  } finally {
    alignmentApplying = false;
    $("#rendering").hidden = true;
    if (finalResult) {
      publishPreviewResult(finalResult);
    } else {
      syncOutputGeometry();
    }
  }
}

function appendSuggestionAction(host) {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "apply-suggestion accent";
  button.disabled = alignmentApplying;
  button.textContent = alignmentApplying ? "Auto-aligning…" : "Auto-align to tolerance";
  button.addEventListener("click", applySuggestedAlignment);
  host.append(button);
}

function syncAlignmentAdvice(points, aspectRatio, fittedEdges) {
  const host = $("#alignment-advice");
  alignmentSuggestion = null;
  host.className = "alignment-advice";
  host.replaceChildren();
  const heading = document.createElement("h4");
  heading.textContent = "Suggested next experiment";
  host.append(heading);

  const adjustmentSignature = JSON.stringify(canonicalAdjustment());
  if (alignmentSuppression?.adjustment === adjustmentSignature) {
    host.classList.add("warn");
    const message = document.createElement("p");
    message.textContent = `Automatic alignment stopped here: ${alignmentSuppression.reason}. The current adjustment was retained, and the rejected proposal is no longer actionable. Change a source control to request a fresh assessment.`;
    host.append(message);
    return;
  }
  if (alignmentSuppression) alignmentSuppression = null;

  const automaticReliable = measurementOrigin === "automatic" &&
    detectedBoundary.status === "reliable" && points.length === 4;
  if (!automaticReliable) {
    const message = document.createElement("p");
    message.textContent = "Exact control suggestions require a reliable automatic boundary. Manual handles remain useful for visual assessment, but they are not used to prescribe source edits.";
    host.append(message);
    return;
  }

  const fineEdgesPresent = fittedEdges.length === 4;
  const hasFineEdges = fittedEdgesAreStable(fittedEdges);
  const offsets = hasFineEdges
    ? fittedEdges.map((edge) => edge.axisDeviationDegrees)
    : edgeAxisOffsets(points, aspectRatio);
  const commonTilt = offsets.reduce((sum, value) => sum + value, 0) / offsets.length;
  const parallelism = Math.max(
    Math.abs(offsets[0] - offsets[2]),
    Math.abs(offsets[1] - offsets[3]),
  );
  const worstAxis = Math.max(...offsets.map(Math.abs));
  const lowestCoverage = fineEdgesPresent
    ? Math.min(...fittedEdges.map((edge) => edge.coverage))
    : null;

  const rotationThreshold = hasFineEdges ? .15 : .25;
  const parallelismThreshold = hasFineEdges ? .15 : .5;
  const axisThreshold = hasFineEdges ? .25 : .5;
  if (fineEdgesPresent && !hasFineEdges) {
    const notice = document.createElement("p");
    const weakEdge = fittedEdges.reduce(
      (lowest, edge, index) => edge.coverage < fittedEdges[lowest].coverage ? index : lowest,
      0,
    );
    notice.textContent = `${edgeNames[weakEdge]} edge refinement is unstable (${(lowestCoverage * 100).toFixed(0)}% coverage). Suggestions use the reliable coarse boundary instead.`;
    host.append(notice);
  }

  if (parallelism >= parallelismThreshold) {
    const proposal = suggestedPerspectiveCorners(points, aspectRatio, commonTilt);
    alignmentSuggestion = {
      kind: "perspective",
      measurement: hasFineEdges ? "fine" : "coarse",
      corners: proposal.map((point) => ({ ...point })),
    };
    host.classList.add("warn");
    const message = document.createElement("p");
    message.textContent = hasFineEdges
      ? "Opposing edges disagree more than they share a rigid tilt. These are the next conservative source-corner values; auto-align will retain them only if parallelism improves."
      : "Fine edge fitting is unavailable or unstable. These coarse-boundary values make a broad perspective pass before rotation is reconsidered.";
    host.append(message);
    const values = document.createElement("div");
    values.className = "suggestion-values";
    proposal.forEach((point, index) => {
      const line = document.createElement("code");
      line.textContent = `${cornerNames[index]}: ${cornerInsetSummary(point, index)}`;
      values.append(line);
    });
    host.append(values);
    appendSuggestionAction(host);
    return;
  }

  if (Math.abs(commonTilt) >= rotationThreshold) {
    const boundedRotation = Math.max(-15, Math.min(15, adjustment.rotationDegrees - commonTilt));
    const proposed = Math.round(boundedRotation * 20) / 20;
    const delta = proposed - adjustment.rotationDegrees;
    alignmentSuggestion = {
      kind: "rotation",
      measurement: hasFineEdges ? "fine" : "coarse",
      rotationDegrees: proposed,
    };
    host.classList.add("warn");
    const message = document.createElement("p");
    const precision = hasFineEdges ? "refined edge fits" : "the reliable coarse boundary";
    message.innerHTML = `Based on ${precision}, the four edges share a ${commonTilt > 0 ? "clockwise" : "counter-clockwise"} tilt. The next pass would use <strong>Rotation ${proposed.toFixed(2)}°</strong> (${delta >= 0 ? "+" : ""}${delta.toFixed(2)}°); auto-align will reassess before changing vertices.`;
    host.append(message);
    appendSuggestionAction(host);
    return;
  }

  if (parallelism >= parallelismThreshold || worstAxis >= axisThreshold) {
    const proposal = suggestedPerspectiveCorners(points, aspectRatio, commonTilt);
    alignmentSuggestion = {
      kind: "perspective",
      measurement: hasFineEdges ? "fine" : "coarse",
      corners: proposal.map((point) => ({ ...point })),
    };
    host.classList.add("warn");
    const message = document.createElement("p");
    message.textContent = hasFineEdges
      ? "Rigid rotation is already close. These are the next conservative source-corner values; auto-align will retain them only if the refined edge and parallelism scores improve."
      : "Fine edge fitting is not available yet. These coarse-boundary values make a broad perspective pass; auto-align will retry refined measurement after rendering.";
    host.append(message);
    const values = document.createElement("div");
    values.className = "suggestion-values";
    proposal.forEach((point, index) => {
      const line = document.createElement("code");
      line.textContent = `${cornerNames[index]}: ${cornerInsetSummary(point, index)}`;
      values.append(line);
    });
    host.append(values);
    appendSuggestionAction(host);
    return;
  }

  host.classList.add("good");
  const message = document.createElement("p");
  message.textContent = hasFineEdges
    ? "No numerical change is worth recommending from this assessment. Any remaining straightness or center-bow residual is not correctable with rotation or four-corner perspective controls."
    : "The reliable coarse boundary is within its alignment tolerance, but four refined edge fits are still unavailable. Treat this as coarse alignment complete and use the grid for any final manual refinement.";
  host.append(message);
}

function renderGeometryMetrics(host, metrics, scored) {
  host.replaceChildren();
  metrics.forEach(([label, value, unit, good, warn, signed = false, higherIsBetter = false, precision = 2]) => {
    const row = document.createElement("div");
    const scoreClass = scored
      ? higherIsBetter
        ? (value >= good ? "good" : value >= warn ? "warn" : "bad")
        : (Math.abs(value) <= good ? "good" : Math.abs(value) <= warn ? "warn" : "bad")
      : "info";
    row.className = `metric ${scoreClass}`;
    const formatted = Number.isFinite(value)
      ? `${signed && value > 0 ? "+" : ""}${value.toFixed(precision)}`
      : "—";
    row.innerHTML = `<span>${label}</span><strong>${formatted}${unit}</strong>`;
    host.append(row);
  });
}

function syncOutputGeometry() {
  const host = $("#output-geometry");
  if (measurementOrigin === "automatic") {
    if (
      ["reliable", "tentative"].includes(detectedBoundary.status) &&
      detectedBoundary.corners.length === 4
    ) {
      measurementCorners = detectedBoundary.corners.map((point) => ({ ...point }));
      measurementTouched = new Set([0, 1, 2, 3]);
    } else {
      measurementCorners = defaultMeasurementCorners();
      measurementTouched = new Set();
    }
  }
  syncMeasurementOverlay();
  if (measurementTouched.size < 4) {
    host.innerHTML = '<p class="analysis-pending">No preview boundary was detected. Position all four orange handles on the card corners to measure it manually.</p>';
    syncFramingGeometry([]);
    syncAlignmentAdvice([], 1, []);
    return;
  }
  const fittedEdges = measurementOrigin === "automatic" ? detectedBoundary.edges : [];
  const aspectRatio = imageAspect("#preview-image");
  renderGeometryMetrics(host, outputGeometryMetrics(measurementCorners, aspectRatio, fittedEdges), true);
  let framingReliabilityMessage = "";
  if (measurementOrigin === "automatic" && detectedBoundary.status !== "reliable") {
    framingReliabilityMessage = "The detected boundary is tentative.";
  } else if (measurementOrigin === "automatic" && fittedEdges.length === 4 && !fittedEdgesAreStable(fittedEdges)) {
    const weakEdge = fittedEdges.reduce(
      (lowest, edge, index) => edge.coverage < fittedEdges[lowest].coverage ? index : lowest,
      0,
    );
    framingReliabilityMessage = `${edgeNames[weakEdge]} boundary refinement is unstable at ${(fittedEdges[weakEdge].coverage * 100).toFixed(0)}% coverage.`;
  }
  syncFramingGeometry(measurementCorners, framingReliabilityMessage);
  syncAlignmentAdvice(measurementCorners, aspectRatio, fittedEdges);
  const method = document.createElement("p");
  method.className = "measurement-method";
  const source = measurementOrigin === "manual"
    ? "manual override"
    : `${detectedBoundary.status} preview boundary detection`;
  const refinement = fittedEdges.length === 4
    ? " Angles use sub-degree edge fits; straightness and bow are normalized against image width or height."
    : "";
  method.textContent = `Measurement source: ${source}.${refinement} This does not change the saved image adjustment.`;
  host.prepend(method);
}

start();
