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

let images = [];
let selectedId = 0;
let adjustment = identity();
let renderTimer = null;
let renderGeneration = 0;
let previewUrl = null;
let dragIndex = null;
let measurementCorners = defaultMeasurementCorners();
let measurementTouched = new Set();
let measurementDragIndex = null;
let measurementOrigin = "automatic";
let detectedBoundary = { status: "unavailable", corners: [] };

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
  const response = await fetch(path, { cache: "no-store", ...options });
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
  history.replaceState(null, "", `?image=${selectedId}`);
  detectedBoundary = { status: "unavailable", corners: [] };
  loadMeasurement();
  $("#image-select").value = selectedId;
  const sourceImage = $("#source-image");
  sourceImage.classList.add("is-loading");
  $("#source-loading").hidden = false;
  sourceImage.src = `/api/images/${selectedId}/source?fresh=${Date.now()}`;
  adjustment = identity();
  await loadFixture(false);
  syncAll();
  log(`Selected ${images[selectedId].name}`);
}

async function loadFixture(announce = true) {
  try {
    const response = await fetch(`/api/images/${selectedId}/fixture`, { cache: "no-store" });
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
    card.innerHTML = `<strong>${index + 1}. ${name}</strong><div class="xy">
      <label>X<input type="number" min="0" max="1" step="0.001" data-corner="${index}" data-axis="x"></label>
      <label>Y<input type="number" min="0" max="1" step="0.001" data-corner="${index}" data-axis="y"></label>
    </div>`;
    host.append(card);
  });
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
    values[Number(event.target.dataset.corner)][event.target.dataset.axis] = clamp(Number(event.target.value));
    adjustment.perspective = { corners: values };
    draftChanged();
  });
  for (const selector of ["#grid-spacing", "#grid-x", "#grid-y"]) {
    $(selector).addEventListener("input", syncGrid);
  }
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

function syncAll() {
  $("#rotation").value = adjustment.rotationDegrees;
  $("#zoom").value = adjustment.zoom;
  $("#pan-x").value = adjustment.panX;
  $("#pan-y").value = adjustment.panY;
  corners().forEach((point, index) => {
    $(`[data-corner="${index}"][data-axis="x"]`).value = point.x.toFixed(4);
    $(`[data-corner="${index}"][data-axis="y"]`).value = point.y.toFixed(4);
  });
  $("#json").value = JSON.stringify(canonicalAdjustment(), null, 2);
  syncSourceOverlay();
  syncMeasurementOverlay();
  syncGeometry();
  syncOutputGeometry();
  syncGrid();
  scheduleRender(0);
}

function draftChanged() {
  $("#json").value = JSON.stringify(canonicalAdjustment(), null, 2);
  corners().forEach((point, index) => {
    $(`[data-corner="${index}"][data-axis="x"]`).value = point.x.toFixed(4);
    $(`[data-corner="${index}"][data-axis="y"]`).value = point.y.toFixed(4);
  });
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

async function renderPreview() {
  const generation = ++renderGeneration;
  $("#rendering").hidden = false;
  $("#output-geometry").innerHTML = '<p class="analysis-pending">Analyzing rendered boundary…</p>';
  try {
    const response = await api(`/api/images/${selectedId}/render`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify(canonicalAdjustment()),
    });
    const boundaryStatus = response.headers.get("x-workbench-boundary-status") || "error";
    let boundaryCorners = [];
    try {
      boundaryCorners = JSON.parse(response.headers.get("x-workbench-boundary-corners") || "[]");
    } catch (_error) {
      boundaryCorners = [];
    }
    const blob = await response.blob();
    if (generation !== renderGeneration) return;
    if (previewUrl) URL.revokeObjectURL(previewUrl);
    previewUrl = URL.createObjectURL(blob);
    detectedBoundary = { status: boundaryStatus, corners: boundaryCorners };
    $("#preview-image").src = previewUrl;
    syncOutputGeometry();
  } catch (error) {
    if (generation === renderGeneration) status(error.message, "error");
  } finally {
    if (generation === renderGeneration) $("#rendering").hidden = true;
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
  $("#measurement-overlay").hidden = !$("#show-measurement").checked;
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
  const widthTop = Math.hypot(edges[0].x, edges[0].y);
  const widthBottom = Math.hypot(edges[2].x, edges[2].y);
  const heightRight = Math.hypot(edges[1].x, edges[1].y);
  const heightLeft = Math.hypot(edges[3].x, edges[3].y);
  return [
    ["Worst edge axis deviation", Math.max(...deviations), "°", 0.5, 1.5],
    ["Worst corner from square", Math.max(...angles.map((angle) => Math.abs(90 - angle))), "°", 0.5, 1.5],
    ["Top / bottom length delta", Math.abs(widthTop - widthBottom) / ((widthTop + widthBottom) / 2) * 100, "%", 0.5, 2],
    ["Left / right length delta", Math.abs(heightLeft - heightRight) / ((heightLeft + heightRight) / 2) * 100, "%", 0.5, 2],
  ];
}

function outputGeometryMetrics(points, aspectRatio = 1) {
  const offsets = edgeAxisOffsets(points, aspectRatio);
  const base = geometryMetrics(points, aspectRatio);
  return [
    ["Top edge vs horizontal", offsets[0], "°", 0.5, 1.5, true],
    ["Right edge vs vertical", offsets[1], "°", 0.5, 1.5, true],
    ["Bottom edge vs horizontal", offsets[2], "°", 0.5, 1.5, true],
    ["Left edge vs vertical", offsets[3], "°", 0.5, 1.5, true],
    ["Horizontal-edge parallelism", Math.abs(offsets[0] - offsets[2]), "°", 0.5, 1.5],
    ["Vertical-edge parallelism", Math.abs(offsets[1] - offsets[3]), "°", 0.5, 1.5],
    base[1],
  ];
}

function renderGeometryMetrics(host, metrics, scored) {
  host.replaceChildren();
  metrics.forEach(([label, value, unit, good, warn, signed = false]) => {
    const row = document.createElement("div");
    const scoreClass = scored
      ? (Math.abs(value) <= good ? "good" : Math.abs(value) <= warn ? "warn" : "bad")
      : "info";
    row.className = `metric ${scoreClass}`;
    const formatted = Number.isFinite(value)
      ? `${signed && value > 0 ? "+" : ""}${value.toFixed(2)}`
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
    return;
  }
  renderGeometryMetrics(host, outputGeometryMetrics(measurementCorners, imageAspect("#preview-image")), true);
  const method = document.createElement("p");
  method.className = "measurement-method";
  const source = measurementOrigin === "manual"
    ? "manual override"
    : `${detectedBoundary.status} preview boundary detection`;
  method.textContent = `Measurement source: ${source}. This does not change the saved image adjustment.`;
  host.prepend(method);
}

start();
