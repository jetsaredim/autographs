const endpoints = {
  health: "/admin/api/health",
  status: "/admin/api/status",
  login: "/admin/api/login",
  logout: "/admin/api/logout",
  items: "/admin/api/items",
  item: (id) => `/admin/api/items/${encodeURIComponent(id)}`,
  history: (id) => `/admin/api/items/${encodeURIComponent(id)}/history`,
  images: (id) => `/admin/api/items/${encodeURIComponent(id)}/images`,
  imagePrimary: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}/primary`,
  imageDelete: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}`,
  imageReplace: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}`,
  cleanupRetry: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}/cleanup/retry`,
  publishIncremental: "/admin/api/publish/incremental",
  publishFull: "/admin/api/publish/full",
  publishStatus: "/admin/api/publish/status",
};

const copy = {
  sessionExpired:
    "Your admin session expired. Log in again to continue; unsent form changes are still on this page.",
  lockout: "Too many login attempts. Wait and try again.",
  saveError:
    "Something did not save. Review the highlighted fields, keep this page open, and try again. If the problem repeats, check the redacted diagnostics panel.",
  removeImage:
    "Remove image: Remove this image from the item and queue cleanup of the private original? This cannot be undone from the admin UI.",
  fullRebuild: "Run a full rebuild only for repair or structural changes. Continue?",
  saveSuccess: "Saved privately. Publish changes when this batch is ready for the public site.",
  publishSuccess: "Published. The public static release is current.",
  cleanupWarning: "Cleanup needs attention. Review the affected item before publishing again.",
};

const state = {
  currentView: "hub-view",
  currentItem: null,
  items: [],
  diagnostics: null,
  dirty: false,
  itemSort: { key: "title", direction: "asc" },
};

const uploadOnlyFieldNames = new Set(["images", "replacementImage", "altText"]);
const adminLoginPath = "/admin/login";
const adminRootPath = "/admin/";
const publicHomePath = "/";

const $ = (selector) => document.querySelector(selector);

const elements = {
  loginView: $("#login-view"),
  workflowView: $("#workflow-view"),
  loginForm: $("#login-form"),
  loginMessage: $("#login-message"),
  logout: $("#logout"),
  sessionStatus: $("#session-status"),
  globalMessage: $("#global-message"),
  tabs: Array.from(document.querySelectorAll(".tab-button")),
  views: Array.from(document.querySelectorAll(".view-panel")),
  itemForm: $("#item-form"),
  itemFilters: $("#item-filters"),
  itemList: $("#item-list"),
  imageGrid: $("#image-grid"),
  imageFiles: $("#image-files"),
  replacementImage: $("#replacement-image"),
  imageMessage: $("#image-message"),
  historyList: $("#history-list"),
  diagnosticsOutput: $("#diagnostics-output"),
  hubDiagnostics: $("#hub-diagnostics"),
  publishStatus: $("#publish-status"),
  publishStatusRows: $("#publish-status-rows"),
  pendingChangeRows: $("#pending-change-rows"),
  cleanupWarningRows: $("#cleanup-warning-rows"),
  runtimeStatusRows: $("#runtime-status-rows"),
  dirtyState: $("#dirty-state"),
  discardUnsaved: $("#discard-unsaved"),
  publishFromEditor: $("#publish-from-editor"),
};

const setText = (selector, value) => {
  const element = $(selector);
  if (element) {
    element.textContent = value;
  }
};

const textNode = (tag, text, className) => {
  const element = document.createElement(tag);
  if (className) {
    element.className = className;
  }
  element.textContent = text;
  return element;
};

const buttonNode = (text, className, onClick) => {
  const button = document.createElement("button");
  button.type = "button";
  button.className = className;
  button.textContent = text;
  button.addEventListener("click", onClick);
  return button;
};

const iconPaths = {
  edit: '<path d="M12 20h9"></path><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"></path>',
  history: '<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z"></path><circle cx="12" cy="12" r="3"></circle>',
  status: '<path d="M22 12h-4l-3 7-6-14-3 7H2"></path>',
};

const iconButton = (label, icon, onClick) => {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "icon-action";
  button.setAttribute("aria-label", label);
  button.title = label;
  button.innerHTML = `<svg aria-hidden="true" viewBox="0 0 24 24">${iconPaths[icon]}</svg>`;
  button.addEventListener("click", onClick);
  return button;
};

const formatEpoch = (seconds) => {
  if (!seconds) {
    return "Not recorded";
  }
  return new Date(seconds * 1000).toLocaleString();
};

const formatValue = (value) => {
  if (value === null || value === undefined) {
    return "Empty";
  }
  if (Array.isArray(value)) {
    return value.length ? value.join(", ") : "Empty";
  }
  if (typeof value === "object") {
    return JSON.stringify(value);
  }
  return String(value);
};

const buildQuery = (form) => {
  const params = new URLSearchParams();
  for (const [key, value] of new FormData(form).entries()) {
    if (key === "changes") {
      continue;
    }
    const trimmed = String(value).trim();
    if (trimmed) {
      params.set(key, trimmed);
    }
  }
  const query = params.toString();
  return query ? `?${query}` : "";
};

const request = async (path, options = {}) => {
  const { allowAnonymous = false, ...fetchOptions } = options;
  const response = await fetch(path, {
    credentials: "same-origin",
    ...fetchOptions,
  });
  if (response.status === 401) {
    if (!allowAnonymous && !elements.workflowView.hidden) {
      handleAuthFailure();
    }
    const error = new Error(copy.sessionExpired);
    error.status = response.status;
    throw error;
  }
  const contentType = response.headers.get("content-type") || "";
  const body = contentType.includes("application/json") ? await response.json() : await response.text();
  if (!response.ok) {
    const error = new Error(typeof body === "string" && body ? body : response.statusText);
    error.status = response.status;
    error.body = body;
    throw error;
  }
  return response.status === 204 ? null : body;
};

const jsonRequest = (path, method, body) =>
  request(path, {
    method,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });

function handleAuthFailure() {
  showLogin(copy.sessionExpired);
  elements.sessionStatus.textContent = copy.sessionExpired;
}

function showWorkflow() {
  elements.loginView.hidden = true;
  elements.workflowView.hidden = false;
  elements.loginMessage.textContent = "";
  elements.sessionStatus.textContent = "Logged in. Private changes stay here until you publish.";
}

function showLogin(message = "") {
  elements.workflowView.hidden = true;
  elements.loginView.hidden = false;
  elements.loginMessage.textContent = message;
}

function setView(viewId) {
  state.currentView = viewId;
  for (const view of elements.views) {
    view.hidden = view.id !== viewId;
  }
  for (const tab of elements.tabs) {
    const active = tab.dataset.view === viewId;
    tab.setAttribute("aria-current", active ? "page" : "false");
  }
  if (viewId === "hub-view") {
    renderHub();
  } else if (viewId === "items-view") {
    renderItemList();
  } else if (viewId === "diagnostics-view") {
    renderDiagnostics();
  }
}

const pendingCopy = (count) => `${count} saved change(s) have not been published yet.`;

const currentAdminPath = () => `${window.location.pathname}${window.location.search}${window.location.hash}`;

const loginRedirectUrl = (next = currentAdminPath()) => {
  const url = new URL(adminLoginPath, window.location.origin);
  url.searchParams.set("next", next);
  return `${url.pathname}${url.search}`;
};

const normalizeNextPath = (next) => {
  if (!next || typeof next !== "string" || next.includes("\\")) {
    return adminRootPath;
  }
  try {
    const url = new URL(next, window.location.origin);
    const isAdminPath = url.pathname === adminRootPath.slice(0, -1) || url.pathname.startsWith(adminRootPath);
    if (url.origin !== window.location.origin || !isAdminPath) {
      return adminRootPath;
    }
    return url.pathname === adminLoginPath ? adminRootPath : `${url.pathname}${url.search}${url.hash}`;
  } catch {
    return adminRootPath;
  }
};

const nextDestination = () => normalizeNextPath(new URLSearchParams(window.location.search).get("next"));

async function renderHub({ allowAnonymous = false } = {}) {
  try {
    const diagnostics = await request(endpoints.status, { allowAnonymous });
    const items = await request(endpoints.items, { allowAnonymous });
    state.items = Array.isArray(items) ? items : [];
    state.diagnostics = diagnostics;
    const pendingCount = diagnostics.pendingChanges?.count || 0;
    const cleanupCount = diagnostics.cleanup?.warningCount || 0;
    setText(
      "#controller-health",
      diagnostics.controller?.ok ? "Healthy. Controller and configured providers responded." : "Needs attention."
    );
    setText("#pending-summary", pendingCopy(pendingCount));
    setText("#publish-summary", publishSummaryText(diagnostics.publish));
    setText(
      "#cleanup-summary",
      cleanupCount > 0 ? copy.cleanupWarning : "0 cleanup warnings"
    );
    setText(
      "#retention-summary",
      `${diagnostics.releaseRetention?.promotedReleaseCount || 0} promoted release(s), ${
        diagnostics.releaseRetention?.failedCandidateCount || 0
      } failed candidate(s) retained.`
    );
    setText("#publish-pending-summary", pendingCopy(pendingCount));
    elements.hubDiagnostics.textContent = JSON.stringify(diagnostics, null, 2);
    renderHubStatusSections(diagnostics);
    renderDiagnostics();
    return true;
  } catch (error) {
    if (error.status !== 401) {
      elements.globalMessage.textContent = `Status unavailable: ${error.message}`;
    }
    return false;
  }
}

function renderHubStatusSections(diagnostics) {
  replaceRows(elements.publishStatusRows, [
    ["State", diagnostics.publish?.state || "idle"],
    ["Release", diagnostics.publish?.releaseId || "Not recorded"],
    ["Artifacts", String(diagnostics.publish?.artifactCount || 0)],
    ["Bytes", String(diagnostics.publish?.byteSize || 0)],
    ["Finished", formatEpoch(diagnostics.publish?.finishedAtEpochSeconds)],
  ]);

  const pendingItems = state.items.filter((item) => item.hasPendingChanges);
  elements.pendingChangeRows.replaceChildren();
  if (pendingItems.length === 0) {
    appendTableMessage(elements.pendingChangeRows, "No pending item changes.", 4);
  } else {
    for (const item of pendingItems) {
      appendRow(elements.pendingChangeRows, [
        item.title,
        item.signer,
        item.publicationStatus,
        formatEpoch(item.updatedAtEpochSeconds),
      ]);
    }
  }

  const cleanupWarnings = diagnostics.cleanup?.warnings || [];
  elements.cleanupWarningRows.replaceChildren();
  if (cleanupWarnings.length === 0) {
    appendTableMessage(elements.cleanupWarningRows, "No cleanup warnings.", 4);
  } else {
    for (const warning of cleanupWarnings) {
      appendRow(elements.cleanupWarningRows, [
        warning.title || warning.itemId || "Item",
        warning.operation,
        warning.status,
        warning.adminMessage,
      ]);
    }
  }

  replaceRows(elements.runtimeStatusRows, [
    ["Controller", diagnostics.controller?.ok ? "Healthy" : "Needs attention"],
    ["Database provider", diagnostics.providers?.database || "Unknown"],
    ["Media provider", diagnostics.providers?.media || "Unknown"],
    [
      "Promoted releases",
      `${diagnostics.releaseRetention?.promotedReleaseCount || 0} of ${
        diagnostics.releaseRetention?.promotedReleaseRetainCount || 0
      } retained`,
    ],
    [
      "Failed candidates",
      `${diagnostics.releaseRetention?.failedCandidateCount || 0} of ${
        diagnostics.releaseRetention?.failedCandidateRetainCount || 0
      } retained`,
    ],
  ]);
}

function replaceRows(body, rows) {
  body.replaceChildren();
  for (const row of rows) {
    appendRow(body, row);
  }
}

function appendRow(body, values) {
  const row = document.createElement("tr");
  for (const value of values) {
    const cell = document.createElement("td");
    cell.textContent = value || "Empty";
    row.append(cell);
  }
  body.append(row);
}

function appendTableMessage(body, message, columns) {
  const row = document.createElement("tr");
  const cell = document.createElement("td");
  cell.colSpan = columns;
  cell.className = "empty-table-cell";
  cell.textContent = message;
  row.append(cell);
  body.append(row);
}

const publishSummaryText = (publish) => {
  if (!publish) {
    return "Idle";
  }
  const stateLabel = publish.state || "idle";
  const release = publish.releaseId ? ` release ${publish.releaseId}` : "";
  const finished = publish.finishedAtEpochSeconds
    ? ` at ${formatEpoch(publish.finishedAtEpochSeconds)}`
    : "";
  return `${stateLabel}${release}${finished}`;
};

async function renderItemList() {
  try {
    const items = await request(`${endpoints.items}${buildQuery(elements.itemFilters)}`);
    const changeFilter = elements.itemFilters.elements.changes.value;
    state.items = (Array.isArray(items) ? items : [])
      .filter((item) => {
        if (changeFilter === "pending") {
          return item.hasPendingChanges;
        }
        if (changeFilter === "clean") {
          return !item.hasPendingChanges;
        }
        return true;
      })
      .sort(compareItems);
    elements.itemList.replaceChildren();
    if (state.items.length === 0) {
      const empty = document.createElement("div");
      empty.className = "empty-state";
      empty.append(
        textNode("h3", "No saved items yet"),
        textNode(
          "p",
          "Start with the backlog: add an autograph item, upload its images, save it privately, then publish when the batch is ready."
        )
      );
      elements.itemList.append(empty);
      return;
    }
    const table = document.createElement("table");
    table.append(itemTableHead());
    const body = document.createElement("tbody");
    for (const item of state.items) {
      const row = document.createElement("tr");
      for (const value of [
        item.title,
        item.signer,
        item.publicationStatus,
        String(item.imageCount || 0),
        item.hasPendingChanges ? "Pending" : "Clean",
        formatEpoch(item.updatedAtEpochSeconds),
      ]) {
        const cell = document.createElement("td");
        cell.textContent = value || "Empty";
        row.append(cell);
      }
      const actions = document.createElement("td");
      actions.className = "row-actions";
      actions.append(
        iconButton("Edit item", "edit", () => loadItem(item.id)),
        iconButton("View history", "history", () => loadItem(item.id, true)),
        iconButton("Publish status", "status", () => setView("publish-view"))
      );
      row.append(actions);
      body.append(row);
    }
    table.append(body);
    elements.itemList.append(table);
  } catch (error) {
    if (error.status !== 401) {
      elements.itemList.replaceChildren(textNode("p", `Item list unavailable: ${error.message}`, "empty-state"));
    }
  }
}

const itemTableHead = () => {
  const head = document.createElement("thead");
  const row = document.createElement("tr");
  for (const column of [
    { label: "Title", key: "title" },
    { label: "Signer", key: "signer" },
    { label: "Status" },
    { label: "Images" },
    { label: "Changes" },
    { label: "Updated" },
    { label: "Actions" },
  ]) {
    const header = document.createElement("th");
    if (column.key) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "sort-button";
      button.textContent = sortLabel(column.label, column.key);
      button.addEventListener("click", () => {
        updateSort(column.key);
        renderItemList();
      });
      header.append(button);
    } else {
      header.textContent = column.label;
    }
    row.append(header);
  }
  head.append(row);
  return head;
};

function sortLabel(label, key) {
  if (state.itemSort.key !== key) {
    return label;
  }
  return `${label} ${state.itemSort.direction === "asc" ? "↑" : "↓"}`;
}

function updateSort(key) {
  if (state.itemSort.key === key) {
    state.itemSort.direction = state.itemSort.direction === "asc" ? "desc" : "asc";
  } else {
    state.itemSort = { key, direction: "asc" };
  }
}

function compareItems(left, right) {
  const direction = state.itemSort.direction === "asc" ? 1 : -1;
  const leftValue = String(left[state.itemSort.key] || "").toLowerCase();
  const rightValue = String(right[state.itemSort.key] || "").toLowerCase();
  return (
    leftValue.localeCompare(rightValue) * direction ||
    String(left.id || "").localeCompare(String(right.id || ""))
  );
}

function renderEditor(item = null) {
  state.currentItem = item;
  state.dirty = false;
  elements.itemForm.reset();
  elements.discardUnsaved.hidden = true;
  elements.publishFromEditor.setAttribute("aria-disabled", "false");
  setText("#dirty-state", "No unsaved client-side edits.");
  setText("#editor-title", item ? "Edit item" : "Add item");
  setText(
    "#editor-context",
    item
      ? "Existing items hydrate into this same editor. Saving still stays private until publish."
      : "Backlog entry starts as a private draft. Save privately, then publish when the batch is ready."
  );
  const values = item || { publicationStatus: "draft", tags: [], images: [] };
  for (const [name, value] of Object.entries({
    itemId: values.id || "",
    title: values.title || "",
    signer: values.signer || "",
    category: values.category || "",
    tags: Array.isArray(values.tags) ? values.tags.join(", ") : "",
    objectReference: values.objectReference || "",
    estimatedYear: values.estimatedYear || "",
    description: values.description || "",
    inscription: values.inscription || "",
    eventName: values.eventName || "",
    eventLocation: values.eventLocation || "",
    source: values.source || "",
    certificationCompany: values.certificationCompany || "",
    certificationId: values.certificationId || "",
    publicationStatus: values.publicationStatus || "draft",
  })) {
    if (elements.itemForm.elements[name]) {
      elements.itemForm.elements[name].value = value;
    }
  }
  renderImages(values.images || [], values.cleanupWarnings || []);
  renderHistory(item?.id);
  setView("add-item-view");
}

function renderImages(images = [], cleanupWarnings = []) {
  elements.imageGrid.replaceChildren();
  if (!state.currentItem?.id) {
    elements.imageGrid.append(textNode("p", "Save the item before uploading images.", "empty-state"));
    return;
  }
  if (images.length === 0) {
    elements.imageGrid.append(textNode("p", "No images uploaded yet.", "empty-state"));
    return;
  }
  const warningsByImage = new Map(cleanupWarnings.map((warning) => [warning.imageId, warning]));
  for (const image of [...images].sort((a, b) => Number(b.isPrimary) - Number(a.isPrimary))) {
    const tile = document.createElement("article");
    tile.className = image.isPrimary ? "image-tile primary-image" : "image-tile";
    tile.append(
      textNode("h4", image.isPrimary ? "Primary image" : "Supporting image"),
      textNode("p", image.altText || "No alt text recorded."),
      textNode("p", `${image.contentType || "image"} - ${image.byteSize || 0} bytes`, "helper-text")
    );
    const warning = warningsByImage.get(image.id);
    if (warning) {
      tile.append(textNode("p", warning.adminMessage || copy.cleanupWarning, "status-warning"));
    }
    const actions = document.createElement("div");
    actions.className = "inline-actions";
    actions.append(
      buttonNode("Mark primary", "secondary-action", () => markPrimary(image.id)),
      buttonNode("Remove image", "destructive", () => removeImage(image.id)),
      buttonNode("Replace image", "secondary-action", () => replaceImage(image.id))
    );
    if (warning) {
      actions.append(buttonNode("Retry cleanup", "secondary-action", () => retryCleanup(image.id)));
    }
    tile.append(actions);
    elements.imageGrid.append(tile);
  }
}

async function renderHistory(itemId = state.currentItem?.id) {
  elements.historyList.replaceChildren();
  if (!itemId) {
    elements.historyList.append(
      textNode(
        "p",
        "No history recorded yet. Changes made after the Phase 6 history update will appear here.",
        "empty-state"
      )
    );
    return;
  }
  try {
    const history = await request(endpoints.history(itemId));
    const events = history.events || [];
    if (events.length === 0) {
      elements.historyList.append(
        textNode(
          "p",
          "No history recorded yet. Changes made after the Phase 6 history update will appear here.",
          "empty-state"
        )
      );
      return;
    }
    for (const event of events) {
      const row = document.createElement("article");
      row.className = "history-entry";
      row.append(
        textNode("h4", event.summary || event.eventType),
        textNode("p", `${event.eventType} - ${formatEpoch(event.createdAtEpochSeconds)}`, "helper-text")
      );
      for (const diff of event.fieldDiffs || []) {
        const diffRow = document.createElement("div");
        diffRow.className = "diff-row";
        diffRow.append(
          textNode("span", diff.field),
          textNode("span", formatValue(diff.before)),
          textNode("span", formatValue(diff.after))
        );
        row.append(diffRow);
      }
      elements.historyList.append(row);
    }
  } catch (error) {
    if (error.status !== 401) {
      elements.historyList.append(textNode("p", `History unavailable: ${error.message}`, "empty-state"));
    }
  }
}

function renderDiagnostics() {
  const diagnostics = state.diagnostics || {};
  elements.diagnosticsOutput.textContent = JSON.stringify(diagnostics, null, 2);
}

const formPayload = () => {
  const form = elements.itemForm;
  const estimatedYear = form.elements.estimatedYear.value.trim();
  const optional = (name) => {
    const value = form.elements[name].value.trim();
    return value || null;
  };
  return {
    title: form.elements.title.value.trim(),
    signer: form.elements.signer.value.trim(),
    description: optional("description"),
    category: form.elements.category.value.trim(),
    tags: form.elements.tags.value.split(",").map((tag) => tag.trim()).filter(Boolean),
    objectReference: optional("objectReference"),
    eventName: optional("eventName"),
    eventLocation: optional("eventLocation"),
    source: optional("source"),
    inscription: optional("inscription"),
    certificationCompany: optional("certificationCompany"),
    certificationId: optional("certificationId"),
    estimatedYear: estimatedYear ? Number(estimatedYear) : null,
    publicationStatus: form.elements.publicationStatus.value,
  };
};

async function saveItem(event) {
  event.preventDefault();
  const id = elements.itemForm.elements.itemId.value.trim();
  const selectedFiles = Array.from(elements.imageFiles.files);
  const selectedAltText = elements.itemForm.elements.altText.value.trim();
  try {
    const item = await jsonRequest(id ? endpoints.item(id) : endpoints.items, id ? "PATCH" : "POST", formPayload());
    state.currentItem = item;
    if (selectedFiles.length) {
      state.dirty = false;
      elements.discardUnsaved.hidden = true;
      elements.publishFromEditor.setAttribute("aria-disabled", "false");
      elements.dirtyState.textContent = "No unsaved client-side edits.";
      await uploadImages(item.id, selectedFiles, selectedAltText, { allowDirty: true });
    } else {
      renderEditor(item);
    }
    elements.globalMessage.textContent = copy.saveSuccess;
    elements.globalMessage.focus();
    await renderHub();
  } catch (error) {
    if (error.status !== 401) {
      elements.globalMessage.textContent = error.status === 429 ? copy.lockout : copy.saveError;
    }
  }
}

async function uploadImages(
  itemId = state.currentItem?.id || elements.itemForm.elements.itemId.value.trim(),
  files = Array.from(elements.imageFiles.files),
  altText = elements.itemForm.elements.altText.value.trim(),
  options = {}
) {
  if (!options.allowDirty && !ensureSavedBeforeImageChange()) {
    return false;
  }
  const selectedFiles = Array.from(files);
  if (!itemId || selectedFiles.length === 0) {
    return false;
  }
  try {
    for (const file of selectedFiles) {
      const upload = new FormData();
      upload.append("image", file);
      upload.append("altText", altText);
      const item = await request(endpoints.images(itemId), {
        method: "POST",
        body: upload,
      });
      state.currentItem = item;
    }
    elements.imageFiles.value = "";
    renderEditor(state.currentItem);
    elements.imageMessage.textContent = "Images uploaded. Mark one primary image if needed.";
    return true;
  } catch (error) {
    if (error.status !== 401) {
      elements.imageMessage.textContent = `Image upload failed: ${error.message}`;
    }
    return false;
  }
}

async function markPrimary(imageId) {
  if (!state.currentItem?.id) {
    return;
  }
  if (!ensureSavedBeforeImageChange()) {
    return;
  }
  try {
    const item = await request(endpoints.imagePrimary(state.currentItem.id, imageId), { method: "POST" });
    state.currentItem = item;
    renderEditor(item);
  } catch (error) {
    if (error.status !== 401) {
      elements.imageMessage.textContent = `Primary image update failed: ${error.message}`;
    }
  }
}

async function removeImage(imageId) {
  if (!state.currentItem?.id || !window.confirm(copy.removeImage)) {
    return;
  }
  if (!ensureSavedBeforeImageChange()) {
    return;
  }
  try {
    const item = await request(endpoints.imageDelete(state.currentItem.id, imageId), { method: "DELETE" });
    state.currentItem = item;
    renderEditor(item);
  } catch (error) {
    if (error.status === 409 && error.body?.cleanupWarning) {
      elements.imageMessage.textContent = copy.cleanupWarning;
      await loadItem(state.currentItem.id);
    } else if (error.status !== 401) {
      elements.imageMessage.textContent = `Image removal failed: ${error.message}`;
    }
  }
}

async function replaceImage(imageId) {
  if (!state.currentItem?.id) {
    return;
  }
  if (!ensureSavedBeforeImageChange()) {
    return;
  }
  const file = elements.replacementImage.files[0];
  if (!file) {
    elements.imageMessage.textContent = "Choose a replacement image first.";
    return;
  }
  const upload = new FormData();
  upload.append("image", file);
  upload.append("altText", elements.itemForm.elements.altText.value.trim());
  try {
    const item = await request(endpoints.imageReplace(state.currentItem.id, imageId), {
      method: "PUT",
      body: upload,
    });
    state.currentItem = item;
    elements.replacementImage.value = "";
    renderEditor(item);
  } catch (error) {
    if (error.status === 409 && error.body?.cleanupWarning) {
      elements.imageMessage.textContent = copy.cleanupWarning;
      await loadItem(state.currentItem.id);
    } else if (error.status !== 401) {
      elements.imageMessage.textContent = `Image replacement failed: ${error.message}`;
    }
  }
}

async function retryCleanup(imageId) {
  if (!state.currentItem?.id) {
    return;
  }
  if (!ensureSavedBeforeImageChange()) {
    return;
  }
  try {
    const item = await request(endpoints.cleanupRetry(state.currentItem.id, imageId), { method: "POST" });
    if (item) {
      state.currentItem = item;
      renderEditor(item);
    } else {
      elements.imageMessage.textContent = "Cleanup retry succeeded.";
      await loadItem(state.currentItem.id);
    }
  } catch (error) {
    if (error.status !== 401) {
      elements.imageMessage.textContent = `Cleanup retry failed: ${error.message}`;
    }
  }
}

function ensureSavedBeforePublish() {
  if (!state.dirty) {
    return true;
  }
  setView("add-item-view");
  elements.globalMessage.textContent = "Save item before publishing these changes.";
  elements.globalMessage.focus();
  return false;
}

function ensureSavedBeforeImageChange() {
  if (!state.dirty) {
    return true;
  }
  setView("add-item-view");
  elements.globalMessage.textContent = "Save item before changing images.";
  elements.globalMessage.focus();
  return false;
}

function ensureSavedBeforeOpeningAnotherItem() {
  if (!state.dirty) {
    return true;
  }
  setView("add-item-view");
  elements.globalMessage.textContent = "Save item before opening another item.";
  elements.globalMessage.focus();
  return false;
}

async function publishChanges(mode = "incremental") {
  if (!ensureSavedBeforePublish()) {
    return;
  }
  if (mode === "full" && !window.confirm(copy.fullRebuild)) {
    return;
  }
  elements.publishStatus.textContent = "Publishing";
  try {
    const status = await request(mode === "full" ? endpoints.publishFull : endpoints.publishIncremental, {
      method: "POST",
    });
    elements.publishStatus.textContent = JSON.stringify(status, null, 2);
    setText("#publish-state", status.state || "Succeeded");
    setText("#release-summary", publishSummaryText(status));
    setText("#publish-next-action", copy.publishSuccess);
    elements.globalMessage.textContent = copy.publishSuccess;
    elements.globalMessage.focus();
    await renderHub();
  } catch (error) {
    if (error.status !== 401) {
      setText("#publish-state", "Failed");
      setText("#publish-next-action", "Retry publish, inspect diagnostics, or run live smoke guidance.");
      elements.publishStatus.textContent = `Publish failed: ${error.message}`;
    }
  }
}

const loadItem = async (id, historyFirst = false) => {
  if (!ensureSavedBeforeOpeningAnotherItem()) {
    return;
  }
  try {
    const item = await request(endpoints.item(id));
    renderEditor(item);
    if (historyFirst) {
      await renderHistory(id);
    }
  } catch (error) {
    if (error.status !== 401) {
      elements.globalMessage.textContent = `Item unavailable: ${error.message}`;
    }
  }
};

const markDirty = (event) => {
  if (uploadOnlyFieldNames.has(event?.target?.name)) {
    return;
  }
  state.dirty = true;
  elements.discardUnsaved.hidden = false;
  elements.publishFromEditor.setAttribute("aria-disabled", "true");
  elements.dirtyState.textContent = "Unsaved client-side edits. Save before publishing.";
};

function publishFromEditor() {
  return publishChanges("incremental");
}

async function bootstrapSession() {
  const onLoginRoute = window.location.pathname === adminLoginPath;
  const hasSession = await renderHub({ allowAnonymous: true });
  if (hasSession) {
    if (onLoginRoute) {
      window.location.replace(nextDestination());
      return;
    }
    showWorkflow();
  } else {
    if (onLoginRoute) {
      showLogin();
    } else {
      window.location.replace(loginRedirectUrl());
    }
  }
}

elements.loginForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  elements.loginMessage.textContent = "";
  try {
    await jsonRequest(endpoints.login, "POST", {
      password: event.currentTarget.elements.password.value,
    });
    const next = nextDestination();
    event.currentTarget.reset();
    if (window.location.pathname === adminRootPath && next === adminRootPath) {
      showWorkflow();
      await renderHub();
      return;
    }
    window.location.replace(next);
  } catch (error) {
    if (error.status === 401 || error.status === 429) {
      window.location.replace(publicHomePath);
    } else {
      elements.loginMessage.textContent = error.status === 429 ? copy.lockout : "Login failed.";
    }
  }
});

elements.logout.addEventListener("click", async () => {
  try {
    await request(endpoints.logout, { method: "POST" });
  } finally {
    showLogin("Logged out.");
  }
});

for (const tab of elements.tabs) {
  tab.addEventListener("click", () => setView(tab.dataset.view));
}

$("#refresh-status").addEventListener("click", renderHub);
$("#refresh-diagnostics").addEventListener("click", renderHub);
$("#refresh-items").addEventListener("click", renderItemList);
$("#refresh-history").addEventListener("click", () => renderHistory());
$("#back-to-hub").addEventListener("click", () => setView("hub-view"));
$("#add-another-item").addEventListener("click", () => {
  if (ensureSavedBeforeOpeningAnotherItem()) {
    renderEditor();
  }
});
$("#discard-unsaved").addEventListener("click", () => renderEditor(state.currentItem));
$("#upload-more-images").addEventListener("click", () => uploadImages());
$("#publish-from-editor").addEventListener("click", publishFromEditor);
$("#publish-incremental").addEventListener("click", () => publishChanges("incremental"));
$("#publish-full").addEventListener("click", () => publishChanges("full"));

elements.itemForm.addEventListener("submit", saveItem);
elements.itemForm.addEventListener("input", markDirty);
elements.itemFilters.addEventListener("submit", (event) => {
  event.preventDefault();
  renderItemList();
});

bootstrapSession();
