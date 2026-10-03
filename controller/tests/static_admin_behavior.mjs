import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";

class FakeClassList {
  constructor(element) { this.element = element; }
  add(name) {
    const names = new Set(this.element.className.split(/\s+/).filter(Boolean));
    names.add(name);
    this.element.className = [...names].join(" ");
  }
  toggle(name, force) {
    const names = new Set(this.element.className.split(/\s+/).filter(Boolean));
    const enabled = force === undefined ? !names.has(name) : force;
    if (enabled) names.add(name); else names.delete(name);
    this.element.className = [...names].join(" ");
  }
}

class FakeElement {
  constructor(tagName = "div", id = "") {
    Object.assign(this, {
      tagName: tagName.toUpperCase(), id, hidden: false, disabled: false, dataset: {}, style: {},
      children: [], attributes: new Map(), listeners: new Map(), className: "", textContent: "",
      elements: {}, value: "", checked: false,
      rect: { left: 0, top: 0, width: 400, height: 300 },
      naturalWidth: tagName === "img" ? 400 : 0, naturalHeight: tagName === "img" ? 300 : 0,
    });
    this.classList = new FakeClassList(this);
  }
  append(...children) {
    for (const child of children) {
      if (child && typeof child === "object") child.parentNode = this;
      this.children.push(child);
    }
  }
  replaceChildren(...children) {
    for (const child of this.children) if (child && typeof child === "object") child.parentNode = null;
    this.children = [];
    this.append(...children);
  }
  remove() {
    if (!this.parentNode) return;
    this.parentNode.children = this.parentNode.children.filter((child) => child !== this);
    this.parentNode = null;
  }
  addEventListener(type, listener) {
    const listeners = this.listeners.get(type) || [];
    listeners.push(listener);
    this.listeners.set(type, listeners);
  }
  removeEventListener(type, listener) {
    this.listeners.set(type, (this.listeners.get(type) || []).filter((value) => value !== listener));
  }
  async dispatch(type, init = {}) {
    const event = {
      preventDefault() { this.defaultPrevented = true; },
      target: this, currentTarget: this, ...init,
    };
    for (const listener of this.listeners.get(type) || []) await listener(event);
    return event;
  }
  setAttribute(name, value) { this.attributes.set(name, String(value)); }
  getAttribute(name) { return this.attributes.get(name); }
  focus() { this.focused = true; }
  setPointerCapture(pointerId) { this.pointerCapture = pointerId; }
  getBoundingClientRect() { return this.rect; }
  querySelector() { return new FakeElement("button"); }
  reset() {}
}

const ids = new Map();
const elementForId = (id) => {
  if (!ids.has(id)) ids.set(id, new FakeElement("div", id));
  return ids.get(id);
};
const adjustmentControls = [
  ["review-rotation", "rotationDegrees"], ["review-rotation-number", "rotationDegrees"],
  ["review-zoom", "zoom"], ["review-pan-x", "panX"], ["review-pan-y", "panY"],
].map(([id, field]) => {
  const control = elementForId(id);
  control.dataset.adjustmentField = field;
  return control;
});
const overlayToggles = ["grid", "centerline", "edges"].map((name) => {
  const toggle = elementForId(`review-overlay-${name}`);
  toggle.dataset.reviewOverlay = name;
  return toggle;
});
const comparisonButtons = ["latest", "before-after", "split"].map((mode) => {
  const button = new FakeElement("button");
  button.dataset.reviewMode = mode;
  return button;
});
const views = ["hub-view", "add-item-view", "image-review-view", "publish-view"].map(elementForId);
const tabButtons = ["hub-view", "publish-view"].map((view) => {
  const button = new FakeElement("button");
  button.dataset.view = view;
  return button;
});
const document = {
  querySelector(selector) { return selector.startsWith("#") ? elementForId(selector.slice(1)) : new FakeElement(); },
  querySelectorAll(selector) {
    if (selector === ".view-panel") return views;
    if (selector === ".tab-button") return tabButtons;
    if (selector === "[data-review-mode]") return comparisonButtons;
    if (selector === "[data-review-overlay]") return overlayToggles;
    if (selector === "[data-adjustment-field]") return adjustmentControls;
    return [];
  },
  createElement(tagName) { return new FakeElement(tagName); },
};

let confirmResult = true;
let timerId = 0;
const timers = new Map();
const fetchQueue = [];
const fetchCalls = [];
let blobId = 0;
const enqueueFetch = (value) => fetchQueue.push(value);
const deferred = () => {
  let resolve;
  let reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
};
const response = ({ status = 200, json, text = "", blob = new Blob(["preview"]) } = {}) => ({
  status, ok: status >= 200 && status < 300, statusText: status >= 400 ? "Request failed" : "OK",
  headers: { get: (name) => name.toLowerCase() === "content-type" && json !== undefined ? "application/json" : "image/webp" },
  async json() { return json; }, async text() { return text; }, async blob() { return blob; },
});
const flushTimers = async () => {
  while (timers.size) {
    const pending = [...timers.entries()].sort(([left], [right]) => left - right);
    timers.clear();
    for (const [, callback] of pending) await callback();
  }
};
const findByClass = (root, className) => {
  if (!root) return null;
  if (root.className?.split(/\s+/).includes(className)) return root;
  for (const child of root.children || []) {
    const found = findByClass(child, className);
    if (found) return found;
  }
  return null;
};
const findAllByClass = (root, className, found = []) => {
  if (!root) return found;
  if (root.className?.split(/\s+/).includes(className)) found.push(root);
  for (const child of root.children || []) findAllByClass(child, className, found);
  return found;
};
const window = {
  __AUTOGRAPHS_STATIC_ADMIN_TEST__: true,
  location: { pathname: "/admin/", search: "", hash: "", origin: "https://autographs.example.test" },
  confirm() { return confirmResult; },
  setTimeout(callback) { timerId += 1; timers.set(timerId, callback); return timerId; },
  clearTimeout(id) { timers.delete(id); },
  addEventListener() {},
};
class TestUrl extends URL {
  static createObjectURL() { blobId += 1; return `blob:test-preview-${blobId}`; }
  static revokeObjectURL() {}
}
const context = vm.createContext({
  AbortController, Blob, FormData, JSON, Map, Math, Number, Object, Promise, Set, String,
  URL: TestUrl, assert, console, deferred, document, enqueueFetch, fetchCalls, findAllByClass,
  findByClass, flushTimers, response,
  fetch: async (path, options = {}) => {
    fetchCalls.push({ path, options });
    if (!fetchQueue.length) throw new Error(`unexpected fetch ${path}`);
    return await fetchQueue.shift();
  },
  requestAnimationFrame(callback) { callback(); },
  setConfirmResult(value) { confirmResult = value; },
  window,
});
const source = fs.readFileSync(new URL("../static-admin/admin.js", import.meta.url), "utf8");
vm.runInContext(source, context, { filename: "admin.js" });
const run = async (code) => vm.runInContext(`(async () => { ${code} })()`, context);
const reviewPayload = (itemId, imageId, adjustment = null) => ({
  itemId, imageId, adjustment, canComparePublicCurrent: true,
  publicCurrentPreviewUrl: `/media/${imageId}-public.webp`,
  privatePreviewUrl: `/admin/api/items/${itemId}/images/${imageId}/preview`,
  draftPreviewUrl: `/admin/api/items/${itemId}/images/${imageId}/preview/draft`,
  sourceGuidePreviewUrl: `/admin/api/items/${itemId}/images/${imageId}/preview/source`,
  state: "published", message: "Compare latest with public current.",
});

await run(`
  const fullFrame = identityReviewAdjustment();
  fullFrame.perspective = { corners: fullFramePerspectiveCorners() };
  assert.equal(canonicalReviewAdjustment(fullFrame).perspective, null);
`);

enqueueFetch(response({ json: reviewPayload("item-a", "image-a") }));
await run(`
  state.currentItem = { id: "item-a", images: [{ id: "image-a", altText: "A" }], cleanupWarnings: [] };
  renderImages(state.currentItem.images, []);
  await elements.imageGrid.children[0].children[4].children[0].dispatch("click");
  assert.equal(state.reviewImage.imageId, "image-a");
  assert.equal(elements.imageReviewSave.disabled, true);
  assert.equal(elements.imageReviewDiscard.hidden, false);
  assert.equal(elements.imageReviewReset.hidden, false);
`);
enqueueFetch(response());
await flushTimers();
await run(`
  assert.equal(state.reviewPreviewStatus, "rendering");
  assert.equal(elements.imageReviewSave.disabled, true);
  await findByClass(elements.imageReviewStage, "review-image-latest").dispatch("load");
  assert.equal(state.reviewPreviewStatus, "ready");
  assert.equal(elements.imageReviewSave.disabled, true);
`);

enqueueFetch(response({ json: { id: "item-a", images: [{ id: "image-a", adjustment: null }] } }));
await run(`
  await elements.imageReviewReset.dispatch("click");
  assert.equal(fetchCalls.at(-1).options.method, "DELETE");
  assert.equal(state.reviewDirty, false);
`);
assert.equal(fetchQueue.length, 0, "reset response should be consumed");
enqueueFetch(response());
await flushTimers();
assert.equal(fetchQueue.length, 0, "reset preview response should be consumed");
await run(`await findByClass(elements.imageReviewStage, "review-image-latest").dispatch("load");`);

await run(`
  const rotation = document.querySelector("#review-rotation");
  rotation.value = "3";
  await rotation.dispatch("input");
  assert.equal(state.reviewDirty, true);
  assert.equal(state.reviewPreviewStatus, "loading");
  assert.equal(state.reviewDisplayedRevision, null);
  assert.equal(elements.imageReviewSave.disabled, true);
  assert.equal(findByClass(elements.imageReviewStage, "review-image-latest"), null);
`);
enqueueFetch(response());
await flushTimers();
assert.equal(fetchQueue.length, 0, "rotation preview response should be consumed");
await run(`
  const draftRequest = fetchCalls.at(-1);
  assert.equal(JSON.parse(draftRequest.options.body).rotationDegrees, 3);
  assert.equal(elements.imageReviewSave.disabled, true);
  await findByClass(elements.imageReviewStage, "review-image-latest").dispatch("load");
  assert.equal(elements.imageReviewSave.disabled, false);
`);

await run(`
  const rotation = document.querySelector("#review-rotation");
  rotation.value = "4";
  await rotation.dispatch("input");
`);
enqueueFetch(response({ status: 500, text: "failed" }));
await flushTimers();
assert.equal(fetchQueue.length, 0, "failed preview response should be consumed");
await run(`
  assert.equal(state.reviewPreviewStatus, "error");
  assert.equal(elements.imageReviewSave.disabled, true);
  assert.equal(findByClass(elements.imageReviewStage, "review-image-latest"), null);
`);

await run(`
  const rotation = document.querySelector("#review-rotation");
  rotation.value = "5";
  await rotation.dispatch("input");
`);
enqueueFetch(response());
await flushTimers();
assert.equal(fetchQueue.length, 0, "recovered preview response should be consumed");
await run(`await findByClass(elements.imageReviewStage, "review-image-latest").dispatch("load");`);

const saveResponse = deferred();
enqueueFetch(saveResponse.promise);
const savePromise = run(`return elements.imageReviewSave.dispatch("click")`);
await Promise.resolve();
await run(`
  assert.equal(state.reviewMutationPending, "save");
  assert.equal(document.querySelector("#review-rotation").disabled, true);
  const rotation = document.querySelector("#review-rotation");
  rotation.value = "6";
  await rotation.dispatch("input");
`);
saveResponse.resolve(response({ json: {
  id: "item-a",
  images: [{ id: "image-a", adjustment: { rotationDegrees: 5, zoom: 1, panX: 0, panY: 0, crop: null, perspective: null } }],
} }));
await savePromise;
await run(`
  const patch = fetchCalls.findLast((call) => call.options.method === "PATCH");
  assert.equal(JSON.parse(patch.options.body).rotationDegrees, 5);
  assert.equal(state.reviewSavedAdjustment.rotationDegrees, 5);
  assert.equal(state.reviewDraftAdjustment.rotationDegrees, 6);
  assert.equal(state.reviewDirty, true);
  assert.equal(elements.imageReviewSave.disabled, true);
`);
enqueueFetch(response());
await flushTimers();
await run(`
  await findByClass(elements.imageReviewStage, "review-image-latest").dispatch("load");
  assert.equal(elements.imageReviewSave.disabled, false);
`);

await run(`await elements.imageReviewDiscard.dispatch("click");`);
const reviewA = deferred();
const reviewB = deferred();
enqueueFetch(reviewA.promise);
const openA = run(`
  state.currentItem = { id: "item-a", images: [{ id: "image-a" }], cleanupWarnings: [] };
  renderImages(state.currentItem.images, []);
  return elements.imageGrid.children[0].children[4].children[0].dispatch("click");
`);
await Promise.resolve();
enqueueFetch(reviewB.promise);
const openB = run(`
  state.currentItem = { id: "item-b", images: [{ id: "image-b" }], cleanupWarnings: [] };
  renderImages(state.currentItem.images, []);
  return elements.imageGrid.children[0].children[4].children[0].dispatch("click");
`);
reviewA.resolve(response({ json: reviewPayload("item-a", "image-a") }));
await openA;
await run(`assert.equal(state.reviewImage, null);`);
reviewB.resolve(response({ json: reviewPayload("item-b", "image-b", { rotationDegrees: 7, zoom: 1, panX: 0, panY: 0, crop: null, perspective: null }) }));
await openB;
await run(`assert.equal(state.reviewImage.imageId, "image-b");`);

const assist = deferred();
enqueueFetch(assist.promise);
const assistPromise = run(`return document.querySelector("#image-review-detect").dispatch("click")`);
await Promise.resolve();
await run(`await elements.imageReviewDiscard.dispatch("click");`);
enqueueFetch(response({ json: reviewPayload("item-b", "image-b") }));
await run(`
  state.currentItem = { id: "item-b", images: [{ id: "image-b" }], cleanupWarnings: [] };
  renderImages(state.currentItem.images, []);
  await elements.imageGrid.children[0].children[4].children[0].dispatch("click");
`);
assist.resolve(response({ json: { status: "confident", corners: [{ x: 0.1, y: 0.1 }, { x: 0.9, y: 0.1 }, { x: 0.9, y: 0.9 }, { x: 0.1, y: 0.9 }] } }));
await assistPromise;
await run(`assert.equal(state.reviewDraftAdjustment.perspective, null);`);

await run(`
  const sourceFrame = findByClass(elements.imageReviewStage, "source-guide-frame");
  const sourceImage = sourceFrame.children[0];
  sourceFrame.rect = { left: 0, top: 0, width: 400, height: 300 };
  sourceImage.naturalWidth = 200;
  sourceImage.naturalHeight = 400;
  await sourceImage.dispatch("load");
  const handle = findAllByClass(sourceFrame, "corner-handle")[0];
  await handle.dispatch("pointerdown", { pointerId: 9 });
  await handle.dispatch("pointermove", { clientX: 125, clientY: 0 });
  assert.deepEqual(state.reviewDraftAdjustment.perspective.corners[0], { x: 0, y: 0 });
  await handle.dispatch("pointermove", { clientX: 200, clientY: 150 });
  assert.deepEqual(state.reviewDraftAdjustment.perspective.corners[0], { x: 0.5, y: 0.5 });
  assert.match(sourceImage.src, /preview\\/source$/);
`);

const lateReset = deferred();
enqueueFetch(lateReset.promise);
const resetPromise = run(`return elements.imageReviewReset.dispatch("click")`);
await Promise.resolve();
await run(`
  assert.equal(state.reviewMutationPending, "reset");
  await elements.imageReviewDiscard.dispatch("click");
`);
enqueueFetch(response({ json: reviewPayload("item-b", "image-b", { rotationDegrees: 9, zoom: 1, panX: 0, panY: 0, crop: null, perspective: null }) }));
await run(`
  state.currentItem = { id: "item-b", images: [{ id: "image-b" }], cleanupWarnings: [] };
  renderImages(state.currentItem.images, []);
  await elements.imageGrid.children[0].children[4].children[0].dispatch("click");
`);
lateReset.resolve(response({ json: { id: "item-b", images: [{ id: "image-b", adjustment: null }] } }));
await resetPromise;
await run(`
  assert.equal(state.reviewImage.imageId, "image-b");
  assert.equal(state.reviewDraftAdjustment.rotationDegrees, 9);
  const beforePublish = fetchCalls.length;
  const rotation = document.querySelector("#review-rotation");
  rotation.value = "10";
  await rotation.dispatch("input");
  await elements.publishIncremental.dispatch("click");
  assert.equal(fetchCalls.length, beforePublish);
`);

await run(`await elements.imageReviewDiscard.dispatch("click");`);
const lateReview = deferred();
enqueueFetch(lateReview.promise);
const lateOpen = run(`
  state.currentItem = { id: "item-a", images: [{ id: "image-a" }], cleanupWarnings: [] };
  renderImages(state.currentItem.images, []);
  return elements.imageGrid.children[0].children[4].children[0].dispatch("click");
`);
await Promise.resolve();
await run(`await elements.tabs.find((tab) => tab.dataset.view === "publish-view").dispatch("click");`);
lateReview.resolve(response({ json: reviewPayload("item-a", "image-a") }));
await lateOpen;
await run(`
  assert.equal(state.currentView, "publish-view");
  assert.equal(state.reviewImage, null);
`);

console.log("static admin behavior tests passed");
