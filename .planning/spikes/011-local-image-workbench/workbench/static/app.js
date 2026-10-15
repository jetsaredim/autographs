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
    await selectImage(0);
  } catch (error) {
    status(error.message, "error");
  }
}

async function selectImage(id) {
  selectedId = Number(id);
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
  $("#source-image").addEventListener("load", () => {
    $("#source-image").classList.remove("is-loading");
    $("#source-loading").hidden = true;
    positionOverlays();
    status("Full source loaded. Adjust the draft or run edge detection.");
  });
  $("#source-image").addEventListener("error", () => {
    $("#source-loading").hidden = true;
    status("The source image could not be loaded.", "error");
  });
  $("#preview-image").addEventListener("load", positionOverlays);
  window.addEventListener("resize", positionOverlays);
  const overlay = $("#source-overlay");
  overlay.addEventListener("pointermove", dragCorner);
  overlay.addEventListener("pointerup", endDrag);
  overlay.addEventListener("pointercancel", endDrag);
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
  syncGeometry();
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

function clamp(value) { return Math.max(0, Math.min(1, value)); }

function positionOverlays() {
  positionOverlay($("#source-frame"), $("#source-image"), $("#source-overlay"));
  positionOverlay($("#preview-frame"), $("#preview-image"), $("#preview-grid"));
  positionOverlay($("#preview-frame"), $("#preview-image"), $("#preview-center"));
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
  try {
    const response = await api(`/api/images/${selectedId}/render`, {
      method: "POST", headers: { "content-type": "application/json" },
      body: JSON.stringify(canonicalAdjustment()),
    });
    const blob = await response.blob();
    if (generation !== renderGeneration) return;
    if (previewUrl) URL.revokeObjectURL(previewUrl);
    previewUrl = URL.createObjectURL(blob);
    $("#preview-image").src = previewUrl;
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
  $("#grid-spacing-value").value = `${spacing} px`;
  $("#grid-x-value").value = `${x} px`;
  $("#grid-y-value").value = `${y} px`;
}

function syncGeometry() {
  const points = corners();
  const edges = points.map((point, index) => {
    const next = points[(index + 1) % points.length];
    return { x: next.x - point.x, y: next.y - point.y };
  });
  const deviations = edges.map((edge, index) => {
    const angle = Math.atan2(edge.y, edge.x) * 180 / Math.PI;
    const target = index % 2 === 0 ? Math.round(angle / 180) * 180 : Math.round((angle - 90) / 180) * 180 + 90;
    return Math.abs(angle - target);
  });
  const angles = points.map((_, index) => {
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
  const metrics = [
    ["Worst edge axis deviation", Math.max(...deviations), "°", 0.5, 1.5],
    ["Worst corner from square", Math.max(...angles.map((angle) => Math.abs(90 - angle))), "°", 0.5, 1.5],
    ["Top / bottom width delta", Math.abs(widthTop - widthBottom) * 100, "% frame", 0.5, 2],
    ["Left / right height delta", Math.abs(heightLeft - heightRight) * 100, "% frame", 0.5, 2],
  ];
  const host = $("#geometry");
  host.replaceChildren();
  metrics.forEach(([label, value, unit, good, warn]) => {
    const row = document.createElement("div");
    row.className = `metric ${value <= good ? "good" : value <= warn ? "warn" : "bad"}`;
    row.innerHTML = `<span>${label}</span><strong>${Number.isFinite(value) ? value.toFixed(2) : "—"}${unit}</strong>`;
    host.append(row);
  });
}

start();
