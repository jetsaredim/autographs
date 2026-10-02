import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";

class FakeClassList {
  constructor(element) {
    this.element = element;
  }

  add(name) {
    const names = new Set(this.element.className.split(/\s+/).filter(Boolean));
    names.add(name);
    this.element.className = [...names].join(" ");
  }

  toggle(name, force) {
    const names = new Set(this.element.className.split(/\s+/).filter(Boolean));
    if (force) {
      names.add(name);
    } else {
      names.delete(name);
    }
    this.element.className = [...names].join(" ");
  }
}

class FakeElement {
  constructor(tagName = "div", id = "") {
    this.tagName = tagName.toUpperCase();
    this.id = id;
    this.hidden = false;
    this.disabled = false;
    this.dataset = {};
    this.style = {};
    this.children = [];
    this.attributes = new Map();
    this.listeners = new Map();
    this.className = "";
    this.classList = new FakeClassList(this);
    this.textContent = "";
    this.elements = {};
  }

  append(...children) {
    this.children.push(...children);
  }

  replaceChildren(...children) {
    this.children = [...children];
  }

  addEventListener(type, listener) {
    const listeners = this.listeners.get(type) || [];
    listeners.push(listener);
    this.listeners.set(type, listeners);
  }

  removeEventListener(type, listener) {
    this.listeners.set(
      type,
      (this.listeners.get(type) || []).filter((candidate) => candidate !== listener)
    );
  }

  setAttribute(name, value) {
    this.attributes.set(name, String(value));
  }

  getAttribute(name) {
    return this.attributes.get(name);
  }

  focus() {
    this.focused = true;
  }

  setPointerCapture(pointerId) {
    this.pointerCapture = pointerId;
  }

  getBoundingClientRect() {
    return { left: 0, top: 0, width: 100, height: 100 };
  }

  querySelector() {
    return new FakeElement("button");
  }
}

const ids = new Map();
const elementForId = (id) => {
  if (!ids.has(id)) {
    ids.set(id, new FakeElement("div", id));
  }
  return ids.get(id);
};

const comparisonButtons = ["latest", "before-after", "split"].map((mode) => {
  const button = new FakeElement("button");
  button.dataset.reviewMode = mode;
  return button;
});
const views = ["hub-view", "add-item-view", "image-review-view", "publish-view"].map(elementForId);

const document = {
  querySelector(selector) {
    if (selector.startsWith("#")) {
      return elementForId(selector.slice(1));
    }
    return new FakeElement();
  },
  querySelectorAll(selector) {
    if (selector === ".view-panel") {
      return views;
    }
    if (selector === "[data-review-mode]") {
      return comparisonButtons;
    }
    return [];
  },
  createElement(tagName) {
    return new FakeElement(tagName);
  },
};

let confirmResult = true;
let timerId = 0;
const window = {
  __AUTOGRAPHS_STATIC_ADMIN_TEST__: true,
  location: {
    pathname: "/admin/",
    search: "",
    hash: "",
    origin: "https://autographs.example.test",
  },
  confirm() {
    return confirmResult;
  },
  setTimeout() {
    timerId += 1;
    return timerId;
  },
  clearTimeout() {},
  addEventListener() {},
};

class TestUrl extends URL {
  static createObjectURL() {
    return "blob:test-preview";
  }

  static revokeObjectURL() {}
}

const context = vm.createContext({
  AbortController,
  FormData,
  JSON,
  Map,
  Math,
  Number,
  Object,
  Promise,
  Set,
  String,
  URL: TestUrl,
  assert,
  console,
  document,
  fetch: async () => {
    throw new Error("unexpected fetch in static admin behavior test");
  },
  requestAnimationFrame(callback) {
    callback();
  },
  setConfirmResult(value) {
    confirmResult = value;
  },
  window,
});

const source = fs.readFileSync(new URL("../static-admin/admin.js", import.meta.url), "utf8");
vm.runInContext(source, context, { filename: "admin.js" });

vm.runInContext(
  `
  const fullFrame = identityReviewAdjustment();
  fullFrame.perspective = { corners: fullFramePerspectiveCorners() };
  assert.equal(canonicalReviewAdjustment(fullFrame).perspective, null);

  state.reviewSavedAdjustment = identityReviewAdjustment();
  state.reviewDraftAdjustment = identityReviewAdjustment();
  state.reviewDraftAdjustment.rotationDegrees = 3;
  syncReviewDirtyState();
  assert.equal(state.reviewDirty, true);
  assert.equal(elements.publishIncremental.disabled, true);
  state.reviewSavedAdjustment = cloneAdjustment(state.reviewDraftAdjustment);
  syncReviewDirtyState();
  assert.equal(state.reviewDirty, false);

  state.reviewImage = { canComparePublicCurrent: false };
  state.reviewComparisonMode = "split";
  syncReviewComparisonButtons();
  assert.equal(state.reviewComparisonMode, "latest");
  assert.equal(document.querySelectorAll("[data-review-mode]")[1].disabled, true);
  assert.equal(document.querySelectorAll("[data-review-mode]")[0].getAttribute("aria-pressed"), "true");

  state.reviewImage = { canComparePublicCurrent: true };
  state.reviewComparisonMode = "split";
  syncReviewComparisonButtons();
  assert.equal(document.querySelectorAll("[data-review-mode]")[2].getAttribute("aria-pressed"), "true");

  state.reviewSavedAdjustment = identityReviewAdjustment();
  state.reviewDraftAdjustment = identityReviewAdjustment();
  const handle = document.createElement("button");
  const keyEvent = {
    key: "ArrowRight",
    shiftKey: false,
    currentTarget: handle,
    preventDefault() {},
  };
  movePerspectiveHandle(0, keyEvent);
  movePerspectiveHandle(0, keyEvent);
  assert.equal(state.reviewDraftAdjustment.perspective.corners[0].x, 0.02);
  assert.equal(handle.style.left, "clamp(22px, 2%, calc(100% - 22px))");
  assert.equal(state.reviewFocusedCornerIndex, 0);

  setPerspectiveCorner(1, 0.75, 0.25, handle);
  assert.deepEqual(state.reviewDraftAdjustment.perspective.corners[1], { x: 0.75, y: 0.25 });

  const dragFrame = document.createElement("div");
  const dragHandle = document.createElement("button");
  beginPerspectiveDrag(2, {
    currentTarget: dragHandle,
    pointerId: 7,
    preventDefault() {},
  }, dragFrame);
  dragHandle.listeners.get("pointermove")[0]({ clientX: 80, clientY: 20 });
  assert.deepEqual(state.reviewDraftAdjustment.perspective.corners[2], { x: 0.8, y: 0.2 });
  assert.equal(dragHandle.pointerCapture, 7);

  state.dirty = false;
  state.reviewDirty = true;
  assert.equal(ensureSavedBeforePublish(), false);
  state.reviewSavedAdjustment = cloneAdjustment(state.reviewDraftAdjustment);
  syncReviewDirtyState();
  assert.equal(ensureSavedBeforePublish(), true);

  state.currentView = "image-review-view";
  state.reviewDirty = true;
  setConfirmResult(false);
  assert.equal(navigateToView("publish-view"), false);
  assert.equal(state.currentView, "image-review-view");
  setConfirmResult(true);
  assert.equal(navigateToView("publish-view"), true);
  assert.equal(state.currentView, "publish-view");
  assert.equal(state.reviewImage, null);

  const tile = renderImagePreviewFrame("item", { id: "image", altText: "Preview" });
  const failedImage = tile.children[0];
  failedImage.listeners.get("error")[0]();
  const retryButton = tile.children[0].children[1];
  retryButton.listeners.get("click")[0]();
  assert.match(tile.children[0].src, /retry=1$/);

  state.reviewImage = {
    canComparePublicCurrent: true,
    publicCurrentPreviewUrl: "/media/public.webp",
    message: "Compare",
  };
  state.reviewSavedAdjustment = identityReviewAdjustment();
  state.reviewDraftAdjustment = identityReviewAdjustment();
  state.reviewPreviewUrl = "blob:latest.webp";
  state.reviewPreviewStatus = "ready";
  state.reviewComparisonMode = "split";
  state.reviewMessage = "Compare";
  renderImageReview();
  const comparisonFrame = elements.imageReviewStage.children[0];
  assert.equal(comparisonFrame.children[0].src, "/media/public.webp");
  assert.equal(comparisonFrame.children[1].src, "blob:latest.webp");
  assert.equal(comparisonFrame.children[1].style.transform, undefined);
  `,
  context
);

console.log("static admin behavior tests passed");
