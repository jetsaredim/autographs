const endpoints = {
  health: "/admin/api/health",
  status: "/admin/api/status",
  login: "/admin/api/login",
  logout: "/admin/api/logout",
  items: "/admin/api/items",
  item: (id) => `/admin/api/items/${encodeURIComponent(id)}`,
  history: (id) => `/admin/api/items/${encodeURIComponent(id)}/history`,
  signers: (query) => `/admin/api/signers?query=${encodeURIComponent(query)}`,
  signer: (id) => `/admin/api/signers/${encodeURIComponent(id)}`,
  signerMerge: "/admin/api/signers/merge",
  taxonomySuggestions: "/admin/api/taxonomy/suggestions",
  images: (id) => `/admin/api/items/${encodeURIComponent(id)}/images`,
  imagePrimary: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}/primary`,
  imageDelete: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}`,
  imageReplace: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}`,
  imagePreview: (id, imageId, mediaRevision) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}/preview?mediaRevision=${encodeURIComponent(mediaRevision)}`,
  imageDraftPreview: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}/preview/draft`,
  imageSourcePreview: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}/preview/source`,
  imageReview: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}/review`,
  imageAdjustment: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}/adjustment`,
  imageAdjustmentAssist: (id, imageId) =>
    `/admin/api/items/${encodeURIComponent(id)}/images/${encodeURIComponent(imageId)}/adjustment/assist`,
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
  fullRebuild: "Run a full rebuild after schema or taxonomy migration changes. Continue?",
  saveSuccess: "Saved privately. Publish changes when this taxonomy batch is ready for the public site.",
  publishSuccess: "Published. Public facets now reflect the saved taxonomy.",
  cleanupWarning: "Cleanup needs attention. Review the affected item before publishing again.",
  signerDuplicate: "Possible duplicate signer. Review the existing profile before saving a new signer.",
  signerCreate: "Type a name to create a new signer, or choose an existing signer.",
  mergeSigner:
    "Merge signer: Merge these signer profiles and update linked items? Review the target profile first; this cannot be undone from the admin UI.",
  previewError:
    "Preview unavailable. Retry the preview or replace the image; provider details are hidden from the browser.",
  privateOnly: "Private image only. Publish when this item is ready for the public catalog.",
  adjustmentSaved:
    "Adjustments saved privately. Publish changes when this image is ready for the public site.",
  assistUnavailable: "Auto correction could not find reliable edges. Adjust the corners manually.",
  mediaChanged: "Image media changed. Reopen the review.",
  resetAdjustment:
    "Reset adjustments: Clear saved crop, rotation, pan, and perspective correction for this image? The original upload stays unchanged.",
  discardImageEdits: "Discard unsaved image edits and return to the item editor?",
  discardImageEditsForNavigation: "Discard unsaved image edits and continue?",
};

const state = {
  currentView: "hub-view",
  currentItem: null,
  items: [],
  diagnostics: null,
  dirty: false,
  itemSort: { key: "title", direction: "asc" },
  signerSuggestions: [],
  managedSigners: [],
  focusedSignerId: null,
  taxonomySuggestions: {},
  reviewImage: null,
  reviewDraftAdjustment: null,
  reviewSavedAdjustment: null,
  reviewDirty: false,
  reviewComparisonMode: "latest",
  reviewOverlays: { grid: false, centerline: false, edges: true },
  reviewPreviewUrl: null,
  reviewPreviewStatus: "idle",
  reviewPreviewAbortController: null,
  reviewPreviewTimer: null,
  reviewPreviewRevision: 0,
  reviewDisplayedRevision: null,
  reviewOutputRenderGeneration: 0,
  reviewMountedOutput: null,
  reviewOutputPanel: null,
  reviewDraftRevision: 0,
  reviewSessionRevision: 0,
  reviewSession: null,
  reviewMutationPending: null,
  reviewMutationRevision: 0,
  reviewPerspectiveGeneration: 0,
  reviewPerspectiveFrame: null,
  reviewPerspectiveImage: null,
  reviewPerspectiveObserver: null,
  reviewPerspectiveResizeListener: null,
  reviewPerspectiveDrag: null,
  reviewSourceProjectionStatus: "idle",
  reviewStageRenderDeferred: false,
  reviewFocusedCornerIndex: null,
  reviewMessage: "",
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
  itemListStatus: $("#item-list-status"),
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
  signerRows: $("#signer-rows"),
  signerWarningSummary: $("#signer-warning-summary"),
  signerMergePanel: $("#signer-merge-panel"),
  signerManagementForm: $("#signer-management-form"),
  signerManagementQuery: $("#signer-management-query"),
  signerManagementRows: $("#signer-management-rows"),
  signerManagementMessage: $("#signer-management-message"),
  imageReviewStage: $("#image-review-stage"),
  imageReviewControls: $("#image-review-controls"),
  imageReviewMessage: $("#image-review-message"),
  imageReviewSave: $("#image-review-save"),
  imageReviewDiscard: $("#image-review-discard"),
  imageReviewReset: $("#image-review-reset"),
  imageReviewDirtyBand: $("#image-review-dirty-band"),
  publishIncremental: $("#publish-incremental"),
  publishFull: $("#publish-full"),
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

const loadingState = (message) => {
  const wrapper = document.createElement("div");
  wrapper.className = "loading-state";
  wrapper.textContent = message;
  return wrapper;
};

const nextFrame = () => new Promise((resolve) => requestAnimationFrame(resolve));

const iconPaths = {
  archived:
    '<path d="M21 8v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8"></path><path d="M10 12h4"></path><path d="M1 3h22v5H1z"></path>',
  clean: '<path d="M20 6 9 17l-5-5"></path>',
  draft: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"></path><path d="M14 2v6h6"></path>',
  edit: '<path d="M12 20h9"></path><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z"></path>',
  history: '<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z"></path><circle cx="12" cy="12" r="3"></circle>',
  pending: '<path d="M12 9v4"></path><path d="M12 17h.01"></path><circle cx="12" cy="12" r="10"></circle>',
  published: '<path d="M12 2v20"></path><path d="m17 5-5-3-5 3"></path><path d="m17 19-5 3-5-3"></path><path d="M2 12h20"></path><path d="m5 7-3 5 3 5"></path><path d="m19 7 3 5-3 5"></path>',
  status: '<path d="M22 12h-4l-3 7-6-14-3 7H2"></path>',
};

const iconButton = (label, icon, onClick) => {
  const button = document.createElement("button");
  button.type = "button";
  button.className = "icon-action review-egress-control";
  button.disabled = Boolean(state.reviewMutationPending);
  button.setAttribute("aria-label", label);
  button.title = label;
  button.innerHTML = `<svg aria-hidden="true" viewBox="0 0 24 24">${iconPaths[icon]}</svg>`;
  button.addEventListener("click", onClick);
  return button;
};

const iconNode = (icon) => {
  const wrapper = document.createElement("span");
  wrapper.className = "status-icon-glyph";
  wrapper.innerHTML = `<svg aria-hidden="true" viewBox="0 0 24 24">${iconPaths[icon]}</svg>`;
  return wrapper;
};

const iconBadge = (label, icon, tone) => {
  const badge = document.createElement("span");
  badge.className = `status-icon ${tone}`;
  badge.setAttribute("role", "img");
  badge.setAttribute("aria-label", label);
  badge.title = label;
  badge.append(iconNode(icon));
  return badge;
};

const publicationStatusParts = (status) => {
  const normalized = String(status || "draft").toLowerCase();
  if (normalized === "published") {
    return { label: "Published", icon: "published", tone: "status-icon-success" };
  }
  if (normalized === "archived") {
    return { label: "Archived", icon: "archived", tone: "status-icon-muted" };
  }
  return { label: "Draft", icon: "draft", tone: "status-icon-neutral" };
};

const publicationStatusButton = (status, onClick) => {
  const { label, icon, tone } = publicationStatusParts(status);
  const button = document.createElement("button");
  button.type = "button";
  button.className = `status-icon status-icon-action review-egress-control ${tone}`;
  button.disabled = Boolean(state.reviewMutationPending);
  button.setAttribute("aria-label", `Publish status: ${label}`);
  button.title = `Publish status: ${label}`;
  button.append(iconNode(icon));
  button.addEventListener("click", onClick);
  return button;
};

const pendingChangesIcon = (hasPendingChanges) =>
  hasPendingChanges
    ? iconBadge("Pending changes", "pending", "status-icon-warning")
    : iconBadge("No pending changes", "clean", "status-icon-success");

const imageCountLabel = (count) => {
  const imageCount = Number(count) || 0;
  return `${imageCount} image${imageCount === 1 ? "" : "s"}`;
};

const stateCell = (item) => {
  const cell = document.createElement("td");
  cell.className = "state-cell";
  const { label } = publicationStatusParts(item.publicationStatus);
  const layout = document.createElement("span");
  layout.className = "state-layout";
  const copy = document.createElement("span");
  copy.className = "state-copy";
  copy.append(
    textNode("span", label, "state-label"),
    textNode(
      "span",
      `${imageCountLabel(item.imageCount)} · ${formatRelativeEpoch(item.updatedAtEpochSeconds)}`,
      "state-meta"
    )
  );
  copy.title = formatEpoch(item.updatedAtEpochSeconds);
  const icons = document.createElement("span");
  icons.className = "state-icons";
  icons.append(
    pendingChangesIcon(item.hasPendingChanges),
    publicationStatusButton(item.publicationStatus, () => navigateToView("publish-view"))
  );
  layout.append(copy, icons);
  cell.append(layout);
  return cell;
};

const taxonomyCell = (item) => {
  const cell = document.createElement("td");
  cell.className = "taxonomy-cell";
  const content = document.createElement("div");
  content.className = "taxonomy-cell-content";
  const franchises = item.franchises?.join(", ") || "";
  const productLine = item.productLine || "";
  cell.title = [franchises, productLine].filter(Boolean).join(" / ") || "Empty";
  content.append(
    textNode("span", franchises || "Empty", "taxonomy-primary"),
    textNode("span", productLine || "No product line", "taxonomy-secondary")
  );
  cell.append(content);
  return cell;
};

const formatEpoch = (seconds) => {
  if (!seconds) {
    return "Not recorded";
  }
  return new Date(seconds * 1000).toLocaleString();
};

const formatRelativeEpoch = (seconds) => {
  const epochSeconds = Number(seconds);
  if (!Number.isFinite(epochSeconds) || epochSeconds <= 0) {
    return "Not recorded";
  }
  const diffSeconds = Math.max(0, Math.floor(Date.now() / 1000) - epochSeconds);
  if (diffSeconds < 60) {
    return "Now";
  }
  const units = [
    ["y", 365 * 24 * 60 * 60],
    ["mo", 30 * 24 * 60 * 60],
    ["d", 24 * 60 * 60],
    ["h", 60 * 60],
    ["m", 60],
  ];
  const [suffix, unitSeconds] = units.find(([, unitSeconds]) => diffSeconds >= unitSeconds);
  return `${Math.floor(diffSeconds / unitSeconds)}${suffix} ago`;
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
  elements.sessionStatus.textContent = state.reviewDirty
    ? `${copy.sessionExpired} Unsaved image edits remain in this page until you log in or close it.`
    : copy.sessionExpired;
}

function showWorkflow() {
  elements.loginView.hidden = true;
  elements.workflowView.hidden = false;
  elements.loginMessage.textContent = "";
  elements.sessionStatus.textContent = "Logged in. Private changes stay here until you publish.";
  loadTaxonomySuggestions();
}

function showLogin(message = "") {
  finishPerspectiveDrag({ settle: false, reason: "login-hidden" });
  state.reviewStageRenderDeferred = false;
  elements.workflowView.hidden = true;
  elements.loginView.hidden = false;
  elements.loginMessage.textContent = message;
}

function setView(viewId) {
  if (
    viewId !== "image-review-view" &&
    state.reviewSession &&
    blockReviewEgressWhileMutationPending()
  ) {
    return false;
  }
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
  } else if (viewId === "signers-view") {
    renderSignerManagement();
  } else if (viewId === "diagnostics-view") {
    renderDiagnostics();
  }
  return true;
}

function confirmDiscardReviewForNavigation() {
  if (blockReviewEgressWhileMutationPending()) {
    return false;
  }
  if (state.reviewDirty && !window.confirm(copy.discardImageEditsForNavigation)) {
    return false;
  }
  if (state.reviewSession) {
    clearImageReviewState();
  }
  return true;
}

function navigateToView(viewId) {
  if (
    viewId !== "image-review-view" &&
    state.reviewSession &&
    !confirmDiscardReviewForNavigation()
  ) {
    return false;
  }
  return setView(viewId);
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
        item.signerText || item.signer,
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
  const cleanup = publish.cleanupWarning ? " — release active; cleanup retry required" : "";
  return `${stateLabel}${release}${finished}${cleanup}`;
};

async function renderItemList() {
  elements.itemList.setAttribute("aria-busy", "true");
  elements.itemListStatus.textContent = "Requesting item summaries...";
  elements.itemList.replaceChildren(loadingState("Requesting item summaries..."));
  try {
    const items = await request(`${endpoints.items}${buildQuery(elements.itemFilters)}`);
    const itemCount = Array.isArray(items) ? items.length : 0;
    elements.itemListStatus.textContent = `Preparing ${itemCount} item${itemCount === 1 ? "" : "s"}...`;
    elements.itemList.replaceChildren(loadingState(`Preparing ${itemCount} item${itemCount === 1 ? "" : "s"}...`));
    await nextFrame();
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
      elements.itemListStatus.textContent = "";
      return;
    }
    const table = document.createElement("table");
    table.append(itemTableHead());
    const body = document.createElement("tbody");
    for (const item of state.items) {
      const row = document.createElement("tr");
      const titleCell = document.createElement("td");
      titleCell.textContent = item.title || "Empty";
      row.append(titleCell, signerCell(item));
      row.insertBefore(taxonomyCell(item), row.children[2]);
      row.append(stateCell(item));
      const actions = document.createElement("td");
      actions.className = "actions-cell";
      const actionGroup = document.createElement("div");
      actionGroup.className = "row-actions";
      actionGroup.append(
        iconButton("Edit item", "edit", () => loadItem(item.id)),
        iconButton("View history", "history", () => loadItem(item.id, true))
      );
      actions.append(actionGroup);
      row.append(actions);
      body.append(row);
    }
    table.append(body);
    elements.itemList.append(table);
    elements.itemListStatus.textContent = "";
  } catch (error) {
    if (error.status !== 401) {
      elements.itemList.replaceChildren(textNode("p", `Item list unavailable: ${error.message}`, "empty-state"));
      elements.itemListStatus.textContent = "Item list unavailable.";
    }
  } finally {
    elements.itemList.removeAttribute("aria-busy");
  }
}

const itemTableHead = () => {
  const head = document.createElement("thead");
  const row = document.createElement("tr");
  for (const column of [
    { label: "Title", key: "title" },
    { label: "Signer", key: "signerText" },
    { label: "Franchise / Product" },
    { label: "State" },
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

const signerCell = (item) => {
  const cell = document.createElement("td");
  cell.className = "signer-cell";
  const content = document.createElement("div");
  content.className = "signer-cell-content";
  const names = Array.isArray(item.signerNames) && item.signerNames.length ? item.signerNames : [item.signerText || item.signer];
  const ids = Array.isArray(item.signerIds) ? item.signerIds : [];
  names.forEach((name, index) => {
    const displayName = name || "Empty";
    const signerId = ids[index];
    if (signerId) {
      const signerButton = buttonNode(displayName, "inline-link review-egress-control", () => {
        openSignerManagement(signerId, displayName);
      });
      signerButton.disabled = Boolean(state.reviewMutationPending);
      content.append(signerButton);
    } else {
      content.append(textNode("span", displayName));
    }
  });
  cell.append(content);
  return cell;
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

const splitList = (value) =>
  [
    ...new Set(
      String(value || "")
        .split(",")
        .map((entry) => entry.trim())
        .filter(Boolean)
    ),
  ];

const signerCreditsFromLegacy = (signer) => {
  const displayName = String(signer || "").trim();
  return displayName ? [{ signer: { displayName } }] : [{ signer: { displayName: "" } }];
};

const profileValue = (credit, key) => credit?.signer?.[key] || credit?.[key] || "";

function renderSignerRows(credits = signerCreditsFromLegacy("")) {
  elements.signerRows.replaceChildren();
  const rows = credits.length ? credits : signerCreditsFromLegacy("");
  rows.forEach((credit, index) => elements.signerRows.append(signerRow(credit, index)));
}

function signerRow(credit, index) {
  const row = document.createElement("article");
  row.className = "signer-row";
  row.dataset.index = String(index);
  row.setAttribute("aria-label", `Signer row ${index + 1}`);
  if (credit?.signer?.id) {
    row.dataset.signerId = credit.signer.id;
  }

  const title = textNode("h4", `Signer ${index + 1}`);
  const hint = textNode("p", copy.signerCreate, "helper-text");
  row.append(title, hint);

  const grid = document.createElement("div");
  grid.className = "signer-row-grid";
  grid.append(
    labeledInput(`signer-name-${index}`, "Signer name", "text", profileValue(credit, "displayName"), {
      className: "signer-name-input",
      field: "name",
      list: "signer-suggestions",
      required: true,
    }),
    labeledInput(`signer-role-${index}`, "Role", "text", credit?.itemRole || credit?.item_role || "", {
      field: "role",
    }),
    labeledInput(`signer-context-${index}`, "Context", "text", credit?.itemContext || credit?.item_context || "", {
      field: "context",
    })
  );
  row.append(grid);

  const actions = document.createElement("div");
  actions.className = "inline-actions";
  actions.append(
    buttonNode("Manage profile", "secondary-action signer-manage-action", () => {
      const displayName = row.querySelector('[data-signer-field="name"]')?.value.trim() || "";
      openSignerManagement(row.dataset.signerId, displayName);
    }),
    buttonNode("Remove signer", "secondary-action", () => {
      row.remove();
      normalizeSignerRowHeadings();
      markDirty();
    })
  );
  row.append(actions);
  setExistingSignerProfileControls(row);

  const nameInput = grid.querySelector(".signer-name-input");
  let selectedSignerName = profileValue(credit, "displayName").trim();
  nameInput.addEventListener("input", async () => {
    if (nameInput.value.trim() !== selectedSignerName) {
      delete row.dataset.signerId;
      setExistingSignerProfileControls(row);
    }
    await loadSignerSuggestions(nameInput.value);
    renderDuplicateWarnings();
  });
  nameInput.addEventListener("change", () => {
    const selected = state.signerSuggestions.find(
      (suggestion) => suggestion.profile.displayName === nameInput.value.trim()
    );
    if (selected?.profile?.id) {
      row.dataset.signerId = selected.profile.id;
      selectedSignerName = selected.profile.displayName;
    } else {
      delete row.dataset.signerId;
      selectedSignerName = "";
    }
    setExistingSignerProfileControls(row);
    renderDuplicateWarnings();
  });

  return row;
}

function setExistingSignerProfileControls(row) {
  const disabled = Boolean(row.dataset.signerId);
  const manage = row.querySelector(".signer-manage-action");
  if (manage) {
    manage.disabled = !disabled;
    manage.title = disabled ? "Manage reusable signer profile" : "Save or select an existing signer first";
  }
}

function labeledInput(id, labelText, type, value, options = {}) {
  const wrapper = document.createElement("div");
  wrapper.className = "field";
  const label = document.createElement("label");
  label.setAttribute("for", id);
  label.textContent = labelText;
  const input = document.createElement("input");
  input.id = id;
  input.name = id;
  input.type = type;
  input.value = value || "";
  if (options.className) {
    input.className = options.className;
  }
  if (options.field) {
    input.dataset.signerField = options.field;
  }
  if (options.list) {
    input.setAttribute("list", options.list);
  }
  if (options.required) {
    input.required = true;
  }
  wrapper.append(label, input);
  return wrapper;
}

function normalizeSignerRowHeadings() {
  Array.from(elements.signerRows.children).forEach((row, index) => {
    row.dataset.index = String(index);
    row.querySelector("h4").textContent = `Signer ${index + 1}`;
  });
}

async function loadSignerSuggestions(query) {
  const trimmed = String(query || "").trim();
  if (trimmed.length < 2) {
    state.signerSuggestions = [];
    renderSignerSuggestionDatalist();
    return [];
  }
  try {
    const result = await request(endpoints.signers(trimmed));
    state.signerSuggestions = Array.isArray(result.suggestions) ? result.suggestions : [];
    renderSignerSuggestionDatalist();
    return state.signerSuggestions;
  } catch (error) {
    if (error.status !== 401) {
      elements.signerWarningSummary.hidden = false;
      elements.signerWarningSummary.textContent = `Signer suggestions unavailable: ${error.message}`;
    }
    return [];
  }
}

function renderSignerSuggestionDatalist() {
  const list = $("#signer-suggestions");
  if (!list) {
    return;
  }
  list.replaceChildren(
    ...state.signerSuggestions.map((suggestion) => {
      const option = document.createElement("option");
      option.value = suggestion.profile.displayName;
      return option;
    })
  );
}

function renderDuplicateWarnings() {
  const warnings = [];
  for (const row of Array.from(elements.signerRows.children)) {
    const input = row.querySelector(".signer-name-input");
    const value = input?.value?.trim();
    if (!value) {
      continue;
    }
    const duplicate = state.signerSuggestions.find(
      (suggestion) => suggestion.possibleDuplicate && suggestion.profile.displayName.toLowerCase() !== value.toLowerCase()
    );
    if (duplicate) {
      warnings.push({ row, duplicate, value });
    }
  }

  elements.signerWarningSummary.replaceChildren();
  if (warnings.length === 0) {
    elements.signerWarningSummary.hidden = true;
    return;
  }
  elements.signerWarningSummary.hidden = false;
  elements.signerWarningSummary.append(textNode("p", copy.signerDuplicate));
  for (const warning of warnings) {
    const action = document.createElement("div");
    action.className = "inline-actions";
    action.append(
      textNode("span", `${warning.value} may match ${warning.duplicate.profile.displayName}.`),
      buttonNode("Use existing signer", "secondary-action", () => {
        const input = warning.row.querySelector(".signer-name-input");
        input.value = warning.duplicate.profile.displayName;
        warning.row.dataset.signerId = warning.duplicate.profile.id;
        setExistingSignerProfileControls(warning.row);
        renderDuplicateWarnings();
        markDirty();
      })
    );
    if (warning.row.dataset.signerId) {
      action.append(
        buttonNode("Merge signer profiles", "destructive", () =>
          mergeSignerProfiles(warning.row.dataset.signerId, warning.duplicate.profile.id)
        )
      );
    }
    elements.signerWarningSummary.append(action);
  }
}

async function loadTaxonomySuggestions() {
  try {
    state.taxonomySuggestions = await request(endpoints.taxonomySuggestions);
    renderTaxonomySuggestions();
  } catch (error) {
    if (error.status !== 401) {
      elements.globalMessage.textContent = `Taxonomy suggestions unavailable: ${error.message}`;
    }
  }
}

async function mergeSignerProfiles(sourceSignerId, targetSignerId) {
  if (!sourceSignerId || !targetSignerId || sourceSignerId === targetSignerId) {
    return;
  }
  if (!window.confirm(copy.mergeSigner)) {
    return;
  }
  try {
    const result = await jsonRequest(endpoints.signerMerge, "POST", {
      sourceSignerId,
      targetSignerId,
    });
    elements.signerMergePanel.hidden = false;
    elements.signerMergePanel.textContent = `Merged signer profiles and updated ${result.updatedItemCount || 0} item(s).`;
    await loadTaxonomySuggestions();
    if (state.currentItem?.id) {
      await loadItem(state.currentItem.id);
    }
  } catch (error) {
    if (error.status !== 401) {
      elements.signerMergePanel.hidden = false;
      elements.signerMergePanel.textContent = `Signer merge failed: ${error.message}`;
    }
  }
}

function openSignerManagement(signerId, displayName = "") {
  if (!signerId || !ensureSavedBeforeManagingSigner()) {
    return;
  }
  state.focusedSignerId = signerId;
  elements.signerManagementQuery.value = displayName;
  setView("signers-view");
}

async function renderSignerManagement() {
  const query = elements.signerManagementQuery.value.trim();
  elements.signerManagementRows.replaceChildren();
  if (query.length < 2) {
    state.managedSigners = [];
    elements.signerManagementMessage.textContent = "Search by at least 2 characters.";
    elements.signerManagementRows.append(textNode("p", "Search by signer name to edit a reusable profile.", "empty-state"));
    return;
  }

  elements.signerManagementMessage.textContent = "Loading signers...";
  elements.signerManagementRows.append(loadingState("Loading signers..."));
  try {
    const result = await request(endpoints.signers(query));
    state.managedSigners = (Array.isArray(result.suggestions) ? result.suggestions : []).map(
      (suggestion) => suggestion.profile
    );
    renderSignerManagementRows();
  } catch (error) {
    if (error.status !== 401) {
      elements.signerManagementMessage.textContent = `Signer profiles unavailable: ${error.message}`;
      elements.signerManagementRows.replaceChildren(
        textNode("p", "Signer profiles could not be loaded.", "empty-state")
      );
    }
  }
}

function renderSignerManagementRows() {
  elements.signerManagementRows.replaceChildren();
  if (state.managedSigners.length === 0) {
    elements.signerManagementMessage.textContent = "No matching signers.";
    elements.signerManagementRows.append(textNode("p", "No matching signers.", "empty-state"));
    return;
  }
  elements.signerManagementMessage.textContent = `${state.managedSigners.length} signer profile${
    state.managedSigners.length === 1 ? "" : "s"
  } found.`;
  for (const profile of state.managedSigners) {
    elements.signerManagementRows.append(signerManagementEditor(profile));
  }
  const focused = elements.signerManagementRows.querySelector(".signer-management-row.is-focused h3");
  if (focused) {
    focused.setAttribute("tabindex", "-1");
    focused.focus();
  }
}

function signerManagementEditor(profile) {
  const section = document.createElement("article");
  section.className = "signer-management-row";
  if (profile.id === state.focusedSignerId) {
    section.classList.add("is-focused");
  }

  const heading = document.createElement("div");
  heading.className = "panel-heading";
  heading.append(
    textNode("h3", profile.displayName || "Signer profile"),
    textNode("p", `Updated ${formatRelativeEpoch(profile.updatedAtEpochSeconds)}`, "helper-text")
  );
  if (profile.id === state.focusedSignerId) {
    heading.append(textNode("span", "Selected from item editor", "status-pill"));
  }
  section.append(heading);

  const form = document.createElement("form");
  form.className = "signer-management-form";
  form.append(
    managementInput(profile.id, "displayName", "Display name", "text", profile.displayName, true),
    managementInput(profile.id, "defaultRole", "Default role", "text", profile.defaultRole || ""),
    managementInput(profile.id, "wikipediaUrl", "Wikipedia short ID", "text", profile.wikipediaUrl || ""),
    managementInput(profile.id, "imdbUrl", "IMDb name ID", "text", profile.imdbUrl || "")
  );

  const actions = document.createElement("div");
  actions.className = "inline-actions full-width";
  const save = buttonNode("Save signer", "primary-action", () => saveSignerProfile(profile, form, save));
  actions.append(save);
  form.append(actions);
  form.addEventListener("submit", (event) => {
    event.preventDefault();
    saveSignerProfile(profile, form, save);
  });
  section.append(form);

  const linkedItems = document.createElement("div");
  linkedItems.className = "signer-linked-items full-width";
  linkedItems.setAttribute("aria-live", "polite");
  linkedItems.append(textNode("p", "Loading linked items...", "helper-text"));
  section.append(linkedItems);
  renderSignerLinkedItems(profile, linkedItems);
  return section;
}

async function renderSignerLinkedItems(profile, container) {
  try {
    const items = await request(`${endpoints.items}?signer=${encodeURIComponent(profile.displayName || "")}`);
    const linkedItems = (Array.isArray(items) ? items : []).filter((item) =>
      Array.isArray(item.signerIds) && item.signerIds.includes(profile.id)
    );
    container.replaceChildren();
    const heading = document.createElement("div");
    heading.className = "linked-items-heading";
    heading.append(
      textNode("h4", "Linked items"),
      textNode("span", `${linkedItems.length} item${linkedItems.length === 1 ? "" : "s"}`, "helper-text")
    );
    container.append(heading);
    if (linkedItems.length === 0) {
      container.append(textNode("p", "No items currently reference this signer profile.", "empty-state"));
      return;
    }
    const list = document.createElement("div");
    list.className = "linked-item-list";
    for (const item of linkedItems) {
      list.append(linkedItemRow(item));
    }
    container.append(list);
  } catch (error) {
    if (error.status !== 401) {
      container.replaceChildren(textNode("p", `Linked items unavailable: ${error.message}`, "empty-state"));
    }
  }
}

function linkedItemRow(item) {
  const row = document.createElement("div");
  row.className = "linked-item-row";
  const summary = document.createElement("div");
  summary.className = "linked-item-summary";
  summary.append(
    textNode("span", item.title || "Untitled item", "linked-item-title"),
    textNode(
      "span",
      [publicationStatusParts(item.publicationStatus).label, item.franchises?.join(", "), item.productLine]
        .filter(Boolean)
        .join(" · "),
      "helper-text"
    )
  );
  const actions = document.createElement("div");
  actions.className = "row-actions";
  actions.append(
    iconButton("Edit item", "edit", () => loadItem(item.id)),
    iconButton("View history", "history", () => loadItem(item.id, true))
  );
  row.append(summary, actions);
  return row;
}

function managementInput(profileId, field, labelText, type, value, required = false) {
  const id = `signer-management-${profileId}-${field}`;
  const wrapper = document.createElement("div");
  wrapper.className = "field";
  const label = document.createElement("label");
  label.setAttribute("for", id);
  label.textContent = labelText;
  const input = document.createElement("input");
  input.id = id;
  input.name = field;
  input.type = type;
  input.value = value || "";
  if (required) {
    input.required = true;
  }
  wrapper.append(label, input);
  return wrapper;
}

async function saveSignerProfile(profile, form, submit) {
  const value = (name) => form.elements[name].value.trim();
  if (!value("displayName")) {
    elements.signerManagementMessage.textContent = "Display name is required.";
    return;
  }
  submit.disabled = true;
  const originalText = submit.textContent;
  submit.textContent = "Saving...";
  try {
    const updated = await jsonRequest(endpoints.signer(profile.id), "PATCH", {
      displayName: value("displayName"),
      defaultRole: optionalValue(value("defaultRole")),
      wikipediaUrl: optionalValue(value("wikipediaUrl")),
      imdbUrl: optionalValue(value("imdbUrl")),
    });
    state.focusedSignerId = updated.id;
    elements.signerManagementQuery.value = updated.displayName;
    elements.signerManagementMessage.textContent = "Signer profile saved privately. Publish changes when ready.";
    await loadTaxonomySuggestions();
    await renderSignerManagement();
  } catch (error) {
    if (error.status !== 401) {
      elements.signerManagementMessage.textContent = `Signer profile save failed: ${error.message}`;
    }
  } finally {
    submit.disabled = false;
    submit.textContent = originalText;
  }
}

function renderTaxonomySuggestions() {
  const suggestions = state.taxonomySuggestions || {};
  fillDatalist("character-suggestions", suggestions.characters);
  fillDatalist("franchise-suggestions", suggestions.franchises);
  fillDatalist("product-line-suggestions", suggestions.productLines);
  fillDatalist("set-name-suggestions", suggestions.setNames);
}

function fillDatalist(id, values = []) {
  const list = $(`#${id}`);
  if (!list) {
    return;
  }
  list.replaceChildren(
    ...(Array.isArray(values) ? values : []).map((value) => {
      const option = document.createElement("option");
      option.value = value;
      return option;
    })
  );
}

function reconcileAdminItemResponse(item) {
  let reviewInvalidated = false;
  if (state.reviewSession) {
    const reviewedImage = item?.images?.find(
      (image) => image.id === state.reviewSession.imageId
    );
    if (
      !reviewedImage ||
      !state.reviewSession.mediaRevision ||
      reviewedImage.mediaRevision !== state.reviewSession.mediaRevision
    ) {
      clearImageReviewState();
      reviewInvalidated = true;
      elements.imageMessage.textContent = copy.mediaChanged;
    }
  }
  state.currentItem = item;
  return { item, reviewInvalidated };
}

function renderEditor(item = null) {
  if (state.reviewSession && state.reviewMutationPending) {
    blockReviewEgressWhileMutationPending();
    return false;
  }
  reconcileAdminItemResponse(item);
  state.dirty = false;
  elements.itemForm.reset();
  elements.discardUnsaved.hidden = true;
  elements.publishFromEditor.setAttribute("aria-disabled", "false");
  syncPublishAvailability();
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
    characters: Array.isArray(values.characters) ? values.characters.join(", ") : "",
    format: values.format || "Trading Card",
    language: values.language || "English",
    franchises: Array.isArray(values.franchises) ? values.franchises.join(", ") : "",
    productLine: values.productLine || "",
    setName: values.setName || "",
    customItem: values.origin === "Custom",
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
      if (elements.itemForm.elements[name].type === "checkbox") {
        elements.itemForm.elements[name].checked = Boolean(value);
      } else {
        elements.itemForm.elements[name].value = value;
      }
    }
  }
  renderSignerRows(values.signerCredits || signerCreditsFromLegacy(values.signer));
  renderDuplicateWarnings();
  renderTaxonomySuggestions();
  renderImages(values.images || [], values.cleanupWarnings || []);
  renderHistory(item?.id);
  setView("add-item-view");
  return true;
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
      renderImagePreviewFrame(state.currentItem.id, image),
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
      buttonNode("Review image", "primary-action", () => openImageReview(image.id)),
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

function renderImagePreviewFrame(itemId, image) {
  const frame = document.createElement("div");
  frame.className = "preview-frame review-matte";
  let attempt = 0;
  const loadPreview = () => {
    const preview = document.createElement("img");
    const previewUrl = endpoints.imagePreview(itemId, image.id, image.mediaRevision);
    const separator = previewUrl.includes("?") ? "&" : "?";
    preview.src = `${previewUrl}${separator}retry=${attempt}`;
    preview.alt = image.altText || "Private autograph image preview";
    preview.loading = "lazy";
    preview.addEventListener("error", () => {
      const failure = document.createElement("div");
      failure.className = "preview-failure";
      failure.append(
        textNode("p", copy.previewError, "status-warning"),
        buttonNode("Retry preview", "secondary-action", () => {
          attempt += 1;
          loadPreview();
        })
      );
      frame.replaceChildren(failure);
    });
    frame.replaceChildren(preview);
  };
  loadPreview();
  return frame;
}

const identityReviewAdjustment = () => ({
  rotationDegrees: 0,
  zoom: 1,
  panX: 0,
  panY: 0,
  crop: null,
  perspective: null,
});

const fullFramePerspectiveCorners = () => [
  { x: 0, y: 0 },
  { x: 1, y: 0 },
  { x: 1, y: 1 },
  { x: 0, y: 1 },
];

const cloneAdjustment = (adjustment) =>
  JSON.parse(JSON.stringify(adjustment || identityReviewAdjustment()));

const canonicalReviewAdjustment = (adjustment) => {
  const canonical = cloneAdjustment(adjustment);
  const corners = canonical.perspective?.corners;
  if (
    Array.isArray(corners) &&
    corners.length === 4 &&
    corners.every((corner, index) => {
      const fullFrame = fullFramePerspectiveCorners()[index];
      return corner.x === fullFrame.x && corner.y === fullFrame.y;
    })
  ) {
    canonical.perspective = null;
  }
  return canonical;
};

const reviewAdjustmentsEqual = (left, right) =>
  JSON.stringify(canonicalReviewAdjustment(left)) === JSON.stringify(canonicalReviewAdjustment(right));

const currentPerspectiveCorners = () =>
  cloneAdjustment(
    state.reviewDraftAdjustment?.perspective?.corners || fullFramePerspectiveCorners()
  );

function syncPublishAvailability() {
  const disabled =
    state.dirty ||
    state.reviewDirty ||
    Boolean(state.reviewMutationPending) ||
    Boolean(state.reviewPerspectiveDrag);
  for (const button of [elements.publishFromEditor, elements.publishIncremental, elements.publishFull]) {
    if (!button) {
      continue;
    }
    button.disabled = disabled;
    button.setAttribute("aria-disabled", disabled ? "true" : "false");
  }
}

function syncReviewDirtyState() {
  state.reviewDirty = Boolean(
    state.reviewDraftAdjustment &&
      !reviewAdjustmentsEqual(state.reviewDraftAdjustment, state.reviewSavedAdjustment)
  );
  elements.imageReviewDirtyBand.hidden = !state.reviewDirty;
  const latestPreviewIsDisplayed =
    state.reviewPreviewStatus === "ready" &&
    state.reviewDisplayedRevision === state.reviewDraftRevision;
  elements.imageReviewSave.disabled =
    !state.reviewDirty ||
    !latestPreviewIsDisplayed ||
    Boolean(state.reviewMutationPending) ||
    Boolean(state.reviewPerspectiveDrag);
  const mutationPending = Boolean(state.reviewMutationPending);
  const interactionBlocked = mutationPending || Boolean(state.reviewPerspectiveDrag);
  elements.imageReviewReset.disabled = interactionBlocked;
  elements.imageReviewDiscard.disabled = interactionBlocked;
  elements.logout.disabled = interactionBlocked;
  for (const tab of elements.tabs) {
    tab.disabled = interactionBlocked;
  }
  for (const control of document.querySelectorAll(".review-egress-control")) {
    control.disabled = interactionBlocked;
  }
  $("#image-review-detect").disabled = interactionBlocked;
  for (const control of document.querySelectorAll("[data-adjustment-field]")) {
    control.disabled = interactionBlocked;
  }
  for (const toggle of document.querySelectorAll("[data-review-overlay]")) {
    toggle.disabled = interactionBlocked;
  }
  syncReviewComparisonButtons();
  syncPublishAvailability();
}

function pendingMutationMessage() {
  const operation = state.reviewMutationPending?.operation === "reset" ? "Reset" : "Save";
  return `${operation} is still in progress. Keep this review open until it finishes.`;
}

function blockReviewEgressWhileMutationPending() {
  if (!state.reviewMutationPending && !state.reviewPerspectiveDrag) {
    return false;
  }
  state.reviewMessage = state.reviewMutationPending
    ? pendingMutationMessage()
    : "Finish positioning the perspective corner before leaving this review.";
  elements.imageReviewMessage.textContent = state.reviewMessage;
  if (typeof elements.imageReviewMessage.focus === "function") {
    elements.imageReviewMessage.focus();
  }
  return true;
}

function beginReviewMutation(operation, session, itemId, imageId) {
  state.reviewMutationRevision += 1;
  state.reviewMutationPending = {
    operation,
    token: state.reviewMutationRevision,
    session,
    itemId,
    imageId,
  };
  syncReviewDirtyState();
  return state.reviewMutationPending;
}

function isCurrentReviewMutation(mutation) {
  return Boolean(
    mutation &&
      state.reviewMutationPending === mutation &&
      isCurrentReviewSession(mutation.session) &&
      state.reviewImage?.itemId === mutation.itemId &&
      state.reviewImage?.imageId === mutation.imageId
  );
}

function invalidateReviewOutputRender() {
  state.reviewOutputRenderGeneration += 1;
  state.reviewMountedOutput = null;
}

function disconnectPerspectiveProjection({ terminateDrag = true } = {}) {
  if (terminateDrag) {
    finishPerspectiveDrag({ settle: false, reason: "teardown" });
  }
  state.reviewPerspectiveGeneration += 1;
  state.reviewPerspectiveFrame = null;
  state.reviewPerspectiveImage = null;
  state.reviewSourceProjectionStatus = "idle";
  if (state.reviewPerspectiveObserver) {
    state.reviewPerspectiveObserver.disconnect();
    state.reviewPerspectiveObserver = null;
  }
  if (state.reviewPerspectiveResizeListener) {
    window.removeEventListener("resize", state.reviewPerspectiveResizeListener);
    state.reviewPerspectiveResizeListener = null;
  }
}

function revokeReviewPreviewUrl() {
  if (state.reviewPreviewUrl) {
    invalidateReviewOutputRender();
    URL.revokeObjectURL(state.reviewPreviewUrl);
    state.reviewPreviewUrl = null;
  }
}

function cancelReviewPreviewRequest() {
  if (state.reviewPreviewTimer) {
    window.clearTimeout(state.reviewPreviewTimer);
    state.reviewPreviewTimer = null;
  }
  if (state.reviewPreviewAbortController) {
    state.reviewPreviewAbortController.abort();
    state.reviewPreviewAbortController = null;
  }
}

function clearImageReviewState() {
  invalidateReviewOutputRender();
  disconnectPerspectiveProjection();
  cancelReviewPreviewRequest();
  revokeReviewPreviewUrl();
  state.reviewImage = null;
  state.reviewDraftAdjustment = null;
  state.reviewSavedAdjustment = null;
  state.reviewDirty = false;
  state.reviewPreviewStatus = "idle";
  state.reviewDisplayedRevision = null;
  state.reviewDraftRevision = 0;
  state.reviewFocusedCornerIndex = null;
  state.reviewMessage = "";
  state.reviewMutationPending = null;
  state.reviewStageRenderDeferred = false;
  state.reviewOutputPanel = null;
  state.reviewSessionRevision += 1;
  state.reviewSession = null;
  state.reviewPreviewRevision += 1;
  syncPublishAvailability();
}

function beginReviewSession(itemId, imageId, mediaRevision = null) {
  invalidateReviewOutputRender();
  disconnectPerspectiveProjection();
  cancelReviewPreviewRequest();
  revokeReviewPreviewUrl();
  state.reviewStageRenderDeferred = false;
  state.reviewSessionRevision += 1;
  state.reviewSession = { revision: state.reviewSessionRevision, itemId, imageId, mediaRevision };
  return { ...state.reviewSession };
}

function isCurrentReviewSession(session) {
  return Boolean(
    session &&
      state.reviewSession &&
      session.revision === state.reviewSession.revision &&
      session.itemId === state.reviewSession.itemId &&
      session.imageId === state.reviewSession.imageId &&
      session.mediaRevision === state.reviewSession.mediaRevision
  );
}

function handleMediaRevisionConflict(error) {
  if (error?.status !== 409 || error?.body?.code !== "mediaRevisionConflict") {
    return false;
  }
  clearImageReviewState();
  elements.imageMessage.textContent = copy.mediaChanged;
  setView("add-item-view");
  if (state.currentItem) {
    renderImages(state.currentItem.images || [], state.currentItem.cleanupWarnings || []);
  }
  return true;
}

function markReviewDraftChanged({ schedulePreview = true } = {}) {
  state.reviewDraftRevision += 1;
  state.reviewDisplayedRevision = null;
  syncReviewDirtyState();
  if (schedulePreview) {
    scheduleDraftPreview();
  }
}

function scheduleDraftPreview({ immediate = false, renderStage = true } = {}) {
  cancelReviewPreviewRequest();
  state.reviewPreviewStatus = "loading";
  state.reviewDisplayedRevision = null;
  const revision = state.reviewPreviewRevision + 1;
  state.reviewPreviewRevision = revision;
  const draftRevision = state.reviewDraftRevision;
  const session = state.reviewSession ? { ...state.reviewSession } : null;
  state.reviewPreviewTimer = window.setTimeout(
    () => loadDraftPreview(revision, draftRevision, session),
    immediate ? 0 : 150
  );
  if (renderStage) {
    renderImageReview();
  }
}

async function loadDraftPreview(revision, draftRevision, session) {
  const { itemId, imageId } = state.reviewImage || {};
  if (
    !itemId ||
    !imageId ||
    !state.reviewDraftAdjustment ||
    !isCurrentReviewSession(session) ||
    draftRevision !== state.reviewDraftRevision
  ) {
    return;
  }
  const draftPreviewUrl = endpoints.imageDraftPreview(itemId, imageId);
  const controller = new AbortController();
  state.reviewPreviewAbortController = controller;
  const submittedAdjustment = canonicalReviewAdjustment(state.reviewDraftAdjustment);
  try {
    const response = await fetch(draftPreviewUrl, {
      method: "POST",
      credentials: "same-origin",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        mediaRevision: session.mediaRevision,
        adjustment: submittedAdjustment,
      }),
      signal: controller.signal,
    });
    if (response.status === 401) {
      handleAuthFailure();
      throw new Error(copy.sessionExpired);
    }
    if (!response.ok) {
      const contentType = response.headers.get("content-type") || "";
      const body = contentType.includes("application/json") ? await response.json() : null;
      const error = new Error(copy.previewError);
      error.status = response.status;
      error.body = body;
      throw error;
    }
    const blob = await response.blob();
    if (
      revision !== state.reviewPreviewRevision ||
      draftRevision !== state.reviewDraftRevision ||
      !isCurrentReviewSession(session)
    ) {
      return;
    }
    const nextUrl = URL.createObjectURL(blob);
    revokeReviewPreviewUrl();
    state.reviewPreviewUrl = nextUrl;
    state.reviewPreviewStatus = "rendering";
    state.reviewDisplayedRevision = null;
    renderImageReview();
  } catch (error) {
    if (handleMediaRevisionConflict(error)) {
      return;
    }
    if (
      error.name === "AbortError" ||
      revision !== state.reviewPreviewRevision ||
      draftRevision !== state.reviewDraftRevision ||
      !isCurrentReviewSession(session)
    ) {
      return;
    }
    state.reviewPreviewStatus = "error";
    state.reviewDisplayedRevision = null;
    state.reviewMessage = copy.previewError;
    renderImageReview();
  } finally {
    if (state.reviewPreviewAbortController === controller) {
      state.reviewPreviewAbortController = null;
    }
  }
}

async function openImageReview(imageId) {
  if (!state.currentItem?.id || !ensureSavedBeforeImageChange()) {
    return;
  }
  const itemId = state.currentItem.id;
  const itemImage = state.currentItem.images?.find((image) => image.id === imageId);
  if (!itemImage?.mediaRevision) {
    elements.imageMessage.textContent = copy.previewError;
    return;
  }
  const session = beginReviewSession(itemId, imageId, itemImage.mediaRevision);
  try {
    const review = await request(endpoints.imageReview(itemId, imageId));
    if (!isCurrentReviewSession(session) || state.currentItem?.id !== itemId) {
      return;
    }
    if (review.mediaRevision !== session.mediaRevision) {
      handleMediaRevisionConflict({
        status: 409,
        body: { code: "mediaRevisionConflict" },
      });
      return;
    }
    state.reviewImage = review;
    state.reviewSavedAdjustment = canonicalReviewAdjustment(review.adjustment);
    state.reviewDraftAdjustment = canonicalReviewAdjustment(review.adjustment);
    state.reviewDirty = false;
    state.reviewComparisonMode = "latest";
    state.reviewOverlays = { grid: false, centerline: false, edges: true };
    state.reviewPreviewStatus = "loading";
    state.reviewDisplayedRevision = null;
    state.reviewDraftRevision = 0;
    state.reviewMutationPending = null;
    state.reviewFocusedCornerIndex = null;
    state.reviewMessage = review.message || copy.privateOnly;
    setView("image-review-view");
    renderImageReview();
    scheduleDraftPreview({ immediate: true });
  } catch (error) {
    if (handleMediaRevisionConflict(error)) {
      return;
    }
    if (isCurrentReviewSession(session) && error.status !== 401) {
      elements.imageMessage.textContent = copy.previewError;
    }
  }
}

function renderImageReview() {
  if (state.reviewPerspectiveDrag && !state.reviewPerspectiveDrag.terminal) {
    state.reviewStageRenderDeferred = true;
    return false;
  }
  state.reviewStageRenderDeferred = false;
  renderImageReviewNow();
  return true;
}

function renderImageReviewNow() {
  invalidateReviewOutputRender();
  disconnectPerspectiveProjection();
  elements.imageReviewStage.replaceChildren();
  state.reviewOutputPanel = null;
  if (!state.reviewImage || !state.reviewDraftAdjustment) {
    elements.imageReviewStage.append(textNode("p", copy.previewError, "status-warning"));
    return;
  }
  const workspace = document.createElement("div");
  workspace.className = "review-preview-workspace";

  const sourcePanel = document.createElement("section");
  sourcePanel.className = "review-preview-panel";
  sourcePanel.append(textNode("h3", "Unadjusted source guide"));
  const sourceFrame = document.createElement("div");
  sourceFrame.className = "source-guide-frame review-matte";
  const sourceImage = document.createElement("img");
  sourceImage.src =
    state.reviewImage.sourceGuidePreviewUrl ||
    endpoints.imageSourcePreview(state.reviewImage.itemId, state.reviewImage.imageId);
  sourceImage.alt = "Sanitized unadjusted source used for perspective coordinates";
  const perspectiveGeneration = state.reviewPerspectiveGeneration;
  state.reviewSourceProjectionStatus = "loading";
  state.reviewPerspectiveFrame = sourceFrame;
  state.reviewPerspectiveImage = sourceImage;
  sourceImage.addEventListener("load", () => {
    if (isCurrentPerspectiveProjection(perspectiveGeneration, sourceFrame, sourceImage)) {
      const bounds = sourceRenderedBounds(sourceFrame, sourceImage);
      if (!sourceImage.naturalWidth || !sourceImage.naturalHeight || !bounds.width || !bounds.height) {
        return;
      }
      state.reviewSourceProjectionStatus = "ready";
      rebasePerspectiveDrag(sourceFrame, sourceImage);
      syncPerspectiveHandleAvailability(sourceFrame);
      projectPerspectiveHandles();
    }
  });
  sourceImage.addEventListener("error", () => {
    if (isCurrentPerspectiveProjection(perspectiveGeneration, sourceFrame, sourceImage)) {
      const terminal = finishPerspectiveDrag({
        settle: true,
        reason: "source-error",
        renderStage: false,
      });
      disconnectPerspectiveProjection({ terminateDrag: false });
      state.reviewSourceProjectionStatus = "error";
      sourceFrame.replaceChildren(textNode("p", copy.previewError, "status-warning"));
      if (terminal?.authoritative && !terminal.moved && terminal.deferredRender) {
        renderReviewOutputPanel(state.reviewOutputPanel);
      }
    }
  });
  sourceFrame.append(sourceImage);
  renderPerspectiveHandles(sourceFrame, sourceImage);
  observePerspectiveProjection(perspectiveGeneration, sourceFrame, sourceImage);
  sourcePanel.append(sourceFrame);

  const outputPanel = document.createElement("section");
  outputPanel.className = "review-preview-panel";
  state.reviewOutputPanel = outputPanel;
  renderReviewOutputPanel(outputPanel, { invalidate: false });
  workspace.append(sourcePanel, outputPanel);
  elements.imageReviewStage.append(workspace);
  elements.imageReviewMessage.textContent = state.reviewMessage;
  syncReviewControls();
  syncReviewDirtyState();
  syncReviewComparisonButtons();
}

function renderReviewOutputPanel(outputPanel, { invalidate = true } = {}) {
  if (!outputPanel) {
    return;
  }
  if (invalidate) {
    invalidateReviewOutputRender();
  }
  outputPanel.replaceChildren();
  outputPanel.append(textNode("h3", "Adjusted output preview"));
  if (state.reviewPreviewStatus === "error") {
    const failure = document.createElement("div");
    failure.className = "preview-failure";
    const retry = buttonNode("Retry preview", "secondary-action", () => {
      if (!state.reviewPerspectiveDrag) {
        scheduleDraftPreview({ immediate: true });
      }
    });
    failure.append(
      textNode("p", copy.previewError, "status-warning"),
      retry
    );
    outputPanel.append(failure);
  } else if (!state.reviewPreviewUrl || state.reviewPreviewStatus === "loading") {
    outputPanel.append(loadingState("Rendering the complete draft adjustment..."));
  } else {
    const frame = document.createElement("div");
    frame.className = `review-frame review-comparison-${state.reviewComparisonMode}`;
    const displayedRevision = state.reviewDraftRevision;
    const displayedSession = state.reviewSession ? { ...state.reviewSession } : null;
    const displayedPreviewRevision = state.reviewPreviewRevision;
    const displayedPreviewUrl = state.reviewPreviewUrl;
    const renderGeneration = state.reviewOutputRenderGeneration;
    state.reviewPreviewStatus = "rendering";
    state.reviewDisplayedRevision = null;
    const latestImage = reviewImageNode(
      displayedPreviewUrl,
      "Latest private image under review",
      "review-image-latest",
      () => {
        if (isAuthoritativeOutputRender({
          session: displayedSession,
          draftRevision: displayedRevision,
          previewRevision: displayedPreviewRevision,
          previewUrl: displayedPreviewUrl,
          generation: renderGeneration,
          node: latestImage,
        })) {
          state.reviewPreviewStatus = "ready";
          state.reviewDisplayedRevision = displayedRevision;
          syncReviewDirtyState();
        }
      },
      () => {
        if (isAuthoritativeOutputRender({
          session: displayedSession,
          draftRevision: displayedRevision,
          previewRevision: displayedPreviewRevision,
          previewUrl: displayedPreviewUrl,
          generation: renderGeneration,
          node: latestImage,
        })) {
          state.reviewPreviewStatus = "error";
          state.reviewDisplayedRevision = null;
          state.reviewMessage = copy.previewError;
          renderImageReview();
        }
      }
    );
    state.reviewMountedOutput = {
      session: displayedSession,
      itemId: state.reviewImage.itemId,
      imageId: state.reviewImage.imageId,
      draftRevision: displayedRevision,
      previewRevision: displayedPreviewRevision,
      previewUrl: displayedPreviewUrl,
      generation: renderGeneration,
      node: latestImage,
    };
    const publicUrl = state.reviewImage.publicCurrentPreviewUrl;
    if (state.reviewComparisonMode === "before-after" && publicUrl) {
      const pair = document.createElement("div");
      pair.className = "before-after-comparison";
      pair.append(
        reviewComparisonPane(
          reviewImageNode(publicUrl, "Current public image", "review-image-public"),
          "Public current"
        ),
        reviewComparisonPane(latestImage, "Private latest")
      );
      frame.append(pair);
    } else if (state.reviewComparisonMode === "split" && publicUrl) {
      frame.classList.add("split-comparison");
      frame.append(
        reviewImageNode(publicUrl, "Current public image", "review-image-public"),
        latestImage
      );
    } else {
      frame.append(latestImage);
    }
    for (const [overlay, enabled] of Object.entries(state.reviewOverlays)) {
      if (enabled) {
        const layer = document.createElement("div");
        layer.className = `review-overlay review-overlay-${overlay}`;
        layer.setAttribute("aria-hidden", "true");
        frame.append(layer);
      }
    }
    outputPanel.append(frame);
  }
  elements.imageReviewMessage.textContent = state.reviewMessage;
  syncReviewDirtyState();
}

function isAuthoritativeOutputRender(candidate) {
  const mounted = state.reviewMountedOutput;
  return Boolean(
    mounted &&
      mounted.node === candidate.node &&
      candidate.node.parentNode &&
      mounted.generation === candidate.generation &&
      state.reviewOutputRenderGeneration === candidate.generation &&
      mounted.previewRevision === candidate.previewRevision &&
      state.reviewPreviewRevision === candidate.previewRevision &&
      mounted.previewUrl === candidate.previewUrl &&
      state.reviewPreviewUrl === candidate.previewUrl &&
      mounted.draftRevision === candidate.draftRevision &&
      state.reviewDraftRevision === candidate.draftRevision &&
      mounted.itemId === state.reviewImage?.itemId &&
      mounted.imageId === state.reviewImage?.imageId &&
      isCurrentReviewSession(candidate.session) &&
      state.reviewPreviewStatus === "rendering"
  );
}

function reviewImageNode(src, alt, className, onLoad = null, onError = null) {
  const image = document.createElement("img");
  image.src = src;
  image.alt = alt;
  image.className = className;
  if (onLoad) {
    image.addEventListener("load", onLoad);
  }
  if (onError) {
    image.addEventListener("error", onError);
  }
  return image;
}

function reviewComparisonPane(image, label) {
  const pane = document.createElement("figure");
  pane.className = "comparison-pane";
  const caption = document.createElement("figcaption");
  caption.textContent = label;
  pane.append(image, caption);
  return pane;
}

function renderPerspectiveHandles(frame, sourceImage) {
  for (const existing of [...frame.children].filter((child) => child.className === "corner-handle")) {
    existing.remove();
  }
  const labels = ["Top left corner", "Top right corner", "Bottom right corner", "Bottom left corner"];
  const corners = currentPerspectiveCorners();
  corners.forEach((corner, index) => {
    const handle = document.createElement("button");
    handle.type = "button";
    handle.className = "corner-handle";
    handle.setAttribute("aria-label", labels[index]);
    handle.title = labels[index];
    handle.disabled =
      Boolean(state.reviewMutationPending) || state.reviewSourceProjectionStatus !== "ready";
    handle.sourceFrame = frame;
    handle.sourceImage = sourceImage;
    positionPerspectiveHandle(handle, corner, frame, sourceImage);
    handle.addEventListener("keydown", (event) => movePerspectiveHandle(index, event));
    handle.addEventListener("pointerdown", (event) =>
      beginPerspectiveDrag(index, event, frame, sourceImage)
    );
    frame.append(handle);
    if (state.reviewFocusedCornerIndex === index) {
      requestAnimationFrame(() => handle.focus());
    }
  });
}

function syncPerspectiveHandleAvailability(frame = state.reviewPerspectiveFrame) {
  if (!frame) {
    return;
  }
  const disabled =
    Boolean(state.reviewMutationPending) || state.reviewSourceProjectionStatus !== "ready";
  for (const handle of [...frame.children].filter((child) => child.className === "corner-handle")) {
    handle.disabled = disabled;
    handle.setAttribute("aria-disabled", disabled ? "true" : "false");
  }
}

function isCurrentPerspectiveProjection(generation, frame, sourceImage) {
  return Boolean(
    state.reviewSession &&
      generation === state.reviewPerspectiveGeneration &&
      state.reviewPerspectiveFrame === frame &&
      state.reviewPerspectiveImage === sourceImage &&
      frame.parentNode
  );
}

function observePerspectiveProjection(generation, frame, sourceImage) {
  const project = () => {
    if (isCurrentPerspectiveProjection(generation, frame, sourceImage)) {
      rebasePerspectiveDrag(frame, sourceImage);
      projectPerspectiveHandles();
    }
  };
  if (typeof ResizeObserver === "function") {
    const observer = new ResizeObserver(project);
    state.reviewPerspectiveObserver = observer;
    observer.observe(frame);
  } else {
    state.reviewPerspectiveResizeListener = project;
    window.addEventListener("resize", project);
  }
}

function projectPerspectiveHandles() {
  const frame = state.reviewPerspectiveFrame;
  const sourceImage = state.reviewPerspectiveImage;
  if (!frame || !sourceImage) {
    return;
  }
  const handles = [...frame.children].filter((child) => child.className === "corner-handle");
  const corners = currentPerspectiveCorners();
  handles.forEach((handle, index) => {
    if (corners[index]) {
      positionPerspectiveHandle(handle, corners[index], frame, sourceImage);
    }
  });
}

function movePerspectiveHandle(index, event) {
  if (
    state.reviewMutationPending ||
    state.reviewPerspectiveDrag ||
    state.reviewSourceProjectionStatus !== "ready"
  ) {
    return;
  }
  const delta = event.shiftKey ? 0.05 : 0.01;
  const direction = {
    ArrowLeft: [-delta, 0],
    ArrowRight: [delta, 0],
    ArrowUp: [0, -delta],
    ArrowDown: [0, delta],
  }[event.key];
  if (!direction) {
    return;
  }
  event.preventDefault();
  const corners = currentPerspectiveCorners();
  const corner = corners[index];
  state.reviewFocusedCornerIndex = index;
  setPerspectiveCorner(
    index,
    corner.x + direction[0],
    corner.y + direction[1],
    event.currentTarget,
    event.currentTarget.sourceFrame,
    event.currentTarget.sourceImage
  );
}

function sourceRenderedBounds(frame, sourceImage) {
  const frameBounds = frame.getBoundingClientRect();
  const sourceWidth = Number(sourceImage?.naturalWidth || 0);
  const sourceHeight = Number(sourceImage?.naturalHeight || 0);
  if (!sourceWidth || !sourceHeight || !frameBounds.width || !frameBounds.height) {
    return frameBounds;
  }
  const scale = Math.min(frameBounds.width / sourceWidth, frameBounds.height / sourceHeight);
  const width = sourceWidth * scale;
  const height = sourceHeight * scale;
  return {
    left: frameBounds.left + (frameBounds.width - width) / 2,
    top: frameBounds.top + (frameBounds.height - height) / 2,
    width,
    height,
  };
}

function normalizedSourcePoint(frame, sourceImage, clientX, clientY) {
  const bounds = sourceRenderedBounds(frame, sourceImage);
  if (
    state.reviewSourceProjectionStatus !== "ready" ||
    !Number.isFinite(clientX) ||
    !Number.isFinite(clientY) ||
    !sourceImage?.naturalWidth ||
    !sourceImage?.naturalHeight ||
    !bounds.width ||
    !bounds.height
  ) {
    return null;
  }
  return {
    x: (clientX - bounds.left) / bounds.width,
    y: (clientY - bounds.top) / bounds.height,
  };
}

function rebasePerspectiveDrag(frame, sourceImage) {
  const drag = state.reviewPerspectiveDrag;
  if (
    !drag ||
    drag.terminal ||
    drag.frame !== frame ||
    drag.sourceImage !== sourceImage ||
    !drag.lastPointer
  ) {
    return;
  }
  const pointer = normalizedSourcePoint(
    frame,
    sourceImage,
    drag.lastPointer.clientX,
    drag.lastPointer.clientY
  );
  const corner = currentPerspectiveCorners()[drag.index];
  if (pointer && corner) {
    drag.grabOffset = { x: corner.x - pointer.x, y: corner.y - pointer.y };
  }
}

function beginPerspectiveDrag(index, event, frame, sourceImage) {
  const pointer = normalizedSourcePoint(frame, sourceImage, event.clientX, event.clientY);
  if (
    state.reviewMutationPending ||
    state.reviewPerspectiveDrag ||
    state.reviewSourceProjectionStatus !== "ready" ||
    !pointer
  ) {
    return;
  }
  event.preventDefault();
  const handle = event.currentTarget;
  const session = state.reviewSession ? { ...state.reviewSession } : null;
  const generation = state.reviewPerspectiveGeneration;
  const itemId = state.reviewImage?.itemId;
  const imageId = state.reviewImage?.imageId;
  const corner = currentPerspectiveCorners()[index];
  state.reviewFocusedCornerIndex = index;
  handle.setPointerCapture(event.pointerId);
  const move = (moveEvent) => {
    const drag = state.reviewPerspectiveDrag;
    if (
      !drag ||
      drag.handle !== handle ||
      moveEvent.pointerId !== drag.pointerId ||
      state.reviewMutationPending ||
      generation !== state.reviewPerspectiveGeneration ||
      !isCurrentReviewSession(session) ||
      state.reviewImage?.itemId !== itemId ||
      state.reviewImage?.imageId !== imageId ||
      state.reviewPerspectiveFrame !== frame ||
      state.reviewPerspectiveImage !== sourceImage ||
      handle.parentNode !== frame ||
      !frame.parentNode
    ) {
      finishPerspectiveDrag({ settle: false, reason: "invalidated" });
      return;
    }
    const currentPointer = normalizedSourcePoint(
      frame,
      sourceImage,
      moveEvent.clientX,
      moveEvent.clientY
    );
    if (!currentPointer) {
      finishPerspectiveDrag({ settle: false, reason: "invalidated" });
      return;
    }
    drag.lastPointer = { clientX: moveEvent.clientX, clientY: moveEvent.clientY };
    const changed = setPerspectiveCorner(
      index,
      currentPointer.x + drag.grabOffset.x,
      currentPointer.y + drag.grabOffset.y,
      handle,
      frame,
      sourceImage,
      { schedulePreview: false }
    );
    drag.moved = drag.moved || changed;
  };
  const finish = (finishEvent) => {
    if (finishEvent?.pointerId !== undefined && finishEvent.pointerId !== event.pointerId) {
      return;
    }
    finishPerspectiveDrag({
      settle: true,
      reason: finishEvent?.type || "pointer-terminal",
      releaseCapture: finishEvent?.type !== "lostpointercapture",
    });
  };
  state.reviewPerspectiveDrag = {
    generation,
    session,
    itemId,
    imageId,
    index,
    pointerId: event.pointerId,
    handle,
    frame,
    sourceImage,
    messageAtStart: state.reviewMessage,
    grabOffset: { x: corner.x - pointer.x, y: corner.y - pointer.y },
    lastPointer: { clientX: event.clientX, clientY: event.clientY },
    move,
    finish,
    moved: false,
    terminal: false,
  };
  handle.addEventListener("pointermove", move);
  handle.addEventListener("pointerup", finish);
  handle.addEventListener("pointercancel", finish);
  handle.addEventListener("lostpointercapture", finish);
  syncReviewDirtyState();
}

function finishPerspectiveDrag({
  settle = false,
  reason = "teardown",
  renderStage = true,
  releaseCapture = true,
} = {}) {
  const drag = state.reviewPerspectiveDrag;
  if (!drag || drag.terminal) {
    return null;
  }
  drag.terminal = true;
  drag.terminalReason = reason;
  drag.handle.removeEventListener("pointermove", drag.move);
  drag.handle.removeEventListener("pointerup", drag.finish);
  drag.handle.removeEventListener("pointercancel", drag.finish);
  if (releaseCapture && typeof drag.handle.releasePointerCapture === "function") {
    try {
      if (
        typeof drag.handle.hasPointerCapture !== "function" ||
        drag.handle.hasPointerCapture(drag.pointerId)
      ) {
        drag.handle.releasePointerCapture(drag.pointerId);
      }
    } catch (_error) {
      // Capture may already have been released by the browser terminal event.
    }
  }
  drag.handle.removeEventListener("lostpointercapture", drag.finish);
  state.reviewPerspectiveDrag = null;
  const authoritative =
    settle &&
    drag.generation === state.reviewPerspectiveGeneration &&
    isCurrentReviewSession(drag.session) &&
    state.reviewImage?.itemId === drag.itemId &&
    state.reviewImage?.imageId === drag.imageId &&
    state.reviewPerspectiveFrame === drag.frame &&
    state.reviewPerspectiveImage === drag.sourceImage &&
    drag.handle.parentNode === drag.frame &&
    drag.frame.parentNode;
  const deferredRender = state.reviewStageRenderDeferred;
  state.reviewStageRenderDeferred = false;
  if (!authoritative) {
    syncReviewDirtyState();
    return { authoritative: false, moved: drag.moved, deferredRender };
  }
  if (drag.moved) {
    state.reviewMessage = drag.messageAtStart;
    revokeReviewPreviewUrl();
    scheduleDraftPreview({ renderStage });
    if (!renderStage) {
      syncReviewDirtyState();
    }
  } else if (deferredRender && renderStage) {
    renderImageReview();
  } else {
    syncReviewDirtyState();
  }
  return { authoritative: true, moved: drag.moved, deferredRender };
}

function setPerspectiveCorner(
  index,
  x,
  y,
  handle,
  frame = handle?.sourceFrame,
  sourceImage = handle?.sourceImage,
  { schedulePreview = true } = {}
) {
  const corners = currentPerspectiveCorners();
  const nextCorner = {
    x: Math.max(0, Math.min(1, x)),
    y: Math.max(0, Math.min(1, y)),
  };
  if (corners[index].x === nextCorner.x && corners[index].y === nextCorner.y) {
    return false;
  }
  corners[index] = nextCorner;
  state.reviewDraftAdjustment.perspective = { corners };
  projectPerspectiveHandles();
  markReviewDraftChanged({ schedulePreview });
  return true;
}

function positionPerspectiveHandle(handle, corner, frame, sourceImage) {
  const frameBounds = frame?.getBoundingClientRect();
  const sourceBounds = frame && sourceImage ? sourceRenderedBounds(frame, sourceImage) : frameBounds;
  if (!frameBounds || !sourceBounds) {
    return;
  }
  const targetRadius = 22;
  const horizontalInset = Math.min(targetRadius, frameBounds.width / 2);
  const verticalInset = Math.min(targetRadius, frameBounds.height / 2);
  const sourceX = sourceBounds.left - frameBounds.left + corner.x * sourceBounds.width;
  const sourceY = sourceBounds.top - frameBounds.top + corner.y * sourceBounds.height;
  handle.style.left = `${Math.max(horizontalInset, Math.min(frameBounds.width - horizontalInset, sourceX))}px`;
  handle.style.top = `${Math.max(verticalInset, Math.min(frameBounds.height - verticalInset, sourceY))}px`;
}

function syncReviewControls() {
  const draft = state.reviewDraftAdjustment;
  for (const [id, value] of [
    ["review-rotation", draft.rotationDegrees],
    ["review-rotation-number", draft.rotationDegrees],
    ["review-zoom", draft.zoom],
    ["review-pan-x", draft.panX],
    ["review-pan-y", draft.panY],
  ]) {
    const control = $(`#${id}`);
    if (control) {
      control.value = value;
    }
  }
}

function setReviewComparisonMode(mode) {
  if (
    state.reviewMutationPending ||
    state.reviewPerspectiveDrag ||
    (mode !== "latest" && !state.reviewImage?.canComparePublicCurrent)
  ) {
    return;
  }
  state.reviewComparisonMode = mode;
  renderImageReview();
}

function syncReviewComparisonButtons() {
  const canCompare = Boolean(state.reviewImage?.canComparePublicCurrent);
  if (!canCompare && state.reviewComparisonMode !== "latest") {
    state.reviewComparisonMode = "latest";
  }
  for (const button of document.querySelectorAll("[data-review-mode]")) {
    const mode = button.dataset.reviewMode;
    const active = mode === state.reviewComparisonMode;
    const unavailable = mode !== "latest" && !canCompare;
    button.setAttribute("aria-pressed", active ? "true" : "false");
    button.classList.toggle("is-active", active);
    const disabled =
      unavailable || Boolean(state.reviewMutationPending) || Boolean(state.reviewPerspectiveDrag);
    button.disabled = disabled;
    button.setAttribute("aria-disabled", disabled ? "true" : "false");
  }
}

async function detectImageEdges() {
  const { itemId, imageId } = state.reviewImage || {};
  if (!itemId || !imageId || state.reviewMutationPending || state.reviewPerspectiveDrag) {
    return;
  }
  const session = state.reviewSession ? { ...state.reviewSession } : null;
  const draftRevision = state.reviewDraftRevision;
  try {
    const proposal = await jsonRequest(
      endpoints.imageAdjustmentAssist(itemId, imageId),
      "POST",
      { mediaRevision: session.mediaRevision }
    );
    if (
      !isCurrentReviewSession(session) ||
      draftRevision !== state.reviewDraftRevision ||
      state.reviewMutationPending ||
      state.reviewPerspectiveDrag
    ) {
      return;
    }
    if (proposal.status === "confident" && proposal.corners?.length === 4) {
      state.reviewDraftAdjustment.perspective = { corners: cloneAdjustment(proposal.corners) };
      state.reviewMessage = "Detected edges applied. Review the corners before saving.";
      markReviewDraftChanged();
      return;
    }
    state.reviewMessage = copy.assistUnavailable;
    elements.imageReviewMessage.textContent = state.reviewMessage;
  } catch (error) {
    if (handleMediaRevisionConflict(error)) {
      return;
    }
    if (
      isCurrentReviewSession(session) &&
      draftRevision === state.reviewDraftRevision &&
      !state.reviewMutationPending &&
      !state.reviewPerspectiveDrag &&
      error.status !== 401
    ) {
      state.reviewMessage = copy.assistUnavailable;
      elements.imageReviewMessage.textContent = state.reviewMessage;
    }
  }
}

async function saveImageAdjustments() {
  const { itemId, imageId } = state.reviewImage || {};
  if (!itemId || !imageId || state.reviewPerspectiveDrag || elements.imageReviewSave.disabled) {
    return;
  }
  const session = state.reviewSession ? { ...state.reviewSession } : null;
  const submittedRevision = state.reviewDraftRevision;
  const submittedAdjustment = canonicalReviewAdjustment(state.reviewDraftAdjustment);
  const mutation = beginReviewMutation("save", session, itemId, imageId);
  try {
    const item = await jsonRequest(
      endpoints.imageAdjustment(itemId, imageId),
      "PATCH",
      { mediaRevision: session.mediaRevision, adjustment: submittedAdjustment }
    );
    if (!isCurrentReviewMutation(mutation)) {
      return;
    }
    const returnedAdjustment = item.images?.find((image) => image.id === imageId)?.adjustment;
    const savedAdjustment = canonicalReviewAdjustment(returnedAdjustment ?? submittedAdjustment);
    const { reviewInvalidated } = reconcileAdminItemResponse(item);
    if (reviewInvalidated || !isCurrentReviewMutation(mutation)) {
      return;
    }
    state.reviewSavedAdjustment = savedAdjustment;
    if (state.reviewDraftRevision === submittedRevision) {
      state.reviewDraftAdjustment = cloneAdjustment(savedAdjustment);
    }
    state.reviewImage.adjustment = cloneAdjustment(savedAdjustment);
    state.reviewMessage = copy.adjustmentSaved;
    elements.imageReviewMessage.textContent = state.reviewMessage;
  } catch (error) {
    if (handleMediaRevisionConflict(error)) {
      return;
    }
    if (isCurrentReviewMutation(mutation) && error.status !== 401) {
      state.reviewMessage =
        "Adjustments did not save. Keep this page open, review the controls, and try again.";
      elements.imageReviewMessage.textContent = state.reviewMessage;
    }
  } finally {
    if (isCurrentReviewMutation(mutation)) {
      state.reviewMutationPending = null;
      syncReviewDirtyState();
    }
  }
}

function discardImageEdits() {
  if (blockReviewEgressWhileMutationPending()) {
    return;
  }
  if (state.reviewDirty && !window.confirm(copy.discardImageEdits)) {
    return;
  }
  clearImageReviewState();
  setView("add-item-view");
  renderImages(state.currentItem?.images || [], state.currentItem?.cleanupWarnings || []);
}

async function resetImageAdjustments() {
  if (
    state.reviewMutationPending ||
    state.reviewPerspectiveDrag ||
    !state.reviewImage ||
    !window.confirm(copy.resetAdjustment)
  ) {
    return;
  }
  const session = state.reviewSession ? { ...state.reviewSession } : null;
  const submittedRevision = state.reviewDraftRevision;
  const { itemId, imageId } = state.reviewImage;
  const mutation = beginReviewMutation("reset", session, itemId, imageId);
  try {
    const item = await jsonRequest(
      endpoints.imageAdjustment(itemId, imageId),
      "DELETE",
      { mediaRevision: session.mediaRevision }
    );
    if (!isCurrentReviewMutation(mutation)) {
      return;
    }
    const { reviewInvalidated } = reconcileAdminItemResponse(item);
    if (reviewInvalidated || !isCurrentReviewMutation(mutation)) {
      return;
    }
    state.reviewSavedAdjustment = identityReviewAdjustment();
    if (state.reviewDraftRevision === submittedRevision) {
      state.reviewDraftAdjustment = identityReviewAdjustment();
      state.reviewDraftRevision += 1;
    }
    state.reviewImage.adjustment = null;
    state.reviewMessage = "Adjustments cleared. The original upload is unchanged.";
    scheduleDraftPreview({ immediate: true });
  } catch (error) {
    if (handleMediaRevisionConflict(error)) {
      return;
    }
    if (isCurrentReviewMutation(mutation) && error.status !== 401) {
      state.reviewMessage = error.message;
      elements.imageReviewMessage.textContent = state.reviewMessage;
    }
  } finally {
    if (isCurrentReviewMutation(mutation)) {
      state.reviewMutationPending = null;
      syncReviewDirtyState();
    }
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
  const taxonomy = taxonomyPayload();
  const signerCredits = signerCreditPayload();
  const signer = signerCredits.map((credit) => credit.displayName).filter(Boolean).join(" + ");
  return {
    title: form.elements.title.value.trim(),
    signer,
    description: optional("description"),
    category: taxonomy.format,
    signerCredits,
    ...taxonomy,
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

function signerCreditPayload() {
  return Array.from(elements.signerRows.children)
    .map((row) => {
      const value = (field) => row.querySelector(`[data-signer-field="${field}"]`)?.value.trim() || null;
      return {
        signerId: row.dataset.signerId || null,
        displayName: value("name"),
        itemRole: value("role"),
        itemContext: value("context"),
      };
    })
    .filter((credit) => credit.displayName || credit.signerId);
}

function taxonomyPayload() {
  const form = elements.itemForm;
  return {
    characters: splitList(form.elements.characters.value),
    franchises: splitList(form.elements.franchises.value),
    productLine: optionalValue(form.elements.productLine.value),
    setName: optionalValue(form.elements.setName.value),
    format: form.elements.format.value,
    origin: form.elements.customItem.checked ? "Custom" : "Official",
    language: form.elements.language.value,
    tags: splitList(form.elements.tags.value),
  };
}

const optionalValue = (value) => {
  const trimmed = String(value || "").trim();
  return trimmed || null;
};

async function saveItem(event) {
  event.preventDefault();
  const id = elements.itemForm.elements.itemId.value.trim();
  const selectedFiles = Array.from(elements.imageFiles.files);
  const selectedAltText = elements.itemForm.elements.altText.value.trim();
  try {
    const item = await jsonRequest(id ? endpoints.item(id) : endpoints.items, id ? "PATCH" : "POST", formPayload());
    reconcileAdminItemResponse(item);
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
      reconcileAdminItemResponse(item);
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
  if (blockReviewEgressWhileMutationPending()) {
    return false;
  }
  if (state.reviewDirty) {
    setView("image-review-view");
    state.reviewMessage = "Save or discard image adjustments before publishing.";
    elements.imageReviewMessage.textContent = state.reviewMessage;
    elements.imageReviewMessage.focus();
    return false;
  }
  if (!state.dirty) {
    return true;
  }
  setView("add-item-view");
  elements.globalMessage.textContent = "Save item before publishing these changes.";
  elements.globalMessage.focus();
  return false;
}

function ensureSavedBeforeImageChange() {
  if (blockReviewEgressWhileMutationPending()) {
    return false;
  }
  if (state.reviewDirty && !confirmDiscardReviewForNavigation()) {
    return false;
  }
  if (!state.dirty) {
    return true;
  }
  setView("add-item-view");
  elements.globalMessage.textContent = "Save item before changing images.";
  elements.globalMessage.focus();
  return false;
}

function ensureSavedBeforeOpeningAnotherItem() {
  if (blockReviewEgressWhileMutationPending()) {
    return false;
  }
  if (state.reviewDirty && !confirmDiscardReviewForNavigation()) {
    return false;
  }
  if (!state.dirty) {
    return true;
  }
  setView("add-item-view");
  elements.globalMessage.textContent = "Save item before opening another item.";
  elements.globalMessage.focus();
  return false;
}

function ensureSavedBeforeManagingSigner() {
  if (blockReviewEgressWhileMutationPending()) {
    return false;
  }
  if (state.reviewDirty && !confirmDiscardReviewForNavigation()) {
    return false;
  }
  if (!state.dirty) {
    return true;
  }
  setView("add-item-view");
  elements.globalMessage.textContent = "Save item before managing this signer profile.";
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
    const publishMessage = status.cleanupWarning || copy.publishSuccess;
    setText("#publish-next-action", publishMessage);
    elements.globalMessage.textContent = publishMessage;
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

function openNewItemEditor() {
  if (ensureSavedBeforeOpeningAnotherItem()) {
    renderEditor();
  }
}

const markDirty = (event) => {
  if (uploadOnlyFieldNames.has(event?.target?.name)) {
    return;
  }
  state.dirty = true;
  elements.discardUnsaved.hidden = false;
  elements.publishFromEditor.setAttribute("aria-disabled", "true");
  elements.dirtyState.textContent = "Unsaved client-side edits. Save before publishing.";
  syncPublishAvailability();
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
  const form = event.currentTarget;
  const submit = form.querySelector("button[type=\"submit\"]");
  const originalSubmitText = submit.textContent;
  submit.disabled = true;
  submit.textContent = "Signing in...";
  elements.loginMessage.setAttribute("role", "status");
  elements.loginMessage.textContent = "Signing in...";
  try {
    await jsonRequest(endpoints.login, "POST", {
      password: form.elements.password.value,
    });
  } catch (error) {
    if (error.status === 401) {
      window.location.replace(publicHomePath);
      return;
    }
    elements.loginMessage.setAttribute("role", "alert");
    elements.loginMessage.textContent = error.status === 429 ? copy.lockout : "Login failed.";
    submit.disabled = false;
    submit.textContent = originalSubmitText;
    return;
  }

  const next = nextDestination();
  form.reset();
  elements.loginMessage.setAttribute("role", "alert");
  elements.loginMessage.textContent = "";
  submit.disabled = false;
  submit.textContent = originalSubmitText;
  if (window.location.pathname === adminRootPath && next === adminRootPath) {
    showWorkflow();
    await renderHub();
    return;
  }

  try {
    window.location.replace(next);
  } catch (error) {
    showWorkflow();
    elements.globalMessage.textContent = `Logged in, but navigation failed: ${error.message}`;
  }
});

elements.logout.addEventListener("click", async () => {
  if (blockReviewEgressWhileMutationPending()) {
    return;
  }
  if (state.reviewDirty && !window.confirm(copy.discardImageEditsForNavigation)) {
    return;
  }
  clearImageReviewState();
  try {
    await request(endpoints.logout, { method: "POST" });
  } finally {
    showLogin("Logged out.");
  }
});

for (const tab of elements.tabs) {
  tab.addEventListener("click", () => {
    if (tab.dataset.view === "add-item-view") {
      openNewItemEditor();
      return;
    }
    navigateToView(tab.dataset.view);
  });
}

$("#refresh-status").addEventListener("click", renderHub);
$("#refresh-diagnostics").addEventListener("click", renderHub);
$("#refresh-items").addEventListener("click", renderItemList);
$("#refresh-history").addEventListener("click", () => renderHistory());
$("#back-to-hub").addEventListener("click", () => navigateToView("hub-view"));
$("#add-another-item").addEventListener("click", openNewItemEditor);
$("#add-signer-row").addEventListener("click", () => {
  const index = elements.signerRows.children.length;
  elements.signerRows.append(signerRow({ signer: { displayName: "" } }, index));
  markDirty();
});
$("#discard-unsaved").addEventListener("click", () => renderEditor(state.currentItem));
$("#upload-more-images").addEventListener("click", () => uploadImages());
$("#publish-from-editor").addEventListener("click", publishFromEditor);
$("#publish-incremental").addEventListener("click", () => publishChanges("incremental"));
$("#publish-full").addEventListener("click", () => publishChanges("full"));
elements.imageReviewSave.addEventListener("click", saveImageAdjustments);
elements.imageReviewDiscard.addEventListener("click", discardImageEdits);
elements.imageReviewReset.addEventListener("click", resetImageAdjustments);
$("#image-review-detect").addEventListener("click", detectImageEdges);
for (const button of document.querySelectorAll("[data-review-mode]")) {
  button.addEventListener("click", () => setReviewComparisonMode(button.dataset.reviewMode));
}
for (const toggle of document.querySelectorAll("[data-review-overlay]")) {
  toggle.addEventListener("change", () => {
    if (state.reviewMutationPending || state.reviewPerspectiveDrag) {
      return;
    }
    state.reviewOverlays[toggle.dataset.reviewOverlay] = toggle.checked;
    renderImageReview();
  });
}
for (const control of document.querySelectorAll("[data-adjustment-field]")) {
  control.addEventListener("input", () => {
    if (!state.reviewDraftAdjustment || state.reviewMutationPending || state.reviewPerspectiveDrag) {
      return;
    }
    state.reviewDraftAdjustment[control.dataset.adjustmentField] = Number(control.value);
    markReviewDraftChanged();
  });
}

window.addEventListener("beforeunload", (event) => {
  if (!state.dirty && !state.reviewDirty && !state.reviewMutationPending) {
    return;
  }
  event.preventDefault();
  event.returnValue = "";
});

elements.itemForm.addEventListener("submit", saveItem);
elements.itemForm.addEventListener("input", markDirty);
elements.itemFilters.addEventListener("submit", (event) => {
  event.preventDefault();
  renderItemList();
});
elements.signerManagementForm.addEventListener("submit", (event) => {
  event.preventDefault();
  state.focusedSignerId = null;
  renderSignerManagement();
});

if (!window.__AUTOGRAPHS_STATIC_ADMIN_TEST__) {
  bootstrapSession();
}
